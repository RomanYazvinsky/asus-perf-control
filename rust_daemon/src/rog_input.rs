use anyhow::{Context, Result};
use std::fmt;
use std::os::fd::{AsRawFd, RawFd};
use std::os::unix::net::UnixStream;
use std::path::Path;
use std::sync::{
    atomic::{AtomicBool, AtomicU64, Ordering},
    Arc,
};
use std::time::{Duration, Instant};
use tokio::sync::mpsc::{error::TrySendError, Sender};

pub const EV_SYN: u16 = 0x00;
pub const EV_KEY: u16 = 0x01;
pub const EV_MSC: u16 = 0x04;
pub const SYN_REPORT: u16 = 0x00;
pub const MSC_SCAN: u16 = 0x04;
pub const KEY_PROG1: u16 = 148;
pub const KEY_UNKNOWN: u16 = 240;
pub const ROG_SCAN: i32 = 0x38;
pub const ROG_INPUT_PATH: &str = "/dev/input/by-path/platform-asus-nb-wmi-event";

#[derive(Debug)]
struct ReceiverDisconnected;

impl fmt::Display for ReceiverDisconnected {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("failed to send ROG press: receiver is gone")
    }
}

impl std::error::Error for ReceiverDisconnected {}

pub fn receiver_disconnected(error: &anyhow::Error) -> bool {
    error.downcast_ref::<ReceiverDisconnected>().is_some()
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum WaitOutcome {
    InputReady,
    Cancelled,
}

pub fn wait_for_input_or_cancel(input_fd: RawFd, cancel_fd: RawFd) -> Result<WaitOutcome> {
    let mut descriptors = [
        nix::libc::pollfd {
            fd: input_fd,
            events: nix::libc::POLLIN,
            revents: 0,
        },
        nix::libc::pollfd {
            fd: cancel_fd,
            events: nix::libc::POLLIN,
            revents: 0,
        },
    ];

    loop {
        // SAFETY: descriptors points to two initialized pollfd values for the duration of poll.
        let result =
            unsafe { nix::libc::poll(descriptors.as_mut_ptr(), descriptors.len() as _, -1) };
        if result < 0 {
            let error = std::io::Error::last_os_error();
            if error.kind() == std::io::ErrorKind::Interrupted {
                continue;
            }
            return Err(error).context("failed to poll ROG input and cancellation fds");
        }

        if descriptors[1].revents
            & (nix::libc::POLLIN | nix::libc::POLLERR | nix::libc::POLLHUP | nix::libc::POLLNVAL)
            != 0
        {
            return Ok(WaitOutcome::Cancelled);
        }
        if descriptors[0].revents
            & (nix::libc::POLLIN | nix::libc::POLLERR | nix::libc::POLLHUP | nix::libc::POLLNVAL)
            != 0
        {
            return Ok(WaitOutcome::InputReady);
        }
    }
}

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

pub fn try_emit_press(sender: &Sender<()>, dropped: &AtomicU64) -> Result<()> {
    match sender.try_send(()) {
        Ok(()) => Ok(()),
        Err(TrySendError::Full(())) => {
            dropped.fetch_add(1, Ordering::Relaxed);
            Ok(())
        }
        Err(TrySendError::Closed(())) => Err(ReceiverDisconnected.into()),
    }
}

pub fn open_input_device(path: impl AsRef<Path>) -> Result<evdev::Device> {
    let path = path.as_ref();
    let device = evdev::Device::open(path)
        .with_context(|| format!("failed to open ROG input device {}", path.display()))?;
    device.set_nonblocking(true).with_context(|| {
        format!(
            "failed to make ROG input device {} nonblocking",
            path.display()
        )
    })?;
    Ok(device)
}

pub fn monitor_device_once<F>(
    mut device: evdev::Device,
    path: impl AsRef<Path>,
    sender: Sender<()>,
    shutdown: Arc<AtomicBool>,
    dropped: Arc<AtomicU64>,
    cancel: &UnixStream,
    on_open: F,
) -> Result<()>
where
    F: FnOnce(),
{
    let path = path.as_ref();
    on_open();
    let mut matcher = RogEventMatcher::new(Duration::from_millis(300));

    while !shutdown.load(Ordering::Acquire) {
        if wait_for_input_or_cancel(device.as_raw_fd(), cancel.as_raw_fd())?
            == WaitOutcome::Cancelled
        {
            return Ok(());
        }
        let events = match device.fetch_events() {
            Ok(events) => events,
            Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => continue,
            Err(error) => {
                return Err(error).with_context(|| {
                    format!("failed to read ROG input device {}", path.display())
                });
            }
        };

        for event in events {
            if matcher.accept(event.into(), Instant::now()) {
                try_emit_press(&sender, &dropped).with_context(|| {
                    format!(
                        "failed to send ROG press from input device {}",
                        path.display()
                    )
                })?;
            }
        }
    }

    Ok(())
}

/// Opens and monitors one input device instance without grabbing it.
///
/// Nonblocking reads keep shutdown latency bounded. The supervisor owns retry/backoff.
#[allow(dead_code)]
pub fn monitor_once<F>(
    path: impl AsRef<Path>,
    sender: Sender<()>,
    shutdown: Arc<AtomicBool>,
    dropped: Arc<AtomicU64>,
    cancel: &UnixStream,
    on_open: F,
) -> Result<()>
where
    F: FnOnce(),
{
    let path = path.as_ref();
    let device = open_input_device(path)?;
    monitor_device_once(device, path, sender, shutdown, dropped, cancel, on_open)
}
