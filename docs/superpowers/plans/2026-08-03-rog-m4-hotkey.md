# ROG/M4 Hotkey Activation Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make the physical ASUS ROG/M4 key launch ASUS Linux Control when absent and raise its existing window when already running.

**Architecture:** The privileged Rust daemon probes and, only when needed, prepares the ASUS M-key firmware binding using the protocol documented by G-Helper, then observes the non-exclusive `asus-nb-wmi` input event and emits a system D-Bus signal. A small per-user Rust agent subscribes to that signal and invokes the unique GTK application through its desktop entry. Unsupported firmware remains untouched; an explicit environment flag enables the scan-code fallback for diagnostics.

**Tech Stack:** Rust 2021, `hidapi` 2.6, `evdev` 0.13, Tokio, zbus 4, GTK 3/GApplication, systemd system and user units, Flutter/Dart packaging tests.

---

## File structure

- `rust_daemon/src/hardware/rog_mkey.rs`: ASUS HID feature-report protocol, capability parsing, firmware guard, and mandatory G-Helper attribution.
- `rust_daemon/src/rog_input.rs`: non-exclusive `asus-nb-wmi` event matching and debounce.
- `rust_daemon/src/rog_hotkey.rs`: firmware/input orchestration and serializable diagnostics.
- `rust_daemon/src/hotkey_agent.rs`: system-bus signal subscription and GUI activation abstraction.
- `rust_daemon/src/bin/asus-control-hotkey-agent.rs`: user-agent executable entry point.
- `rust_daemon/src/main.rs`: D-Bus signal/status method and daemon lifecycle wiring.
- `rust_daemon/tests/rog_mkey_test.rs`: firmware protocol and restoration tests.
- `rust_daemon/tests/rog_input_test.rs`: scan/key matching and debounce tests.
- `rust_daemon/tests/rog_hotkey_test.rs`: fallback-policy and diagnostic-schema tests.
- `rust_daemon/tests/hotkey_agent_test.rs`: activation behavior without starting a GUI.
- `linux/runner/my_application.cc`: unique-instance activation and window presentation.
- `test/install/linux_activation_test.dart`: source-level regression test for GTK activation flags.
- `install/systemd/asus-control-hotkey-agent.service`: per-user listener lifecycle.
- `install/dbus/com.asuslinuxcontrol.Daemon.conf`: permission to receive the ROG signal and read status.
- `install/install.sh`, `packaging/arch/PKGBUILD`, `packaging/rpm/asus-linux-control.spec`: install the agent/unit and start it with graphical sessions.
- `test/install/dbus_config_test.dart`, `test/install/package_metadata_test.dart`: integration metadata coverage.
- `docs/research/rog-m4-hotkey-manual-verification.md`: hardware verification procedure and attribution.

### Task 1: Encode and test the G-Helper M-key protocol

**Files:**
- Create: `rust_daemon/src/hardware/rog_mkey.rs`
- Modify: `rust_daemon/src/hardware/mod.rs`
- Create: `rust_daemon/tests/rog_mkey_test.rs`

- [ ] **Step 1: Write failing protocol tests**

Create `rust_daemon/tests/rog_mkey_test.rs` with focused tests for a valid five-slot response, malformed counts, request construction, slot mapping, and attribution:

```rust
#![allow(dead_code)]

#[path = "../src/hardware/rog_mkey.rs"]
mod rog_mkey;

use rog_mkey::{parse_layout, probe_request, set_opcode_request, AURA_REPORT_ID, ROG_OPCODE};

#[test]
fn five_slot_layout_uses_last_slot_for_rog() {
    let response = [AURA_REPORT_ID, 0x9f, 0x02, 0x00, 5, 2, 3, 1, 14, 4];
    let layout = parse_layout(&response).unwrap();
    assert_eq!(layout.rog_slot(), Some(4));
    assert_eq!(layout.rog_opcode(), Some(4));
}

#[test]
fn malformed_layout_is_rejected_without_a_write() {
    assert!(parse_layout(&[AURA_REPORT_ID, 0x9f, 0x02, 0, 9]).is_err());
    assert!(parse_layout(&[AURA_REPORT_ID, 0x9f, 0x02, 0, 5, 1]).is_err());
}

#[test]
fn protocol_messages_match_g_helper_reports() {
    assert_eq!(&probe_request(8)[..4], &[AURA_REPORT_ID, 0x9f, 0x02, 0]);
    assert_eq!(
        &set_opcode_request(8, 4, ROG_OPCODE)[..6],
        &[AURA_REPORT_ID, 0x9f, 0x03, 0x01, 4, 4]
    );
}

#[test]
fn source_retains_g_helper_attribution() {
    let source = include_str!("../src/hardware/rog_mkey.rs");
    assert!(source.contains("https://github.com/seerge/g-helper/pull/5691"));
    assert!(source.contains("seerge"));
}
```

