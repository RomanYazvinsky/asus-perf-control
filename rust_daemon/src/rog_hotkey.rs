use crate::hardware::rog_mkey::{FirmwareBindingGuard, HidMKeyTransport};
use crate::rog_input::{
    monitor_device_once, open_input_device, open_native_rog_keyboard, receiver_disconnected,
    RogInputSource, KEY_PROG1, ROG_INPUT_PATH, ROG_SCAN,
};
use anyhow::{anyhow, Result};
use log::{debug, error, info, warn};
use serde::Serialize;
use std::io::Write;
use std::os::unix::net::UnixStream;
use std::path::PathBuf;
use std::sync::{
    atomic::{AtomicBool, AtomicU64, Ordering},
    Arc, Mutex,
};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};
use tokio::sync::mpsc::{self, Receiver, Sender};

const FALLBACK_ENV: &str = "ASUS_ROG_HOTKEY_SOFTWARE_FALLBACK";
const ENABLED_ENV: &str = "ASUS_ROG_HOTKEY_ENABLED";
const RESTORE_ATTEMPTS: usize = 3;
const RESTORE_RETRY_DELAY: Duration = Duration::from_millis(100);
pub const HEALTHY_SESSION: Duration = Duration::from_secs(10);

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

    pub fn replace(&self, status: RogHotkeyStatus) {
        *self.lock() = status;
        self.dropped_presses.store(0, Ordering::Relaxed);
    }

    #[doc(hidden)]
    pub fn dropped_counter(&self) -> Arc<AtomicU64> {
        self.dropped_presses.clone()
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[allow(dead_code)]
pub struct MonitorPolicy {
    pub start_monitor: bool,
    pub fallback_enabled: bool,
}

pub fn fallback_enabled(value: Option<&str>) -> bool {
    value == Some("1")
}

pub fn hotkey_enabled(value: Option<&str>) -> bool {
    value == Some("1")
}

pub enum StartupResolution<H, D, P> {
    Unsupported {
        reason: String,
    },
    FallbackReady {
        device: D,
        reason: String,
    },
    /// No firmware M-key binding, but the keyboard reports the ROG key natively.
    NativeReady {
        device: D,
        reason: String,
    },
    InputUnavailable {
        hardware: Option<H>,
        reason: String,
        fallback_requested: bool,
    },
    SupportedReady {
        device: D,
        prepared: P,
    },
}

/// Startup order: firmware M-key discovery (then WMI input + binding); if the firmware has no
/// binding, the keyboard's native ROG key; only then the opt-in WMI software fallback.
pub fn resolve_runtime_startup<H, D, P, Discover, Open, OpenNative, Prepare>(
    fallback_requested: bool,
    discover: Discover,
    open: Open,
    open_native: OpenNative,
    prepare: Prepare,
) -> Result<StartupResolution<H, D, P>>
where
    Discover: FnOnce() -> Result<H>,
    Open: FnOnce() -> Result<D>,
    OpenNative: FnOnce() -> Result<D>,
    Prepare: FnOnce(H) -> Result<P>,
{
    match discover() {
        Ok(hardware) => match open() {
            Ok(device) => Ok(StartupResolution::SupportedReady {
                device,
                prepared: prepare(hardware)?,
            }),
            Err(error) => Ok(StartupResolution::InputUnavailable {
                hardware: Some(hardware),
                reason: format!("failed to pre-open ROG input device: {error:#}"),
                fallback_requested: false,
            }),
        },
        Err(discovery_error) => {
            let discovery =
                format!("failed to discover and probe ROG hardware: {discovery_error:#}");
            match open_native() {
                Ok(device) => Ok(StartupResolution::NativeReady {
                    device,
                    reason: format!(
                        "{discovery}; using the keyboard's native ROG key (KEY_PROG1), no firmware binding needed"
                    ),
                }),
                Err(native_error) if !fallback_requested => Ok(StartupResolution::Unsupported {
                    reason: format!("{discovery}; no native ROG key input: {native_error:#}"),
                }),
                Err(native_error) => match open() {
                    Ok(device) => Ok(StartupResolution::FallbackReady {
                        device,
                        reason: format!("{discovery}; no native ROG key input: {native_error:#}"),
                    }),
                    Err(input_error) => Ok(StartupResolution::InputUnavailable {
                        hardware: None,
                        reason: format!(
                            "{discovery}; no native ROG key input: {native_error:#}; failed to pre-open ROG input device: {input_error:#}"
                        ),
                        fallback_requested: true,
                    }),
                },
            }
        }
    }
}

pub fn take_initial_or_open<D, Open>(initial: &mut Option<D>, open: Open) -> Result<D>
where
    Open: FnOnce() -> Result<D>,
{
    match initial.take() {
        Some(device) => Ok(device),
        None => open(),
    }
}

#[allow(dead_code)]
pub fn monitor_policy(hardware_supported: bool, fallback_requested: bool) -> MonitorPolicy {
    MonitorPolicy {
        start_monitor: hardware_supported || fallback_requested,
        fallback_enabled: !hardware_supported && fallback_requested,
    }
}

pub fn require_prepared<T>(result: Result<T>) -> Result<T> {
    result.map_err(|error| {
        anyhow!(
            "failed to prepare ROG firmware binding; firmware state may be uncertain and daemon refuses to continue: {error:#}"
        )
    })
}

pub const fn retry_delay() -> Duration {
    Duration::from_secs(1)
}

pub const fn should_warn_failure(consecutive_failures: u64) -> bool {
    consecutive_failures.is_power_of_two()
}

pub fn next_failure_count(current: u64, opened_for: Option<Duration>) -> u64 {
    if opened_for.is_some_and(|duration| duration >= HEALTHY_SESSION) {
        1
    } else {
        current.saturating_add(1)
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
    cancellation: Option<UnixStream>,
    status: RogHotkeyStatusStore,
    cleanup_attempted: bool,
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
            cancellation: None,
            status,
            cleanup_attempted: false,
        }
    }

    #[cfg(test)]
    #[doc(hidden)]
    pub fn set_cancellation_for_test(&mut self, cancellation: UnixStream) {
        self.cancellation = Some(cancellation);
    }

    fn finish(&mut self, restore_attempts: usize) -> Result<()> {
        if self.cleanup_attempted {
            return Ok(());
        }
        self.shutdown.store(true, Ordering::Release);
        let mut failures = Vec::new();

        if let Some(cancellation) = self.cancellation.as_mut() {
            if let Err(error) = cancellation.write_all(&[1]) {
                let message = format!("failed to wake ROG input monitor: {error}");
                warn!("{message}");
                self.status.set_last_error(Some(message));
            }
        }

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
                            info!("restored original ROG firmware binding");
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
            self.cleanup_attempted = true;
            Ok(())
        } else {
            self.cleanup_attempted = true;
            Err(anyhow!(failures.join("; ")))
        }
    }

    pub fn shutdown(&mut self) -> Result<()> {
        self.finish(RESTORE_ATTEMPTS)
    }
}

