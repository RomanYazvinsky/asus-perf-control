use crate::hardware::rog_mkey::{FirmwareBindingGuard, HidMKeyTransport};
use crate::rog_input::{monitor_once, receiver_disconnected, KEY_PROG1, ROG_INPUT_PATH, ROG_SCAN};
use anyhow::{anyhow, Result};
use log::{debug, error, warn};
use serde::Serialize;
use std::sync::{
    atomic::{AtomicBool, AtomicU64, Ordering},
    Arc, Mutex,
};
use std::thread::{self, JoinHandle};
use std::time::Duration;
use tokio::sync::mpsc::{self, Receiver, Sender};

const FALLBACK_ENV: &str = "ASUS_ROG_HOTKEY_SOFTWARE_FALLBACK";
const RESTORE_ATTEMPTS: usize = 3;
const RESTORE_RETRY_DELAY: Duration = Duration::from_millis(100);

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RogHotkeyStatus {
    pub schema_version: u8,
    pub hardware_supported: bool,
    pub firmware_changed: bool,
    pub fallback_enabled: bool,
    pub input_path: String,
    pub scan_code: u16,
    pub key_code: u16,
    pub hardware_identity: Option<String>,
    pub reason: Option<String>,
    pub monitor_state: String,
    pub last_error: Option<String>,
    pub dropped_presses: u64,
    pub signal_error: Option<String>,
    pub restoration_state: String,
}

impl RogHotkeyStatus {
    pub fn unsupported(reason: impl Into<String>) -> Self {
        Self {
            schema_version: 1,
            hardware_supported: false,
            firmware_changed: false,
            fallback_enabled: false,
            input_path: ROG_INPUT_PATH.to_string(),
            scan_code: ROG_SCAN as u16,
            key_code: KEY_PROG1,
            hardware_identity: None,
            reason: Some(reason.into()),
            monitor_state: "disabled".to_string(),
            last_error: None,
            dropped_presses: 0,
            signal_error: None,
            restoration_state: "notNeeded".to_string(),
        }
    }

    fn supported(identity: String, firmware_changed: bool) -> Self {
        Self {
            schema_version: 1,
            hardware_supported: true,
            firmware_changed,
            fallback_enabled: false,
            input_path: ROG_INPUT_PATH.to_string(),
            scan_code: ROG_SCAN as u16,
            key_code: KEY_PROG1,
            hardware_identity: Some(identity),
            reason: None,
            monitor_state: "starting".to_string(),
            last_error: None,
            dropped_presses: 0,
            signal_error: None,
            restoration_state: if firmware_changed {
                "pending".to_string()
            } else {
                "notNeeded".to_string()
            },
        }
    }

    pub fn to_json(&self) -> serde_json::Result<String> {
        serde_json::to_string(self)
    }
}

#[derive(Clone)]
pub struct RogHotkeyStatusStore {
    status: Arc<Mutex<RogHotkeyStatus>>,
    dropped_presses: Arc<AtomicU64>,
}