- [ ] **Step 2: Run the new test in a subagent and verify RED**

Run: `cargo test --manifest-path rust_daemon/Cargo.toml --test rog_mkey_test`

Expected: FAIL because `rust_daemon/src/hardware/rog_mkey.rs` does not exist.

- [ ] **Step 3: Implement the pure protocol module**

Create the module with the attribution immediately above the protocol constants:

```rust
use anyhow::{bail, Result};

/// ASUS M-key feature-report protocol adapted from G-Helper's
/// MKeyControl.cs by seerge:
/// https://github.com/seerge/g-helper/pull/5691
pub const AURA_REPORT_ID: u8 = 0x5d;
pub const ROG_OPCODE: u8 = 4;
const MAX_MKEY_SLOTS: usize = 8;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MKeyLayout {
    opcodes: Vec<u8>,
}

impl MKeyLayout {
    pub fn rog_slot(&self) -> Option<usize> {
        (self.opcodes.len() > 3).then(|| self.opcodes.len() - 1)
    }

    pub fn rog_opcode(&self) -> Option<u8> {
        self.rog_slot().map(|slot| self.opcodes[slot])
    }
}

pub fn probe_request(report_len: usize) -> Vec<u8> {
    let mut report = vec![0; report_len];
    if report_len >= 4 {
        report[..4].copy_from_slice(&[AURA_REPORT_ID, 0x9f, 0x02, 0x00]);
    }
    report
}

pub fn set_opcode_request(report_len: usize, slot: usize, opcode: u8) -> Vec<u8> {
    let mut report = vec![0; report_len];
    if report_len >= 6 {
        report[..6].copy_from_slice(&[
            AURA_REPORT_ID,
            0x9f,
            0x03,
            0x01,
            slot as u8,
            opcode,
        ]);
    }
    report
}

pub fn parse_layout(response: &[u8]) -> Result<MKeyLayout> {
    if response.len() < 5 || response[0..3] != [AURA_REPORT_ID, 0x9f, 0x02] {
        bail!("unexpected ASUS M-key response header");
    }
    let count = response[4] as usize;
    if !(1..=MAX_MKEY_SLOTS).contains(&count) || 5 + count > response.len() {
        bail!("invalid ASUS M-key slot count: {count}");
    }
    Ok(MKeyLayout {
        opcodes: response[5..5 + count].to_vec(),
    })
}
```

Export it with `pub mod rog_mkey;` in `rust_daemon/src/hardware/mod.rs`.

- [ ] **Step 4: Run the protocol test in a subagent and verify GREEN**

Run: `cargo test --manifest-path rust_daemon/Cargo.toml --test rog_mkey_test`

Expected: PASS, 4 tests.

- [ ] **Step 5: Commit only Task 1 files**

```bash
git add rust_daemon/src/hardware/mod.rs rust_daemon/src/hardware/rog_mkey.rs rust_daemon/tests/rog_mkey_test.rs
git commit -m "feat: define ASUS ROG M-key protocol"
```

### Task 2: Add safe HID discovery and firmware restoration

**Files:**
- Modify: `rust_daemon/Cargo.toml`
- Modify: `rust_daemon/src/hardware/rog_mkey.rs`
- Modify: `rust_daemon/tests/rog_mkey_test.rs`

- [ ] **Step 1: Add failing firmware-guard tests**

Add a fake transport that records reports, then prove that an already-correct ROG binding is not written, a changed binding writes opcode `4`, and shutdown restores the original opcode:

```rust
#[derive(Default)]
struct FakeTransport {
    writes: Vec<Vec<u8>>,
}

impl rog_mkey::MKeyTransport for FakeTransport {
    fn report_len(&self) -> usize { 8 }
    fn send_feature(&mut self, report: &[u8]) -> anyhow::Result<()> {
        self.writes.push(report.to_vec());
        Ok(())
    }
}

#[test]
fn firmware_guard_changes_only_rog_slot_and_restores_it() {
    let layout = parse_layout(&[AURA_REPORT_ID, 0x9f, 0x02, 0, 4, 2, 3, 1, 14]).unwrap();
    let mut transport = FakeTransport::default();
    let guard = rog_mkey::FirmwareBindingGuard::prepare(&mut transport, &layout).unwrap();
    assert_eq!(&transport.writes[0][..6], &[0x5d, 0x9f, 0x03, 1, 3, 4]);
    guard.restore(&mut transport).unwrap();
    assert_eq!(&transport.writes[1][..6], &[0x5d, 0x9f, 0x03, 1, 3, 14]);
}

#[test]
fn correct_rog_binding_does_not_write_firmware() {
    let layout = parse_layout(&[AURA_REPORT_ID, 0x9f, 0x02, 0, 4, 2, 3, 1, 4]).unwrap();
    let mut transport = FakeTransport::default();
    let guard = rog_mkey::FirmwareBindingGuard::prepare(&mut transport, &layout).unwrap();
    assert!(!guard.changed());
    assert!(transport.writes.is_empty());
}
```

- [ ] **Step 2: Run the guard tests in a subagent and verify RED**

Run: `cargo test --manifest-path rust_daemon/Cargo.toml --test rog_mkey_test firmware`

Expected: FAIL because `MKeyTransport` and `FirmwareBindingGuard` are absent.

- [ ] **Step 3: Add the dependency and minimal guard**

Add to `rust_daemon/Cargo.toml`:

```toml
hidapi = { version = "2.6", default-features = false, features = ["linux-static-hidraw"] }
```

Add to `rog_mkey.rs`:

```rust
pub trait MKeyTransport {
    fn report_len(&self) -> usize;
    fn send_feature(&mut self, report: &[u8]) -> Result<()>;
}

pub struct FirmwareBindingGuard {
    slot: usize,
    original_opcode: u8,
    changed: bool,
}

impl FirmwareBindingGuard {
    pub fn prepare<T: MKeyTransport>(transport: &mut T, layout: &MKeyLayout) -> Result<Self> {
        let slot = layout.rog_slot().ok_or_else(|| anyhow::anyhow!("ROG slot unavailable"))?;
        let original_opcode = layout.rog_opcode().unwrap();
        let changed = original_opcode != ROG_OPCODE;
        if changed {
            transport.send_feature(&set_opcode_request(transport.report_len(), slot, ROG_OPCODE))?;
        }
        Ok(Self { slot, original_opcode, changed })
    }

    pub fn changed(&self) -> bool { self.changed }

    pub fn restore<T: MKeyTransport>(&self, transport: &mut T) -> Result<()> {
        if self.changed {
            transport.send_feature(&set_opcode_request(
                transport.report_len(),
                self.slot,
                self.original_opcode,
            ))?;
        }
        Ok(())
    }
}
```

Implement `HidMKeyTransport::discover_and_probe()` with `hidapi::HidApi`: iterate ASUS vendor ID `0x0b05`, open each candidate, send `probe_request()`, poll `get_feature_report()` up to ten times with 20 ms delay, and accept only `parse_layout()` success. Do not write an opcode during discovery.

- [ ] **Step 4: Run all protocol/guard tests in a subagent**

Run: `cargo test --manifest-path rust_daemon/Cargo.toml --test rog_mkey_test`

Expected: PASS, including restoration and zero-write coverage.

- [ ] **Step 5: Commit Task 2**

```bash
git add rust_daemon/Cargo.toml rust_daemon/src/hardware/rog_mkey.rs rust_daemon/tests/rog_mkey_test.rs
git commit -m "feat: safely prepare ROG firmware binding"
```

### Task 3: Recognize the Linux ROG event without grabbing input

**Files:**
- Modify: `rust_daemon/Cargo.toml`
- Create: `rust_daemon/src/rog_input.rs`
- Create: `rust_daemon/tests/rog_input_test.rs`

- [ ] **Step 1: Write failing matcher tests**