impl Drop for RogHotkeyHandle {
    fn drop(&mut self) {
        if !self.cleanup_attempted {
            if let Err(error) = self.finish(1) {
                error!("best-effort ROG hotkey cleanup failed: {error:#}");
            }
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
            cancellation: None,
            status,
            cleanup_attempted: false,
        }),
        Err(error) => {
            let startup_error = anyhow!("failed to spawn ROG input monitor: {error}");
            let mut handle = RogHotkeyHandle {
                restorer,
                shutdown,
                monitor_thread: None,
                cancellation: None,
                status,
                cleanup_attempted: false,
            };
            match handle.finish(RESTORE_ATTEMPTS) {
                Ok(()) => Err(startup_error),
                Err(cleanup_error) => Err(anyhow!(
                    "{startup_error:#}; startup cleanup also failed: {cleanup_error:#}"
                )),
            }
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

fn open_source(source: RogInputSource) -> Result<(PathBuf, evdev::Device)> {
    match source {
        RogInputSource::AsusWmi => {
            open_input_device(ROG_INPUT_PATH).map(|device| (PathBuf::from(ROG_INPUT_PATH), device))
        }
        RogInputSource::AsusHidKeyboard => open_native_rog_keyboard(),
    }
}

fn run_monitor_supervisor(
    shutdown: Arc<AtomicBool>,
    sender: Sender<()>,
    cancellation: UnixStream,
    status: RogHotkeyStatusStore,
    source: RogInputSource,
    mut initial_device: Option<(PathBuf, evdev::Device)>,
) {
    let mut consecutive_failures = 0_u64;
    let dropped = status.dropped_counter();

    while !shutdown.load(Ordering::Acquire) {
        status.set_monitor_state("starting");
        let opened_at = Arc::new(Mutex::new(None));
        let opened_for_callback = opened_at.clone();
        let status_for_callback = status.clone();
        let on_open = move || {
            *opened_for_callback
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner()) = Some(Instant::now());
            status_for_callback.set_monitor_state("running");
            status_for_callback.set_last_error(None);
        };
        let result = take_initial_or_open(&mut initial_device, || open_source(source)).and_then(
            |(path, device)| {
                monitor_device_once(
                    device,
                    source,
                    path,
                    sender.clone(),
                    shutdown.clone(),
                    dropped.clone(),
                    &cancellation,
                    on_open,
                )
            },
        );

        if shutdown.load(Ordering::Acquire) {
            break;
        }
        if result.is_ok() {
            break;
        }

        let error = result.unwrap_err();
        if receiver_disconnected(&error) {
            error!("ROG input monitor stopped because the D-Bus consumer is gone: {error:#}");
            status.set_last_error(Some(format!("{error:#}")));
            break;
        }
        let opened_for = opened_at
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .map(|opened_at| opened_at.elapsed());
        consecutive_failures = next_failure_count(consecutive_failures, opened_for);
        if should_warn_failure(consecutive_failures) {
            warn!(
                "ROG input monitor failure #{consecutive_failures}; retrying in 1 second: {error:#}"
            );
        } else {
            debug!(
                "ROG input monitor failure #{consecutive_failures}; retrying in 1 second: {error:#}"
            );
        }
        status.set_last_error(Some(format!("{error:#}")));
        status.set_monitor_state("backoff");

        if wait_for_shutdown(&shutdown, retry_delay()) {
            break;
        }
    }

    status.set_monitor_state("stopped");
}

pub fn start(status_store: RogHotkeyStatusStore) -> Result<(RogHotkeyHandle, Receiver<()>)> {
    if !hotkey_enabled(std::env::var(ENABLED_ENV).ok().as_deref()) {
        status_store.replace(RogHotkeyStatus::unsupported(
            "ROG hotkey runtime disabled; set ASUS_ROG_HOTKEY_ENABLED=1 to enable it",
        ));
        let (sender, receiver) = mpsc::channel(1);
        drop(sender);
        return Ok((
            RogHotkeyHandle {
                restorer: None,
                shutdown: Arc::new(AtomicBool::new(false)),
                monitor_thread: None,
                cancellation: None,
                status: status_store,
                cleanup_attempted: false,
            },
            receiver,
        ));
    }

    let fallback_requested = fallback_enabled(std::env::var(FALLBACK_ENV).ok().as_deref());
    let shutdown = Arc::new(AtomicBool::new(false));
    let (sender, receiver) = mpsc::channel(1);
    let (cancellation_read, cancellation_write) = UnixStream::pair()?;

    let resolution = resolve_runtime_startup(
        fallback_requested,
        || {
            HidMKeyTransport::discover_and_probe().map(|(transport, layout)| {
                let identity = transport.identity().to_string();
                (transport, layout, identity)
            })
        },
        || open_input_device(ROG_INPUT_PATH).map(|device| (PathBuf::from(ROG_INPUT_PATH), device)),
        open_native_rog_keyboard,
        |(transport, layout, identity)| {
            let guard = require_prepared(FirmwareBindingGuard::prepare(transport, &layout))?;
            let firmware_changed = guard.changed();
            Ok((
                Box::new(guard) as Box<dyn FirmwareRestorer>,
                identity,
                firmware_changed,
            ))
        },
    )?;

    let mut source = RogInputSource::AsusWmi;
    let (initial_device, restorer, status) = match resolution {
        StartupResolution::Unsupported { reason } => {
            (None, None, RogHotkeyStatus::unsupported(reason))
        }
        StartupResolution::FallbackReady { device, reason } => {
            let mut status = RogHotkeyStatus::unsupported(reason);
            status.fallback_enabled = true;
            status.monitor_state = "starting".to_string();
            (Some(device), None, status)
        }
        StartupResolution::NativeReady { device, reason } => {
            source = RogInputSource::AsusHidKeyboard;
            let path = device.0.display().to_string();
            info!(
                "ROG key: firmware has no M-key binding; listening for native KEY_PROG1 on {path}"
            );
            let mut status = RogHotkeyStatus::supported(
                format!("ASUS keyboard native KEY_PROG1 at {path}"),
                false,
            );
            status.input_path = path;
            status.reason = Some(reason);
            (Some(device), None, status)
        }
        StartupResolution::InputUnavailable {
            hardware,
            reason,
            fallback_requested,
        } => {
            let mut status = match hardware {
                Some((_, _, identity)) => RogHotkeyStatus::supported(identity, false),
                None => RogHotkeyStatus::unsupported(reason.clone()),
            };
            status.fallback_enabled = fallback_requested;
            status.monitor_state = "disabled".to_string();
            status.reason = Some(reason);
            (None, None, status)
        }
        StartupResolution::SupportedReady {
            device,
            prepared: (guard, identity, firmware_changed),
        } => (
            Some(device),
            Some(guard),
            RogHotkeyStatus::supported(identity, firmware_changed),
        ),
    };

    let start_monitor = initial_device.is_some();
    status_store.replace(status);
    let status = status_store;
    let handle = if start_monitor {
        let thread_shutdown = shutdown.clone();
        let thread_status = status.clone();
        let monitor_thread = thread::Builder::new()
            .name("rog-hotkey-monitor".to_string())
            .spawn(move || {
                run_monitor_supervisor(
                    thread_shutdown,
                    sender,
                    cancellation_read,
                    thread_status,
                    source,
                    initial_device,
                )
            });
        let mut handle =
            handle_after_monitor_spawn(restorer, shutdown, status.clone(), monitor_thread)?;
        handle.cancellation = Some(cancellation_write);
        handle
    } else {
        drop(sender);
        drop(cancellation_read);
        drop(cancellation_write);
        RogHotkeyHandle {
            restorer,
            shutdown,
            monitor_thread: None,
            cancellation: None,
            status: status.clone(),
            cleanup_attempted: false,
        }
    };

    Ok((handle, receiver))
}
