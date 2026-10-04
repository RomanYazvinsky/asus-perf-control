#![allow(dead_code)]

#[path = "../src/rog_input.rs"]
mod rog_input;

use rog_input::{
    find_native_rog_keyboard, monitor_once, receiver_disconnected, try_emit_press,
    wait_for_input_or_cancel, RawInputEvent, RogEventMatcher, RogInputSource, WaitOutcome, EV_KEY,
    EV_MSC, EV_SYN, KEY_PROG1, KEY_UNKNOWN, MSC_SCAN, NATIVE_ROG_SCAN, ROG_INPUT_PATH, ROG_SCAN,
    SYN_REPORT,
};
use std::os::fd::AsRawFd;
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};
use std::sync::{
    atomic::{AtomicBool, AtomicU64, Ordering},
    Arc,
};
use std::time::{Duration, Instant};
use tokio::sync::mpsc::Sender;

fn matcher() -> RogEventMatcher {
    RogEventMatcher::new(Duration::from_millis(300))
}

#[test]
fn direct_key_prog1_press_is_accepted() {
    let mut matcher = matcher();

    assert!(matcher.accept(RawInputEvent::new(EV_KEY, KEY_PROG1, 1), Instant::now()));
}

#[test]
fn rog_scan_followed_by_key_unknown_press_is_accepted() {
    let now = Instant::now();
    let mut matcher = matcher();

    assert!(!matcher.accept(RawInputEvent::new(EV_MSC, MSC_SCAN, ROG_SCAN), now));
    assert!(matcher.accept(RawInputEvent::new(EV_KEY, KEY_UNKNOWN, 1), now));
}

#[test]
fn rog_scan_followed_by_model_specific_key_press_is_accepted() {
    let now = Instant::now();
    let mut matcher = matcher();

    assert!(!matcher.accept(RawInputEvent::new(EV_MSC, MSC_SCAN, ROG_SCAN), now));
    assert!(matcher.accept(RawInputEvent::new(EV_KEY, 0x2ff, 1), now));
}

#[test]
fn key_release_after_rog_scan_is_ignored_and_consumes_scan() {
    let now = Instant::now();
    let mut matcher = matcher();

    assert!(!matcher.accept(RawInputEvent::new(EV_MSC, MSC_SCAN, ROG_SCAN), now));
    assert!(!matcher.accept(RawInputEvent::new(EV_KEY, KEY_UNKNOWN, 0), now));
    assert!(!matcher.accept(RawInputEvent::new(EV_KEY, KEY_UNKNOWN, 1), now));
}

#[test]
fn syn_report_clears_pending_scan() {
    let now = Instant::now();
    let mut matcher = matcher();

    assert!(!matcher.accept(RawInputEvent::new(EV_MSC, MSC_SCAN, ROG_SCAN), now));
    assert!(!matcher.accept(RawInputEvent::new(EV_SYN, SYN_REPORT, 0), now));
    assert!(!matcher.accept(RawInputEvent::new(EV_KEY, KEY_UNKNOWN, 1), now));
}

#[test]
fn a_different_scan_code_replaces_stale_rog_scan_state() {
    let now = Instant::now();
    let mut matcher = matcher();

    assert!(!matcher.accept(RawInputEvent::new(EV_MSC, MSC_SCAN, ROG_SCAN), now));
    assert!(!matcher.accept(RawInputEvent::new(EV_MSC, MSC_SCAN, ROG_SCAN + 1), now));
    assert!(!matcher.accept(RawInputEvent::new(EV_KEY, KEY_UNKNOWN, 1), now));
}

#[test]
fn key_repeat_is_ignored() {
    let mut matcher = matcher();

    assert!(!matcher.accept(RawInputEvent::new(EV_KEY, KEY_PROG1, 2), Instant::now()));
}

#[test]
fn direct_presses_are_debounced_from_last_accepted_press() {
    let now = Instant::now();
    let mut matcher = matcher();

    assert!(matcher.accept(RawInputEvent::new(EV_KEY, KEY_PROG1, 1), now));
    assert!(!matcher.accept(
        RawInputEvent::new(EV_KEY, KEY_PROG1, 1),
        now + Duration::from_millis(50)
    ));
    assert!(matcher.accept(
        RawInputEvent::new(EV_KEY, KEY_PROG1, 1),
        now + Duration::from_millis(301)
    ));
}

#[test]
fn press_at_exact_debounce_boundary_is_accepted() {
    let now = Instant::now();
    let mut matcher = matcher();

    assert!(matcher.accept(RawInputEvent::new(EV_KEY, KEY_PROG1, 1), now));
    assert!(matcher.accept(
        RawInputEvent::new(EV_KEY, KEY_PROG1, 1),
        now + Duration::from_millis(300)
    ));
}