```rust
#![allow(dead_code)]

#[path = "../src/rog_input.rs"]
mod rog_input;

use rog_input::{RawInputEvent, RogEventMatcher, EV_KEY, EV_MSC, KEY_PROG1, MSC_SCAN};
use std::time::{Duration, Instant};

#[test]
fn prog1_press_is_a_rog_press() {
    let now = Instant::now();
    let mut matcher = RogEventMatcher::new(Duration::from_millis(300));
    assert!(matcher.accept(RawInputEvent::new(EV_KEY, KEY_PROG1, 1), now));
}

#[test]
fn scan_38_followed_by_unknown_press_is_a_rog_press() {
    let now = Instant::now();
    let mut matcher = RogEventMatcher::new(Duration::from_millis(300));
    assert!(!matcher.accept(RawInputEvent::new(EV_MSC, MSC_SCAN, 0x38), now));
    assert!(matcher.accept(RawInputEvent::new(EV_KEY, 240, 1), now));
}

#[test]
fn repeated_press_inside_debounce_window_is_ignored() {
    let now = Instant::now();
    let mut matcher = RogEventMatcher::new(Duration::from_millis(300));
    assert!(matcher.accept(RawInputEvent::new(EV_KEY, KEY_PROG1, 1), now));
    assert!(!matcher.accept(
        RawInputEvent::new(EV_KEY, KEY_PROG1, 1),
        now + Duration::from_millis(50),
    ));
}
```

- [ ] **Step 2: Run matcher tests in a subagent and verify RED**

Run: `cargo test --manifest-path rust_daemon/Cargo.toml --test rog_input_test`

Expected: FAIL because `rog_input.rs` is absent.

- [ ] **Step 3: Implement the pure matcher and evdev adapter**

Add `evdev = "0.13"` to `rust_daemon/Cargo.toml`. Define the raw constants and matcher exactly once:

```rust
pub const EV_KEY: u16 = 0x01;
pub const EV_MSC: u16 = 0x04;
pub const MSC_SCAN: u16 = 0x04;
pub const KEY_PROG1: u16 = 148;
pub const ROG_SCAN: i32 = 0x38;

#[derive(Clone, Copy)]
pub struct RawInputEvent { pub kind: u16, pub code: u16, pub value: i32 }

impl RawInputEvent {
    pub const fn new(kind: u16, code: u16, value: i32) -> Self {
        Self { kind, code, value }
    }
}

pub struct RogEventMatcher {
    debounce: std::time::Duration,
    last_press: Option<std::time::Instant>,
    pending_scan: bool,
}
```

`accept()` sets `pending_scan` for `(EV_MSC, MSC_SCAN, 0x38)`, recognizes either `KEY_PROG1` press or the immediately following key press, clears pending state at `SYN_REPORT`, and rejects events within the debounce interval. Add `monitor(path, sender)` using `evdev::Device::open(path)` and `fetch_events()`; convert each event through `event_type().0`, `code()`, and `value()`. Do not call `Device::grab()`.

- [ ] **Step 4: Run matcher tests in a subagent and verify GREEN**

Run: `cargo test --manifest-path rust_daemon/Cargo.toml --test rog_input_test`

Expected: PASS, 3 tests.

- [ ] **Step 5: Commit Task 3**

```bash
git add rust_daemon/Cargo.toml rust_daemon/src/rog_input.rs rust_daemon/tests/rog_input_test.rs
git commit -m "feat: recognize Linux ROG key events"
```

### Task 4: Emit the system D-Bus signal and expose diagnostics

**Files:**
- Create: `rust_daemon/src/rog_hotkey.rs`
- Create: `rust_daemon/tests/rog_hotkey_test.rs`
- Modify: `rust_daemon/src/main.rs`
- Modify: `install/dbus/com.asuslinuxcontrol.Daemon.conf`
- Modify: `test/install/dbus_config_test.dart`

- [ ] **Step 1: Write failing policy and diagnostic tests**

Test `RogHotkeyStatus` serialization and the explicit fallback gate:

```rust
#![allow(dead_code)]

#[path = "../src/rog_hotkey.rs"]
mod rog_hotkey;

use rog_hotkey::{fallback_enabled, RogHotkeyStatus};

#[test]
fn unsupported_status_has_stable_json_fields() {
    let json = RogHotkeyStatus::unsupported("no compatible HID report").to_json().unwrap();
    let value: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(value["schemaVersion"], 1);
    assert_eq!(value["hardwareSupported"], false);
    assert_eq!(value["inputPath"], "/dev/input/by-path/platform-asus-nb-wmi-event");
    assert_eq!(value["scanCode"], 56);
}

#[test]
fn software_fallback_requires_exact_opt_in() {
    assert!(fallback_enabled(Some("1")));
    assert!(!fallback_enabled(None));
    assert!(!fallback_enabled(Some("true")));
}
```

