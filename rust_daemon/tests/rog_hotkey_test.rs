#![allow(dead_code)]

#[path = "../src/hardware/mod.rs"]
mod hardware;
#[path = "../src/rog_hotkey.rs"]
mod rog_hotkey;
#[path = "../src/rog_input.rs"]
mod rog_input;

use anyhow::{anyhow, Result};
use rog_hotkey::{
    fallback_enabled, handle_after_monitor_spawn, hotkey_enabled, monitor_policy,
    next_failure_count, preflight_then_prepare, require_prepared, retry_delay, should_warn_failure,
    take_initial_or_open, FirmwareRestorer, RogHotkeyHandle, RogHotkeyStatus, RogHotkeyStatusStore,
    HEALTHY_SESSION,
};
use std::os::unix::net::UnixStream;
use std::sync::{
    atomic::{AtomicBool, AtomicUsize, Ordering},
    Arc, Mutex,
};
use std::thread;
use std::time::Duration;

#[test]
fn unsupported_status_has_a_stable_json_schema() {
    let status = RogHotkeyStatus::unsupported("probe failed");

    assert_eq!(
        status.to_json().unwrap(),
        r#"{"schemaVersion":1,"hardwareSupported":false,"firmwareChanged":false,"fallbackEnabled":false,"inputPath":"/dev/input/by-path/platform-asus-nb-wmi-event","scanCode":56,"keyCode":148,"hardwareIdentity":null,"reason":"probe failed","monitorState":"disabled","lastError":null,"droppedPresses":0,"signalError":null,"restorationState":"notNeeded"}"#
    );
}

#[test]
fn software_fallback_requires_the_exact_opt_in_value() {
    assert!(fallback_enabled(Some("1")));
    assert!(!fallback_enabled(None));
    assert!(!fallback_enabled(Some("0")));
    assert!(!fallback_enabled(Some("true")));
    assert!(!fallback_enabled(Some(" 1")));
    assert!(!fallback_enabled(Some("1 ")));
}

#[test]
fn rog_runtime_requires_exact_explicit_opt_in() {
    assert!(hotkey_enabled(Some("1")));
    assert!(!hotkey_enabled(None));
    assert!(!hotkey_enabled(Some("true")));
    assert!(!hotkey_enabled(Some("0")));
}

#[test]
fn input_preflight_runs_before_firmware_prepare_and_hands_device_off() {
    let calls = Arc::new(Mutex::new(Vec::new()));
    let open_calls = calls.clone();
    let prepare_calls = calls.clone();

    let (device, guard) = preflight_then_prepare(
        move || {
            open_calls.lock().unwrap().push("open");
            Ok::<_, anyhow::Error>("already-open-device")
        },
        move || {
            prepare_calls.lock().unwrap().push("prepare");
            Ok::<_, anyhow::Error>("firmware-guard")
        },
    )
    .unwrap();

    assert_eq!(*calls.lock().unwrap(), vec!["open", "prepare"]);
    assert_eq!(device, "already-open-device");
    assert_eq!(guard, "firmware-guard");
}

#[test]
fn input_preflight_failure_never_prepares_firmware() {
    let prepare_calls = Arc::new(AtomicUsize::new(0));
    let calls = prepare_calls.clone();

    let result = preflight_then_prepare::<(), (), _, _>(
        || Err(anyhow!("input unavailable")),
        move || {
            calls.fetch_add(1, Ordering::Relaxed);
            Ok(())
        },
    );

    assert!(result.is_err());
    assert_eq!(prepare_calls.load(Ordering::Relaxed), 0);
}

#[test]
fn first_monitor_session_consumes_preopened_device_without_reopen() {
    let mut initial_device = Some("preopened-device");
    let reopen_calls = Arc::new(AtomicUsize::new(0));
    let calls = reopen_calls.clone();

    let device = take_initial_or_open(&mut initial_device, move || {
        calls.fetch_add(1, Ordering::Relaxed);
        Ok::<_, anyhow::Error>("reopened-device")
    })
    .unwrap();

    assert_eq!(device, "preopened-device");
    assert!(initial_device.is_none());
    assert_eq!(reopen_calls.load(Ordering::Relaxed), 0);
}

#[test]
fn disabled_runtime_gate_precedes_input_and_hid_access() {
    let source = include_str!("../src/rog_hotkey.rs");
    let enabled_gate = source.find("if !hotkey_enabled").unwrap();
    let input_open = source.rfind("open_input_device(ROG_INPUT_PATH)").unwrap();
    let hid_probe = source.find("HidMKeyTransport::discover_and_probe").unwrap();

    assert!(enabled_gate < input_open);
    assert!(enabled_gate < hid_probe);
}

