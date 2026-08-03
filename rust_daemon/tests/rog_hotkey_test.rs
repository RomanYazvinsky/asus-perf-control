#![allow(dead_code)]

#[path = "../src/hardware/mod.rs"]
mod hardware;
#[path = "../src/rog_hotkey.rs"]
mod rog_hotkey;
#[path = "../src/rog_input.rs"]
mod rog_input;

use anyhow::{anyhow, Result};
use rog_hotkey::{
    fallback_enabled, handle_after_monitor_spawn, monitor_policy, require_prepared, retry_delay,
    should_warn_failure, FirmwareRestorer, RogHotkeyHandle, RogHotkeyStatus, RogHotkeyStatusStore,
};
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
    let error = require_prepared::<()>(Err(anyhow!("feature write failed")), true).unwrap_err();

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
fn monitor_spawn_failure_drops_prepared_restorer_for_raii_restore() {
    let shutdown = Arc::new(AtomicBool::new(false));
    let restore_count = Arc::new(AtomicUsize::new(0));
    struct RaiiRestorer(Arc<AtomicUsize>);
    impl FirmwareRestorer for RaiiRestorer {
        fn restore(&mut self) -> Result<()> {
            Ok(())
        }
    }
    impl Drop for RaiiRestorer {
        fn drop(&mut self) {
            self.0.fetch_add(1, Ordering::Relaxed);
        }
    }
    let restorer: Box<dyn FirmwareRestorer> = Box::new(RaiiRestorer(restore_count.clone()));
    let status = RogHotkeyStatusStore::new(RogHotkeyStatus::unsupported("test"));

    let result = handle_after_monitor_spawn(
        Some(restorer),
        shutdown,
        status,
        Err(std::io::Error::other("spawn failed")),
    );

    assert!(result.is_err());
    assert_eq!(restore_count.load(Ordering::Relaxed), 1);
}

#[test]
fn signals_are_registered_before_firmware_runtime_starts() {
    let source = include_str!("../src/main.rs");
    let sigint = source.find("SignalKind::interrupt").unwrap();
    let sigterm = source.find("SignalKind::terminate").unwrap();
    let startup = source.find("rog_hotkey::start()").unwrap();

    assert!(sigint < startup);
    assert!(sigterm < startup);
}