Extend `test/install/dbus_config_test.dart` with assertions for `GetRogHotkeyStatus` and receiving `RogKeyPressed`.

- [ ] **Step 2: Run both tests in subagents and verify RED**

Run: `cargo test --manifest-path rust_daemon/Cargo.toml --test rog_hotkey_test`

Run: `flutter test test/install/dbus_config_test.dart`

Expected: both FAIL for missing status/module and D-Bus members.

- [ ] **Step 3: Implement orchestration and D-Bus API**

`rog_hotkey.rs` must expose:

```rust
pub const INPUT_PATH: &str = "/dev/input/by-path/platform-asus-nb-wmi-event";

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RogHotkeyStatus {
    pub schema_version: u8,
    pub hardware_supported: bool,
    pub firmware_changed: bool,
    pub fallback_enabled: bool,
    pub input_path: String,
    pub scan_code: u16,
    pub key_code: u16,
    pub reason: Option<String>,
}

pub fn fallback_enabled(value: Option<&str>) -> bool { value == Some("1") }
```

Add `start(connection)` that performs the read-only HID probe first, prepares only the last/ROG slot, and starts `rog_input::monitor` only when the probe succeeds or `ASUS_ROG_HOTKEY_SOFTWARE_FALLBACK=1`. Return a handle that owns the firmware guard and restores it during `shutdown()`.

In `main.rs`, add `mod rog_hotkey; mod rog_input;`, store the shared status, expose `GetRogHotkeyStatus`, and declare the signal:

```rust
#[zbus(signal)]
async fn rog_key_pressed(
    signal_context: &zbus::SignalContext<'_>,
) -> zbus::Result<()>;
```

For every accepted input press call `AsusControlDaemon::rog_key_pressed(&signal_context).await`. Replace the infinite pending future with SIGINT/SIGTERM shutdown selection so `handle.shutdown()` can restore a changed firmware opcode before exit.

Allow the read method and signal in the D-Bus config without adding polkit, because neither operation writes settings on behalf of a client.

- [ ] **Step 4: Run focused daemon and D-Bus tests in subagents**

Run: `cargo test --manifest-path rust_daemon/Cargo.toml --test rog_hotkey_test --test rog_input_test --test rog_mkey_test`

Run: `flutter test test/install/dbus_config_test.dart`

Expected: PASS.

- [ ] **Step 5: Commit Task 4**

```bash
git add rust_daemon/src/main.rs rust_daemon/src/rog_hotkey.rs install/dbus/com.asuslinuxcontrol.Daemon.conf test/install/dbus_config_test.dart
git commit -m "feat: publish ROG hotkey events over D-Bus"
```

### Task 5: Add the per-user activation agent

**Files:**
- Modify: `rust_daemon/Cargo.toml`
- Create: `rust_daemon/src/hotkey_agent.rs`
- Create: `rust_daemon/src/bin/asus-control-hotkey-agent.rs`
- Create: `rust_daemon/tests/hotkey_agent_test.rs`

- [ ] **Step 1: Write a failing activation test**

```rust
#![allow(dead_code)]

#[path = "../src/hotkey_agent.rs"]
mod hotkey_agent;

use hotkey_agent::{activate_app, CommandRunner};

#[derive(Default)]
struct FakeRunner { calls: Vec<(String, Vec<String>)> }

impl CommandRunner for FakeRunner {
    fn run(&mut self, program: &str, args: &[&str]) -> anyhow::Result<()> {
        self.calls.push((program.to_string(), args.iter().map(|v| v.to_string()).collect()));
        Ok(())
    }
}

#[test]
fn signal_activates_the_installed_desktop_app() {
    let mut runner = FakeRunner::default();
    activate_app(&mut runner).unwrap();
    assert_eq!(runner.calls, vec![("gtk-launch".into(), vec!["asus-linux-control".into()])]);
}
```

- [ ] **Step 2: Run the agent test in a subagent and verify RED**

Run: `cargo test --manifest-path rust_daemon/Cargo.toml --test hotkey_agent_test`

Expected: FAIL because `hotkey_agent.rs` is absent.