#[test]
fn unsupported_hardware_without_fallback_does_not_start_monitor() {
    let policy = monitor_policy(false, false);

    assert!(!policy.start_monitor);
    assert!(!policy.fallback_enabled);
}

#[test]
fn unsupported_hardware_with_fallback_starts_monitor_in_fallback_mode() {
    let policy = monitor_policy(false, true);

    assert!(policy.start_monitor);
    assert!(policy.fallback_enabled);
}

#[test]
fn supported_hardware_starts_monitor_without_fallback_mode() {
    let policy = monitor_policy(true, true);

    assert!(policy.start_monitor);
    assert!(!policy.fallback_enabled);
}

#[test]
fn prepare_failure_is_fatal_even_when_fallback_was_requested() {
    let error = require_prepared::<()>(Err(anyhow!("feature write failed"))).unwrap_err();

    assert!(error
        .to_string()
        .contains("firmware state may be uncertain"));
    assert!(error.to_string().contains("daemon refuses to continue"));
}

#[test]
fn retry_delay_never_exceeds_one_second() {
    assert_eq!(retry_delay(), Duration::from_secs(1));
    assert!(retry_delay() <= Duration::from_secs(1));
}

#[test]
fn warning_schedule_is_limited_to_first_and_power_of_two_failures() {
    let decisions = (1..=10).map(should_warn_failure).collect::<Vec<_>>();

    assert_eq!(
        decisions,
        vec![true, true, false, true, false, false, false, true, false, false]
    );
}

#[test]
fn flapping_open_then_read_errors_keep_advancing_failure_streak() {
    let mut failures = 0;
    for _ in 0..5 {
        failures = next_failure_count(failures, Some(Duration::from_millis(10)));
    }
    assert_eq!(failures, 5);
}

#[test]
fn genuinely_healthy_session_resets_failure_streak() {
    assert_eq!(next_failure_count(7, Some(HEALTHY_SESSION)), 1);
    assert_eq!(next_failure_count(7, None), 8);
}

struct FakeRestorer {
    attempts: Arc<AtomicUsize>,
    failures: usize,
    shutdown: Arc<AtomicBool>,
    events: Arc<Mutex<Vec<&'static str>>>,
    drops: Option<Arc<AtomicUsize>>,
}

impl FirmwareRestorer for FakeRestorer {
    fn restore(&mut self) -> Result<()> {
        assert!(self.shutdown.load(Ordering::Acquire));
        self.events.lock().unwrap().push("restore");
        let attempt = self.attempts.fetch_add(1, Ordering::Relaxed) + 1;
        if attempt <= self.failures {
            Err(anyhow!("restore attempt {attempt} failed"))
        } else {
            Ok(())
        }
    }
}

impl Drop for FakeRestorer {
    fn drop(&mut self) {
        if let Some(drops) = &self.drops {
            drops.fetch_add(1, Ordering::Relaxed);
        }
    }
}

fn test_restorer(
    failures: usize,
    shutdown: Arc<AtomicBool>,
    events: Arc<Mutex<Vec<&'static str>>>,
) -> (Box<dyn FirmwareRestorer>, Arc<AtomicUsize>) {
    let attempts = Arc::new(AtomicUsize::new(0));
    (
        Box::new(FakeRestorer {
            attempts: attempts.clone(),
            failures,
            shutdown,
            events,
            drops: None,
        }),
        attempts,
    )
}

#[test]
fn shutdown_sets_flag_then_joins_monitor_before_restoring() {
    let shutdown = Arc::new(AtomicBool::new(false));
    let events = Arc::new(Mutex::new(Vec::new()));
    let thread_shutdown = shutdown.clone();
    let thread_events = events.clone();
    let monitor = thread::spawn(move || {
        while !thread_shutdown.load(Ordering::Acquire) {
            thread::yield_now();
        }
        thread_events.lock().unwrap().push("join");
    });
    let (restorer, _) = test_restorer(0, shutdown.clone(), events.clone());
    let status = RogHotkeyStatusStore::new(RogHotkeyStatus::unsupported("test"));
    let mut handle =
        RogHotkeyHandle::from_parts_for_test(Some(restorer), shutdown, Some(monitor), status);

    handle.shutdown().unwrap();

    assert_eq!(*events.lock().unwrap(), vec!["join", "restore"]);
}

