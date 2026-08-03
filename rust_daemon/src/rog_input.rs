use anyhow::{Context, Result};
use std::path::Path;
use std::time::{Duration, Instant};

pub const EV_SYN: u16 = 0x00;
pub const EV_KEY: u16 = 0x01;
pub const EV_MSC: u16 = 0x04;
pub const SYN_REPORT: u16 = 0x00;
pub const MSC_SCAN: u16 = 0x04;
pub const KEY_PROG1: u16 = 148;
pub const KEY_UNKNOWN: u16 = 240;
pub const ROG_SCAN: i32 = 0x38;
pub const ROG_INPUT_PATH: &str = "/dev/input/by-path/platform-asus-nb-wmi-event";

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct RawInputEvent {
    pub kind: u16,
    pub code: u16,
    pub value: i32,
}

impl RawInputEvent {
    pub const fn new(kind: u16, code: u16, value: i32) -> Self {
        Self { kind, code, value }
    }
}

impl From<evdev::InputEvent> for RawInputEvent {
    fn from(event: evdev::InputEvent) -> Self {
        Self::new(event.event_type().0, event.code(), event.value())
    }
}

pub struct RogEventMatcher {
    debounce: Duration,
    last_press: Option<Instant>,
    pending_scan: bool,
}

impl RogEventMatcher {
    pub fn new(debounce: Duration) -> Self {
        Self {
            debounce,
            last_press: None,
            pending_scan: false,
        }
    }

    pub fn accept(&mut self, event: RawInputEvent, now: Instant) -> bool {
        if event.kind == EV_MSC && event.code == MSC_SCAN {
            self.pending_scan = event.value == ROG_SCAN;
            return false;
        }

        if event.kind == EV_SYN && event.code == SYN_REPORT {
            self.pending_scan = false;
            return false;
        }

        if event.kind != EV_KEY {
            return false;
        }

        let follows_rog_scan = std::mem::take(&mut self.pending_scan);
        let is_candidate = event.value == 1 && (follows_rog_scan || event.code == KEY_PROG1);
        if !is_candidate {
            return false;
        }

        if self
            .last_press
            .and_then(|last_press| now.checked_duration_since(last_press))
            .is_some_and(|elapsed| elapsed < self.debounce)
        {
            return false;
        }

        self.last_press = Some(now);
        true
    }
}

/// Opens and monitors one input device instance.
///
/// This returns on open, read, or channel failure. The caller or supervisor owns retry/backoff
/// and process shutdown.
pub fn monitor_once(
    path: impl AsRef<Path>,
    sender: tokio::sync::mpsc::UnboundedSender<()>,
) -> Result<()> {
    let path = path.as_ref();
    let mut device = evdev::Device::open(path)
        .with_context(|| format!("failed to open ROG input device {}", path.display()))?;
    let mut matcher = RogEventMatcher::new(Duration::from_millis(300));

    loop {
        let events = device
            .fetch_events()
            .with_context(|| format!("failed to read ROG input device {}", path.display()))?;

        for event in events {
            if matcher.accept(event.into(), Instant::now()) {
                sender.send(()).with_context(|| {
                    format!(
                        "failed to send ROG press from input device {}: receiver is gone",
                        path.display()
                    )
                })?;
            }
        }
    }
}
