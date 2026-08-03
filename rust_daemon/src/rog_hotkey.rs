use crate::hardware::rog_mkey::{FirmwareBindingGuard, HidMKeyTransport};
use crate::rog_input::{monitor_once, KEY_PROG1, ROG_INPUT_PATH, ROG_SCAN};
use anyhow::Result;
use log::error;
use serde::Serialize;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::thread;
use std::time::Duration;
use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};

const FALLBACK_ENV: &str = "ASUS_ROG_HOTKEY_SOFTWARE_FALLBACK";

#[derive(Debug, Serialize)]
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
        }
    }

    pub fn to_json(&self) -> serde_json::Result<String> {
        serde_json::to_string(self)
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

pub struct RogHotkeyHandle {
    guard: Option<FirmwareBindingGuard<HidMKeyTransport>>,
    shutdown: Arc<AtomicBool>,
}

impl RogHotkeyHandle {
    pub fn shutdown(&mut self) -> Result<()> {
        self.shutdown.store(true, Ordering::Release);
        if let Some(guard) = self.guard.as_mut() {
            guard.restore()?;
        }
        Ok(())
    }
}

impl Drop for RogHotkeyHandle {
    fn drop(&mut self) {
        self.shutdown.store(true, Ordering::Release);
    }
}

fn spawn_monitor_supervisor(shutdown: Arc<AtomicBool>, sender: UnboundedSender<()>) -> Result<()> {
    thread::Builder::new()
        .name("rog-hotkey-monitor".to_string())
        .spawn(move || {
            while !shutdown.load(Ordering::Acquire) {
                if let Err(error) = monitor_once(ROG_INPUT_PATH, sender.clone()) {
                    error!("ROG input monitor stopped: {error:#}");
                }

                for _ in 0..10 {
                    if shutdown.load(Ordering::Acquire) {
                        return;
                    }
                    thread::sleep(Duration::from_millis(100));
                }
            }
        })?;
    Ok(())
}

pub fn start() -> Result<(RogHotkeyHandle, RogHotkeyStatus, UnboundedReceiver<()>)> {
    let (sender, receiver) = mpsc::unbounded_channel();
    let shutdown = Arc::new(AtomicBool::new(false));
    let fallback_requested = fallback_enabled(std::env::var(FALLBACK_ENV).ok().as_deref());

    let (guard, status) = match HidMKeyTransport::discover_and_probe() {
        Ok((transport, layout)) => {
            let identity = transport.identity().to_string();
            match FirmwareBindingGuard::prepare(transport, &layout) {
                Ok(guard) => {
                    let firmware_changed = guard.changed();
                    (
                        Some(guard),
                        RogHotkeyStatus {
                            schema_version: 1,
                            hardware_supported: true,
                            firmware_changed,
                            fallback_enabled: false,
                            input_path: ROG_INPUT_PATH.to_string(),
                            scan_code: ROG_SCAN as u16,
                            key_code: KEY_PROG1,
                            hardware_identity: Some(identity),
                            reason: None,
                        },
                    )
                }
                Err(error) => {
                    let mut status = RogHotkeyStatus::unsupported(format!(
                        "failed to prepare ROG firmware binding: {error:#}"
                    ));
                    status.fallback_enabled = fallback_requested;
                    (None, status)
                }
            }
        }
        Err(error) => {
            let mut status = RogHotkeyStatus::unsupported(format!(
                "failed to discover and probe ROG hardware: {error:#}"
            ));
            status.fallback_enabled = fallback_requested;
            (None, status)
        }
    };

    let policy = monitor_policy(status.hardware_supported, fallback_requested);
    if policy.start_monitor {
        spawn_monitor_supervisor(shutdown.clone(), sender)?;
    }

    Ok((RogHotkeyHandle { guard, shutdown }, status, receiver))
}
