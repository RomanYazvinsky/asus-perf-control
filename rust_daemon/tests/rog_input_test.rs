#![allow(dead_code)]

#[path = "../src/rog_input.rs"]
mod rog_input;

use rog_input::{
    monitor_once, receiver_disconnected, try_emit_press, RawInputEvent, RogEventMatcher, EV_KEY,
    EV_MSC, EV_SYN, KEY_PROG1, KEY_UNKNOWN, MSC_SCAN, ROG_INPUT_PATH, ROG_SCAN, SYN_REPORT,
};
use std::path::Path;
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
    assert!(source.contains("Duration::from_millis(50)"));
    assert!(source.contains("shutdown.load("));
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
) {
    let _ = monitor_once(path, sender, shutdown, dropped, || {});
}