#[test]
fn successful_shutdown_restores_once_and_updates_live_status() {
    let shutdown = Arc::new(AtomicBool::new(false));
    let events = Arc::new(Mutex::new(Vec::new()));
    let (restorer, attempts) = test_restorer(0, shutdown.clone(), events);
    let status = RogHotkeyStatusStore::new(RogHotkeyStatus::unsupported("test"));
    status.set_restoration_state("pending");
    let mut handle = handle_after_monitor_spawn(
        Some(restorer),
        shutdown,
        status.clone(),
        Ok(thread::spawn(|| {})),
    )
    .unwrap();

    handle.shutdown().unwrap();

    assert_eq!(attempts.load(Ordering::Relaxed), 1);
    assert_eq!(status.snapshot().restoration_state, "restored");
}

#[test]
fn cancellation_wake_failure_does_not_override_successful_restore() {
    let shutdown = Arc::new(AtomicBool::new(false));
    let events = Arc::new(Mutex::new(Vec::new()));
    let (restorer, attempts) = test_restorer(0, shutdown.clone(), events);
    let status = RogHotkeyStatusStore::new(RogHotkeyStatus::unsupported("test"));
    let (cancellation_read, cancellation_write) = UnixStream::pair().unwrap();
    drop(cancellation_read);
    let mut handle =
        RogHotkeyHandle::from_parts_for_test(Some(restorer), shutdown, None, status.clone());
    handle.set_cancellation_for_test(cancellation_write);

    handle.shutdown().unwrap();

    assert_eq!(attempts.load(Ordering::Relaxed), 1);
    assert!(status
        .snapshot()
        .last_error
        .unwrap()
        .contains("wake ROG input monitor"));
}

#[test]
fn status_store_serializes_fresh_signal_and_drop_diagnostics() {
    let status = RogHotkeyStatusStore::new(RogHotkeyStatus::unsupported("test"));
    status.dropped_counter().store(7, Ordering::Relaxed);
    status.set_signal_error(Some("bus unavailable".to_string()));

    let json = status.to_json().unwrap();

    assert!(json.contains(r#""droppedPresses":7"#));
    assert!(json.contains(r#""signalError":"bus unavailable""#));
}

#[test]
fn restoration_failure_retries_three_times_and_propagates() {
    let shutdown = Arc::new(AtomicBool::new(false));
    let events = Arc::new(Mutex::new(Vec::new()));
    let (restorer, attempts) = test_restorer(3, shutdown.clone(), events);
    let status = RogHotkeyStatusStore::new(RogHotkeyStatus::unsupported("test"));
    let mut handle =
        RogHotkeyHandle::from_parts_for_test(Some(restorer), shutdown, None, status.clone());

    let error = handle.shutdown().unwrap_err();

    assert_eq!(attempts.load(Ordering::Relaxed), 3);
    assert!(error.to_string().contains("after 3 attempts"));
    assert_eq!(status.snapshot().restoration_state, "failed");
}

#[test]
fn monitor_spawn_failure_retries_restoration_and_combines_errors() {
    let shutdown = Arc::new(AtomicBool::new(false));
    let events = Arc::new(Mutex::new(Vec::new()));
    let (restorer, attempts) = test_restorer(3, shutdown.clone(), events);
    let status = RogHotkeyStatusStore::new(RogHotkeyStatus::unsupported("test"));

    let result = handle_after_monitor_spawn(
        Some(restorer),
        shutdown,
        status,
        Err(std::io::Error::other("spawn failed")),
    );

    let error = result.err().unwrap().to_string();
    assert_eq!(attempts.load(Ordering::Relaxed), 3);
    assert!(error.contains("spawn failed"));
    assert!(error.contains("after 3 attempts"));
}

#[test]
fn signals_are_registered_before_firmware_runtime_starts() {
    let source = include_str!("../src/main.rs");
    let sigint = source.find("SignalKind::interrupt").unwrap();
    let sigterm = source.find("SignalKind::terminate").unwrap();
    let dbus_ready = source.find(".build()\n        .await?").unwrap();
    let pending_guard = source.rfind("shutdown_already_pending(").unwrap();
    let startup = source.find("rog_hotkey::start(").unwrap();
    let started_log = source
        .find("ASUS Control Daemon started successfully")
        .unwrap();

    assert!(sigint < startup);
    assert!(sigterm < startup);
    assert!(dbus_ready < startup);
    assert!(pending_guard < startup);
    assert!(startup < started_log);
}

#[test]
fn fallible_monitor_resources_are_created_before_firmware_prepare() {
    let source = include_str!("../src/rog_hotkey.rs");
    let cancellation_pair = source.find("UnixStream::pair()?").unwrap();
    let firmware_prepare = source.find("FirmwareBindingGuard::prepare").unwrap();

    assert!(cancellation_pair < firmware_prepare);
}