- [ ] **Step 3: Implement activation and signal subscription**

Add `futures-util = "0.3"` if it is not already a direct dependency. Implement:

```rust
pub trait CommandRunner {
    fn run(&mut self, program: &str, args: &[&str]) -> anyhow::Result<()>;
}

pub fn activate_app(runner: &mut impl CommandRunner) -> anyhow::Result<()> {
    runner.run("gtk-launch", &["asus-linux-control"])
}
```

The production runner must use `std::process::Command::status()` and return an error for a non-zero status. `run()` connects to the system bus, constructs a proxy for `com.asuslinuxcontrol.Daemon` at `/com/asuslinuxcontrol/Daemon`, calls `receive_signal("RogKeyPressed")`, and invokes `activate_app()` for each message. Log failures and continue listening so the next press retries.

Create the binary entry point:

```rust
#[path = "../hotkey_agent.rs"]
mod hotkey_agent;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();
    hotkey_agent::run().await
}
```

- [ ] **Step 4: Run the agent test in a subagent and verify GREEN**

Run: `cargo test --manifest-path rust_daemon/Cargo.toml --test hotkey_agent_test`

Expected: PASS.

- [ ] **Step 5: Commit Task 5**

```bash
git add rust_daemon/Cargo.toml rust_daemon/src/hotkey_agent.rs rust_daemon/src/bin/asus-control-hotkey-agent.rs rust_daemon/tests/hotkey_agent_test.rs
git commit -m "feat: add ROG hotkey session agent"
```

### Task 6: Make the GTK application single-instance and raiseable

**Files:**
- Modify: `linux/runner/my_application.cc`
- Create: `test/install/linux_activation_test.dart`

- [ ] **Step 1: Write a failing source regression test**

```dart
import 'dart:io';
import 'package:flutter_test/flutter_test.dart';

void main() {
  test('Linux runner reuses and presents its active window', () {
    final source = File('linux/runner/my_application.cc').readAsStringSync();
    expect(source, contains('gtk_application_get_active_window'));
    expect(source, contains('gtk_window_present'));
    expect(source, contains('G_APPLICATION_DEFAULT_FLAGS'));
    expect(source, isNot(contains('G_APPLICATION_NON_UNIQUE')));
  });
}
```

- [ ] **Step 2: Run the test in a subagent and verify RED**

Run: `flutter test test/install/linux_activation_test.dart`

Expected: FAIL because the runner is explicitly non-unique and always creates a window.

- [ ] **Step 3: Implement unique activation**

At the start of `my_application_activate()`:

```cpp
GtkWindow* existing = gtk_application_get_active_window(
    GTK_APPLICATION(application));
if (existing != nullptr) {
  gtk_window_present(existing);
  return;
}
```

Change the constructor flag from `G_APPLICATION_NON_UNIQUE` to `G_APPLICATION_DEFAULT_FLAGS`. Keep application ID `com.asuslinuxcontrol.App`; a second `gtk-launch` invocation will route activation to the primary GApplication.

- [ ] **Step 4: Run the activation test in a subagent and verify GREEN**

Run: `flutter test test/install/linux_activation_test.dart`

Expected: PASS.

- [ ] **Step 5: Commit Task 6**

```bash
git add linux/runner/my_application.cc test/install/linux_activation_test.dart
git commit -m "feat: raise existing Linux application window"
```

### Task 7: Install and autostart the session agent

**Files:**
- Create: `install/systemd/asus-control-hotkey-agent.service`
- Modify: `install/install.sh`
- Modify: `packaging/arch/PKGBUILD`
- Modify: `packaging/rpm/asus-linux-control.spec`
- Modify: `test/install/package_metadata_test.dart`

- [ ] **Step 1: Add failing package assertions**

Extend `package_metadata_test.dart` to require:

```dart
expect(File('install/systemd/asus-control-hotkey-agent.service').existsSync(), isTrue);
expect(pkgbuild, contains('asus-control-hotkey-agent'));
expect(pkgbuild, contains('graphical-session.target.wants'));
expect(spec, contains('/usr/lib/asus-linux-control/asus-control-hotkey-agent'));
expect(spec, contains('/usr/lib/systemd/user/asus-control-hotkey-agent.service'));
expect(spec, contains('graphical-session.target.wants/asus-control-hotkey-agent.service'));
```

- [ ] **Step 2: Run package tests in a subagent and verify RED**

