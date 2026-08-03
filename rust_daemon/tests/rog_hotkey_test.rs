#![allow(dead_code)]

#[path = "../src/hardware/mod.rs"]
mod hardware;
#[path = "../src/rog_hotkey.rs"]
mod rog_hotkey;
#[path = "../src/rog_input.rs"]
mod rog_input;

use rog_hotkey::{fallback_enabled, monitor_policy, RogHotkeyStatus};

#[test]
fn unsupported_status_has_a_stable_json_schema() {
    let status = RogHotkeyStatus::unsupported("probe failed");

    assert_eq!(
        status.to_json().unwrap(),
        r#"{"schemaVersion":1,"hardwareSupported":false,"firmwareChanged":false,"fallbackEnabled":false,"inputPath":"/dev/input/by-path/platform-asus-nb-wmi-event","scanCode":56,"keyCode":148,"hardwareIdentity":null,"reason":"probe failed"}"#
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