#[test]
fn reversed_instant_is_accepted_without_panicking() {
    let now = Instant::now();
    let earlier = now.checked_sub(Duration::from_millis(1)).unwrap();
    let mut matcher = matcher();

    assert!(matcher.accept(RawInputEvent::new(EV_KEY, KEY_PROG1, 1), now));
    assert!(matcher.accept(RawInputEvent::new(EV_KEY, KEY_PROG1, 1), earlier));
}

#[test]
fn unrelated_key_press_is_ignored() {
    let mut matcher = matcher();

    assert!(!matcher.accept(RawInputEvent::new(EV_KEY, 30, 1), Instant::now()));
}

#[test]
fn stable_input_path_is_exposed() {
    assert_eq!(
        ROG_INPUT_PATH,
        "/dev/input/by-path/platform-asus-nb-wmi-event"
    );
}

#[test]
fn monitor_never_grabs_the_input_device() {
    let source = include_str!("../src/rog_input.rs");

    assert!(!source.contains(".grab("));
}

#[test]
fn monitor_is_nonblocking_and_checks_shutdown_frequently() {
    let source = include_str!("../src/rog_input.rs");

    assert!(source.contains("set_nonblocking(true)"));
    assert!(source.contains("wait_for_input_or_cancel"));
    assert!(!source.contains("Duration::from_millis(50)"));
}

#[test]
fn cancellation_fd_wakes_blocking_wait() {
    use std::io::Write;
    let (input_read, _input_write) = UnixStream::pair().unwrap();
    let (cancel_read, mut cancel_write) = UnixStream::pair().unwrap();
    let waiter = std::thread::spawn(move || {
        wait_for_input_or_cancel(input_read.as_raw_fd(), cancel_read.as_raw_fd()).unwrap()
    });

    cancel_write.write_all(&[1]).unwrap();

    assert_eq!(waiter.join().unwrap(), WaitOutcome::Cancelled);
}

#[test]
fn full_queue_coalesces_press_and_counts_drop() {
    let (sender, mut receiver) = tokio::sync::mpsc::channel(1);
    let dropped = AtomicU64::new(0);

    try_emit_press(&sender, &dropped).unwrap();
    try_emit_press(&sender, &dropped).unwrap();

    assert_eq!(dropped.load(Ordering::Relaxed), 1);
    assert_eq!(receiver.try_recv(), Ok(()));
    assert!(receiver.try_recv().is_err());
}

#[test]
fn disconnected_press_receiver_is_an_error() {
    let (sender, receiver) = tokio::sync::mpsc::channel(1);
    let dropped = AtomicU64::new(0);
    drop(receiver);

    let error = try_emit_press(&sender, &dropped).unwrap_err();

    assert!(error.to_string().contains("receiver is gone"));
    assert!(receiver_disconnected(&error));
    assert_eq!(dropped.load(Ordering::Relaxed), 0);
}

#[allow(dead_code)]
fn monitor_once_accepts_interruptible_bounded_contract(
    path: &Path,
    sender: Sender<()>,
    shutdown: Arc<AtomicBool>,
    dropped: Arc<AtomicU64>,
    cancel: &UnixStream,
) {
    let _ = monitor_once(path, sender, shutdown, dropped, cancel, || {});
}

fn keyboard_matcher() -> RogEventMatcher {
    RogEventMatcher::for_source(RogInputSource::AsusHidKeyboard, Duration::from_millis(300))
}

#[test]
fn native_scan_value_is_the_asus_vendor_usage() {
    // HID usage page 0xff31, usage 0x38, as the kernel reports it in a signed MSC_SCAN value.
    assert_eq!(NATIVE_ROG_SCAN as u32, 0xff31_0038);
}

#[test]
fn keyboard_rog_scan_followed_by_key_prog1_press_is_accepted() {
    let now = Instant::now();
    let mut matcher = keyboard_matcher();

    assert!(!matcher.accept(RawInputEvent::new(EV_MSC, MSC_SCAN, NATIVE_ROG_SCAN), now));
    assert!(matcher.accept(RawInputEvent::new(EV_KEY, KEY_PROG1, 1), now));
}

#[test]
fn keyboard_key_prog1_without_rog_scan_is_ignored() {
    let mut matcher = keyboard_matcher();

    assert!(!matcher.accept(RawInputEvent::new(EV_KEY, KEY_PROG1, 1), Instant::now()));
}

#[test]
fn keyboard_rog_scan_followed_by_another_key_is_ignored() {
    let now = Instant::now();
    let mut matcher = keyboard_matcher();

    assert!(!matcher.accept(RawInputEvent::new(EV_MSC, MSC_SCAN, NATIVE_ROG_SCAN), now));
    assert!(!matcher.accept(RawInputEvent::new(EV_KEY, 30, 1), now));
}