Run: `flutter test test/install/package_metadata_test.dart`

Expected: FAIL because the agent and user unit are not packaged.

- [ ] **Step 3: Add the user unit and packaging**

Create:

```ini
[Unit]
Description=ASUS Linux Control ROG hotkey agent
After=graphical-session.target
PartOf=graphical-session.target

[Service]
Type=simple
ExecStart=/usr/lib/asus-linux-control/asus-control-hotkey-agent
Restart=on-failure
RestartSec=2

[Install]
WantedBy=graphical-session.target
```

Install the binary beside the daemon, the unit under `/usr/lib/systemd/user`, and a relative symlink under `graphical-session.target.wants`. Put the binary and user unit in the GUI split package because they activate the GUI; keep raw HID/input access in the root daemon. Update `install.sh` to copy both binaries and create the global user-unit link without attempting to launch a GUI from the root installer.

- [ ] **Step 4: Run package tests in a subagent and verify GREEN**

Run: `flutter test test/install/package_metadata_test.dart test/install/dbus_config_test.dart test/install/linux_activation_test.dart`

Expected: PASS.

- [ ] **Step 5: Commit Task 7**

```bash
git add install/systemd/asus-control-hotkey-agent.service install/install.sh packaging/arch/PKGBUILD packaging/rpm/asus-linux-control.spec test/install/package_metadata_test.dart
git commit -m "packaging: install ROG hotkey session agent"
```

### Task 8: Document hardware verification and run the full checks

**Files:**
- Create: `docs/research/rog-m4-hotkey-manual-verification.md`
- Modify: `README.md`

- [ ] **Step 1: Write the verification document**

Document these exact checks:

```markdown
# ROG/M4 hotkey manual verification

Protocol attribution: the ASUS M-key firmware report sequence is adapted from
G-Helper by seerge: https://github.com/seerge/g-helper/pull/5691

1. Inspect capability without pressing the key:
   `busctl --system call com.asuslinuxcontrol.Daemon /com/asuslinuxcontrol/Daemon com.asuslinuxcontrol.Daemon GetRogHotkeyStatus`
2. Stop the GUI, press ROG/M4, and verify one GUI process appears.
3. Minimize the GUI, press ROG/M4, and verify the same process is presented.
4. Press twice within 300 ms and verify no duplicate activation.
5. Verify M1, M2, M3, and the Strix physical M4 retain their prior actions.
6. Stop the daemon and verify any changed ROG opcode is restored in the log.
7. On unsupported firmware, verify no feature-report write occurs. Only for
   diagnosis, set `ASUS_ROG_HOTKEY_SOFTWARE_FALLBACK=1` in a systemd override.
```

Add a short README section linking this document and repeating the G-Helper attribution.

- [ ] **Step 2: Run Rust formatting in a subagent**

Run: `cargo fmt --manifest-path rust_daemon/Cargo.toml -- --check`

Expected: exit 0.

- [ ] **Step 3: Run all Rust daemon tests in a subagent**

Run: `cargo test --manifest-path rust_daemon/Cargo.toml`

Expected: all tests PASS.

- [ ] **Step 4: Run Flutter analysis and tests in subagents**

Run: `flutter analyze`

Run: `flutter test`

Expected: both exit 0 with no failures.

- [ ] **Step 5: Run release builds in subagents**

Run: `cargo build --release --manifest-path rust_daemon/Cargo.toml --bins`

Run: `flutter build linux --release`

Expected: both `asus-control-daemon` and `asus-control-hotkey-agent` are produced, and the Flutter Linux bundle builds successfully.

- [ ] **Step 6: Commit documentation**

```bash
git add README.md docs/research/rog-m4-hotkey-manual-verification.md
git commit -m "docs: add ROG hotkey verification guide"
```

## External references checked

- G-Helper M-key hardware binding implementation and attribution source: <https://github.com/seerge/g-helper/pull/5691>
- `hidapi` 2.6.6 `send_feature_report` / `get_feature_report`: <https://docs.rs/hidapi/2.6.6/hidapi/struct.HidDevice.html>
- `evdev` 0.13.2 device event API: <https://docs.rs/evdev/0.13.2/evdev/struct.Device.html>
- Linux input key definitions (`KEY_PROG1 = 148`): <https://github.com/torvalds/linux/blob/master/include/uapi/linux/input-event-codes.h>