impl RogHotkeyStatusStore {
    pub fn new(status: RogHotkeyStatus) -> Self {
        Self {
            status: Arc::new(Mutex::new(status)),
            dropped_presses: Arc::new(AtomicU64::new(0)),
        }
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, RogHotkeyStatus> {
        self.status
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    pub fn snapshot(&self) -> RogHotkeyStatus {
        let mut status = self.lock().clone();
        status.dropped_presses = self.dropped_presses.load(Ordering::Relaxed);
        status
    }

    pub fn to_json(&self) -> serde_json::Result<String> {
        self.snapshot().to_json()
    }

    pub fn set_monitor_state(&self, state: &str) {
        self.lock().monitor_state = state.to_string();
    }

    pub fn set_last_error(&self, message: Option<String>) {
        self.lock().last_error = message;
    }

    pub fn set_signal_error(&self, message: Option<String>) {
        self.lock().signal_error = message;
    }

    pub fn set_restoration_state(&self, state: &str) {
        self.lock().restoration_state = state.to_string();
    }

    #[doc(hidden)]
    pub fn dropped_counter(&self) -> Arc<AtomicU64> {
        self.dropped_presses.clone()
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct MonitorPolicy {
    pub start_monitor: bool,
    pub fallback_enabled: bool,
}

pub fn fallback_enabled(value: Option<&str>) -> bool {
    value == Some("1")
}

pub fn monitor_policy(hardware_supported: bool, fallback_requested: bool) -> MonitorPolicy {
    MonitorPolicy {
        start_monitor: hardware_supported || fallback_requested,
        fallback_enabled: !hardware_supported && fallback_requested,
    }
}

pub fn require_prepared<T>(result: Result<T>, _fallback_requested: bool) -> Result<T> {
    result.map_err(|error| {
        anyhow!(
            "failed to prepare ROG firmware binding; firmware state may be uncertain and daemon refuses to continue: {error:#}"
        )
    })
}

pub fn next_backoff(current: Duration) -> Duration {
    match current.as_secs() {
        0 => Duration::from_secs(1),
        1 => Duration::from_secs(2),
        2 => Duration::from_secs(4),
        4 => Duration::from_secs(8),
        8 => Duration::from_secs(16),
        16 => Duration::from_secs(32),
        _ => Duration::from_secs(60),
    }
}

pub trait FirmwareRestorer: Send {
    fn restore(&mut self) -> Result<()>;
}

impl FirmwareRestorer for FirmwareBindingGuard<HidMKeyTransport> {
    fn restore(&mut self) -> Result<()> {
        FirmwareBindingGuard::restore(self)
    }
}

pub struct RogHotkeyHandle {
    restorer: Option<Box<dyn FirmwareRestorer>>,
    shutdown: Arc<AtomicBool>,
    monitor_thread: Option<JoinHandle<()>>,
    status: RogHotkeyStatusStore,
}

impl RogHotkeyHandle {
    #[cfg(test)]
    #[doc(hidden)]
    pub fn from_parts_for_test(
        restorer: Option<Box<dyn FirmwareRestorer>>,
        shutdown: Arc<AtomicBool>,
        monitor_thread: Option<JoinHandle<()>>,
        status: RogHotkeyStatusStore,
    ) -> Self {
        Self {
            restorer,
            shutdown,
            monitor_thread,
            status,
        }
    }

    fn finish(&mut self, restore_attempts: usize) -> Result<()> {
        self.shutdown.store(true, Ordering::Release);
        let mut failures = Vec::new();

        if let Some(monitor_thread) = self.monitor_thread.take() {
            if monitor_thread.join().is_err() {
                failures.push("ROG input monitor thread panicked".to_string());
            }
        }

        if let Some(restorer) = self.restorer.as_mut() {
            let mut last_error = None;
            for attempt in 1..=restore_attempts {
                match restorer.restore() {
                    Ok(()) => {
                        if self.status.snapshot().restoration_state == "pending" {
                            self.status.set_restoration_state("restored");
                        }
                        self.restorer = None;
                        last_error = None;
                        break;
                    }
                    Err(error) => {
                        last_error = Some(error);
                        if attempt < restore_attempts {
                            thread::sleep(RESTORE_RETRY_DELAY);
                        }
                    }
                }
            }

            if let Some(error) = last_error {
                self.status.set_restoration_state("failed");
                failures.push(format!(
                    "failed to restore ROG firmware binding after {restore_attempts} attempts: {error:#}"
                ));
            }
        }

        if failures.is_empty() {
            Ok(())
        } else {
            Err(anyhow!(failures.join("; ")))
        }
    }

    pub fn shutdown(&mut self) -> Result<()> {
        self.finish(RESTORE_ATTEMPTS)
    }
}

impl Drop for RogHotkeyHandle {
    fn drop(&mut self) {
        if let Err(error) = self.finish(1) {
            error!("best-effort ROG hotkey cleanup failed: {error:#}");
        }
    }
}

pub fn handle_after_monitor_spawn(
    restorer: Option<Box<dyn FirmwareRestorer>>,
    shutdown: Arc<AtomicBool>,
    status: RogHotkeyStatusStore,
    monitor_thread: std::io::Result<JoinHandle<()>>,
) -> Result<RogHotkeyHandle> {
    match monitor_thread {
        Ok(monitor_thread) => Ok(RogHotkeyHandle {
            restorer,
            shutdown,
            monitor_thread: Some(monitor_thread),
            status,
        }),
        Err(error) => {
            shutdown.store(true, Ordering::Release);
            Err(anyhow!("failed to spawn ROG input monitor: {error}"))
        }
    }
}

fn wait_for_shutdown(shutdown: &AtomicBool, duration: Duration) -> bool {
    let steps = duration.as_millis().div_ceil(100);
    for _ in 0..steps {
        if shutdown.load(Ordering::Acquire) {
            return true;
        }
        thread::sleep(Duration::from_millis(100));
    }
    shutdown.load(Ordering::Acquire)
}

fn run_monitor_supervisor(
    shutdown: Arc<AtomicBool>,
    sender: Sender<()>,
    status: RogHotkeyStatusStore,
) {
    let mut backoff = Duration::ZERO;
    let dropped = status.dropped_counter();

    while !shutdown.load(Ordering::Acquire) {
        status.set_monitor_state("starting");
        let opened = Arc::new(AtomicBool::new(false));
        let opened_for_callback = opened.clone();
        let status_for_callback = status.clone();
        let result = monitor_once(
            ROG_INPUT_PATH,
            sender.clone(),
            shutdown.clone(),
            dropped.clone(),
            move || {
                opened_for_callback.store(true, Ordering::Release);
                status_for_callback.set_monitor_state("running");
                status_for_callback.set_last_error(None);
            },
        );

        if shutdown.load(Ordering::Acquire) {
            break;
        }
        if result.is_ok() {
            break;
        }

        if opened.load(Ordering::Acquire) {
            backoff = Duration::ZERO;
        }
        let error = result.unwrap_err();
        if receiver_disconnected(&error) {
            error!("ROG input monitor stopped because the D-Bus consumer is gone: {error:#}");
            status.set_last_error(Some(format!("{error:#}")));
            break;
        }
        let next = next_backoff(backoff);
        if next != backoff {
            warn!(
                "ROG input monitor failed; retrying in {} seconds: {error:#}",
                next.as_secs()
            );
        } else {
            debug!("ROG input monitor still unavailable at capped backoff: {error:#}");
        }
        status.set_last_error(Some(format!("{error:#}")));
        status.set_monitor_state("backoff");
        backoff = next;

        if wait_for_shutdown(&shutdown, backoff) {
            break;
        }
    }

    status.set_monitor_state("stopped");
}

pub fn start() -> Result<(RogHotkeyHandle, RogHotkeyStatusStore, Receiver<()>)> {
    let fallback_requested = fallback_enabled(std::env::var(FALLBACK_ENV).ok().as_deref());
    let shutdown = Arc::new(AtomicBool::new(false));
    let (sender, receiver) = mpsc::channel(1);

    let (restorer, status) = match HidMKeyTransport::discover_and_probe() {
        Ok((transport, layout)) => {
            let identity = transport.identity().to_string();
            let guard = require_prepared(
                FirmwareBindingGuard::prepare(transport, &layout),
                fallback_requested,
            )?;
            let firmware_changed = guard.changed();
            (
                Some(Box::new(guard) as Box<dyn FirmwareRestorer>),
                RogHotkeyStatus::supported(identity, firmware_changed),
            )
        }
        Err(error) => {
            let mut status = RogHotkeyStatus::unsupported(format!(
                "failed to discover and probe ROG hardware: {error:#}"
            ));
            status.fallback_enabled = fallback_requested;
            if fallback_requested {
                status.monitor_state = "starting".to_string();
            }
            (None, status)
        }
    };

    let policy = monitor_policy(status.hardware_supported, fallback_requested);
    let status = RogHotkeyStatusStore::new(status);
    let handle = if policy.start_monitor {
        let thread_shutdown = shutdown.clone();
        let thread_status = status.clone();
        let monitor_thread = thread::Builder::new()
            .name("rog-hotkey-monitor".to_string())
            .spawn(move || run_monitor_supervisor(thread_shutdown, sender, thread_status));
        handle_after_monitor_spawn(restorer, shutdown, status.clone(), monitor_thread)?
    } else {
        drop(sender);
        RogHotkeyHandle {
            restorer,
            shutdown,
            monitor_thread: None,
            status: status.clone(),
        }
    };

    Ok((handle, status, receiver))
}