#[test]
fn keyboard_ignores_the_wmi_scan_code() {
    let now = Instant::now();
    let mut matcher = keyboard_matcher();

    assert!(!matcher.accept(RawInputEvent::new(EV_MSC, MSC_SCAN, ROG_SCAN), now));
    assert!(!matcher.accept(RawInputEvent::new(EV_KEY, KEY_UNKNOWN, 1), now));
    assert!(!matcher.accept(RawInputEvent::new(EV_MSC, MSC_SCAN, ROG_SCAN), now));
    assert!(!matcher.accept(RawInputEvent::new(EV_KEY, KEY_PROG1, 1), now));
}

#[test]
fn keyboard_release_and_repeat_are_ignored_and_presses_debounced() {
    let now = Instant::now();
    let mut matcher = keyboard_matcher();
    let scan = RawInputEvent::new(EV_MSC, MSC_SCAN, NATIVE_ROG_SCAN);

    assert!(!matcher.accept(scan, now));
    assert!(!matcher.accept(RawInputEvent::new(EV_KEY, KEY_PROG1, 0), now));
    assert!(!matcher.accept(scan, now));
    assert!(!matcher.accept(RawInputEvent::new(EV_KEY, KEY_PROG1, 2), now));
    assert!(!matcher.accept(scan, now));
    assert!(matcher.accept(RawInputEvent::new(EV_KEY, KEY_PROG1, 1), now));
    assert!(!matcher.accept(scan, now + Duration::from_millis(50)));
    assert!(!matcher.accept(
        RawInputEvent::new(EV_KEY, KEY_PROG1, 1),
        now + Duration::from_millis(50)
    ));
}

/// Fake /sys/class/input tree; `key` is the capabilities/key bitmap as the kernel prints it.
struct FakeInput {
    root: PathBuf,
}

impl FakeInput {
    fn new(name: &str) -> Self {
        let root = std::env::temp_dir().join(format!(
            "asus-control-daemon-rog-input-{name}-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();
        Self { root }
    }

    fn device(&self, event: &str, vendor: &str, bustype: &str, key: &str) -> &Self {
        let device = self.root.join(event).join("device");
        std::fs::create_dir_all(device.join("id")).unwrap();
        std::fs::create_dir_all(device.join("capabilities")).unwrap();
        std::fs::write(device.join("id/vendor"), format!("{vendor}\n")).unwrap();
        std::fs::write(device.join("id/bustype"), format!("{bustype}\n")).unwrap();
        std::fs::write(device.join("capabilities/key"), format!("{key}\n")).unwrap();
        self
    }

    fn find(&self) -> anyhow::Result<PathBuf> {
        find_native_rog_keyboard(&self.root, Path::new("/dev/input"))
    }
}

impl Drop for FakeInput {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

// Bitmaps captured from a ROG Flow X13 GV301QH: event5 reports KEY_PROG1 (bit 148), event7 not.
const GV301_KEYBOARD_KEYS: &str = "733eff 0 20000 483ffff17aff32d bfd4444600000000 1 1b30c730b17cc00 9fa67bfad9715fed 9e168000004400 10000002";
const GV301_CONSUMER_KEYS: &str =
    "1000000000007 ff9f207ac14057ff febeffdfffefffff fffffffffffffffe";

#[test]
fn native_keyboard_discovery_picks_the_asus_usb_device_with_key_prog1() {
    let input = FakeInput::new("gv301");
    input
        .device("event4", "0000", "0019", GV301_KEYBOARD_KEYS)
        .device("event5", "0b05", "0003", GV301_KEYBOARD_KEYS)
        .device("event7", "0b05", "0003", GV301_CONSUMER_KEYS)
        .device("event12", "046d", "0003", GV301_KEYBOARD_KEYS);

    assert_eq!(input.find().unwrap(), PathBuf::from("/dev/input/event5"));
}

#[test]
fn native_keyboard_discovery_prefers_the_lowest_event_number() {
    let input = FakeInput::new("order");
    input
        .device("event11", "0b05", "0003", GV301_KEYBOARD_KEYS)
        .device("event9", "0b05", "0003", GV301_KEYBOARD_KEYS);

    assert_eq!(input.find().unwrap(), PathBuf::from("/dev/input/event9"));
}

#[test]
fn native_keyboard_discovery_fails_without_a_matching_device() {
    let input = FakeInput::new("none");
    input
        .device("event4", "0000", "0019", GV301_KEYBOARD_KEYS)
        .device("event7", "0b05", "0003", GV301_CONSUMER_KEYS);

    let error = input.find().unwrap_err().to_string();
    assert!(error.contains("KEY_PROG1"), "{error}");
}
