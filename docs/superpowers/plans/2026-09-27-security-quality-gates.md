# Security and Quality Gates Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Remove the highest-risk privilege and runtime-safety defects and add one repeatable repository-wide quality entry point.

**Architecture:** Security policy is pinned by semantic tests before configuration changes. Rust runtime safety is improved without beginning the later crate-wide decomposition: handwritten unsafe operations move to safe `nix` APIs, Polkit identifies callers by their unique D-Bus name, and fallible state/time/conversion paths return errors. A small quality script handles host-dependent integration validators while `Makefile` owns the mandatory cross-language checks.

**Tech Stack:** Flutter/Dart tests and analyzer, Rust 2021 with Cargo/Clippy and `nix`, D-Bus, Polkit, udev, Bash, Make.

**Spec:** `docs/superpowers/specs/2026-09-26-full-project-quality-design.md`

## Global Constraints

- Preserve all user-visible hardware-control behavior except for closing confirmed security holes.
- Do not edit `lib/bridge_generated.dart/**` or `rust_core/src/frb_generated.rs` manually.
- Do not overwrite, stage, or revert the existing fan-screen and navigation-plan changes.
- Generated directories (`build`, `dist`, `.dart_tool`, and Cargo `target`) are not maintained source.
- No world-writable device node or direct unprivileged fan/thermal sysfs write is allowed.
- All production fallibility returns a typed/contextual error; it must not panic or silently continue with corrupted state.
- Each implementation task stages and commits only the files listed for that task.
- Before Task 1, record the comparison base with `export QUALITY_BASE_SHA=$(git rev-parse HEAD)` in the execution shell.

## Review Focus

- A local user outside the controlled access group must not gain raw write access to arbitrary ASUS USB or HID devices; Task 1 tests this against the installed udev rules.
- A recycled process ID must not inherit an earlier caller's authorization identity; Task 2 tests that Polkit receives the unique D-Bus sender name and no PID/start-time subject.
- Interrupted input polling must retry, while cancellation and input readiness must remain distinguishable; Task 3 keeps the existing cancellation test and adds an explicit input-ready assertion.
- A poisoned package-update mutex or invalid system clock must return an error without killing the daemon or FFI request; Task 4 covers the mutex and Task 5 covers the clock.
- A machine missing optional packaging validators must report a skip, while missing Dart, Flutter, Cargo, or Rustfmt must fail immediately; Task 6 tests the quality-script command classification.

---

### Task 1: Lock Down udev and Polkit Policy Defaults

**Files:**
- Create: `test/install/security_policy_test.dart`
- Modify: `install/99-asus-control.rules`
- Modify: `install/dbus/com.asuslinuxcontrol.Daemon.policy`

**Interfaces:**
- Consumes: installed udev and Polkit text files.
- Produces: semantic policy tests and least-privilege configuration; no application API changes.

- [ ] **Step 1: Write failing security-policy tests**

Add these assertions in tests with the shown names:

```dart
test('udev rules never grant world-writable device access', () {
  final rules = File('install/99-asus-control.rules').readAsStringSync();
  expect(rules, isNot(contains('MODE="0666"')));
});

test('udev rules do not write permission strings into sysfs attributes', () {
  final rules = File('install/99-asus-control.rules').readAsStringSync();
  expect(rules, isNot(matches(RegExp(r'ATTR\{[^}]+\}\s*=\s*"0666"'))));
});

test('udev hotkey access is restricted to hidraw and plugdev', () {
  final vendorRules = File('install/99-asus-control.rules')
      .readAsLinesSync()
      .where((line) => !line.trimLeft().startsWith('#'))
      .where((line) => line.contains('idVendor') && line.contains('0b05'))
      .toList();
  expect(vendorRules, hasLength(1));
  expect(vendorRules.single, contains('SUBSYSTEM=="hidraw"'));
  expect(vendorRules.single, contains('MODE="0660"'));
  expect(vendorRules.single, contains('GROUP="plugdev"'));
});

test('all mutating polkit actions require retained admin authentication', () {
  final policy = File(
    'install/dbus/com.asuslinuxcontrol.Daemon.policy',
  ).readAsStringSync();
  const actionIds = [
    'com.asuslinuxcontrol.set-temp-limit',
    'com.asuslinuxcontrol.manage-fan-engine',
    'com.asuslinuxcontrol.set-cpu-boost-mode',
    'com.asuslinuxcontrol.manage-updates',
  ];
  for (final actionId in actionIds) {
    final action = RegExp(
      '<action id="${RegExp.escape(actionId)}">([\\s\\S]*?)</action>',
    ).firstMatch(policy)?.group(1);
    expect(action, isNotNull, reason: actionId);
    expect(action, contains('<allow_any>auth_admin</allow_any>'));
    expect(action, contains('<allow_inactive>auth_admin</allow_inactive>'));
    expect(action, contains('<allow_active>auth_admin_keep</allow_active>'));
  }
});
```

The assertions must reject `MODE="0666"`, reject any sysfs `ATTR` assignment whose value is the permission string `0666`, reject a generic `SUBSYSTEM=="usb"` ASUS write rule, require one ASUS `hidraw` rule with `MODE="0660"` and `GROUP="plugdev"`, and inspect each of these action blocks independently:

```text
com.asuslinuxcontrol.set-temp-limit
com.asuslinuxcontrol.manage-fan-engine
com.asuslinuxcontrol.set-cpu-boost-mode
com.asuslinuxcontrol.manage-updates
```

Each block must contain `auth_admin`, `auth_admin`, and `auth_admin_keep` for any, inactive, and active callers respectively.

- [ ] **Step 2: Run the policy tests and confirm the current files fail**

Run: `flutter test test/install/security_policy_test.dart`

Expected: FAIL on the current world-writable/misapplied udev rules and `allow_active=yes` policy entries.

- [ ] **Step 3: Replace the unsafe rules and policy defaults**

In `install/99-asus-control.rules`, remove the keyboard/sysfs permission-value assignments, the generic ASUS USB rule, and broad hwmon mode assignments. Retain one ASUS `hidraw` access rule using mode `0660` and group `plugdev` for the user-session hotkey path.

In the Polkit policy, set all four mutating action blocks to:

```xml
<allow_any>auth_admin</allow_any>
<allow_inactive>auth_admin</allow_inactive>
<allow_active>auth_admin_keep</allow_active>
```

- [ ] **Step 4: Run focused install-policy tests**

Run: `flutter test test/install/security_policy_test.dart test/install/dbus_config_test.dart test/install/package_metadata_test.dart`

Expected: PASS.

- [ ] **Step 5: Commit only the policy task**

```bash
git add test/install/security_policy_test.dart install/99-asus-control.rules install/dbus/com.asuslinuxcontrol.Daemon.policy
git commit -m "security: restrict privileged hardware access"
```

### Task 2: Bind Polkit Authorization to the D-Bus Sender

**Files:**
- Modify: `rust_daemon/src/polkit.rs`
- Test: inline `#[cfg(test)]` module in `rust_daemon/src/polkit.rs`

**Interfaces:**
- Consumes: the unique sender name from `zbus::MessageHeader` already passed to `check_authorization`.
- Produces: `AuthorizationSubject<'a>` with `kind()`, `detail_keys()`, `sender()`, and a private conversion to the Polkit `(kind, a{sv})` payload; `fn authorization_subject(sender: &str) -> AuthorizationSubject<'_>`.

- [ ] **Step 1: Add failing unit tests for the authorization subject**

Add tests for the returned subject type:

```rust
#[test]
fn authorization_subject_uses_system_bus_name() {
    let subject = authorization_subject(":1.42");
    assert_eq!(subject.kind(), "system-bus-name");
}

#[test]
fn authorization_subject_contains_only_the_unique_sender_name() {
    let subject = authorization_subject(":1.42");
    assert_eq!(subject.detail_keys(), ["name"]);
    assert_eq!(subject.sender(), ":1.42");
}
```

Assert that the subject kind is `system-bus-name`, the `name` detail contains a representative unique name such as `:1.42`, and no `pid` or `start-time` key exists.

- [ ] **Step 2: Run the focused Rust test and confirm failure**

Run: `cargo test polkit::tests --manifest-path rust_daemon/Cargo.toml`

Expected: FAIL because `authorization_subject` does not exist.

- [ ] **Step 3: Implement the typed subject and remove PID lookup**

Define the subject type and helper above, convert it to a Polkit `system-bus-name` payload containing only the `name` variant, use it in `check_authorization`, and delete `get_sender_pid`. Keep the existing Polkit action and response handling unchanged.

- [ ] **Step 4: Run Polkit tests and daemon tests**

Run: `cargo test polkit::tests --manifest-path rust_daemon/Cargo.toml`

Run: `cargo test --all-targets --all-features --locked --manifest-path rust_daemon/Cargo.toml`

Expected: PASS.

- [ ] **Step 5: Commit only the Polkit task**

```bash
git add rust_daemon/src/polkit.rs
git commit -m "security: bind polkit checks to dbus sender"
```

### Task 3: Remove Handwritten Unsafe Hotkey Operations

**Files:**
- Modify: `rust_daemon/Cargo.toml`
- Modify: `rust_daemon/src/main.rs`
- Modify: `rust_daemon/src/bin/asus-control-hotkey-agent.rs`
- Modify: `rust_daemon/src/hotkey_agent.rs`
- Modify: `rust_daemon/src/rog_input.rs`
- Modify: `rust_daemon/tests/rog_input_test.rs`

**Interfaces:**
- Consumes: file-descriptor owners implementing `AsFd` and the effective process UID.
- Produces: `pub fn wait_for_input_or_cancel(input: &impl AsFd, cancel: &impl AsFd) -> Result<WaitOutcome>` with no raw-pointer or raw-FD construction.

- [ ] **Step 1: Add the input-ready regression assertion**

Extend `rust_daemon/tests/rog_input_test.rs` with:

```rust
#[test]
fn input_fd_wakes_blocking_wait() {
    use std::io::Write;
    let (input_read, mut input_write) = UnixStream::pair().unwrap();
    let (cancel_read, _cancel_write) = UnixStream::pair().unwrap();
    input_write.write_all(&[1]).unwrap();
    assert_eq!(
        wait_for_input_or_cancel(&input_read, &cancel_read).unwrap(),
        WaitOutcome::InputReady,
    );
}
```

Use `UnixStream::pair`, write to the input pair, and assert `WaitOutcome::InputReady`. Update the existing cancellation test to pass references instead of raw file descriptors; it should fail to compile against the old signature.

- [ ] **Step 2: Confirm the focused test target fails before implementation**

Run: `cargo test --test rog_input_test --manifest-path rust_daemon/Cargo.toml`

Expected: compile failure until the reference-based polling interface exists.

- [ ] **Step 3: Replace libc calls with safe `nix` APIs**

Enable only the `user` and `poll` `nix` features in addition to the current features. Use `nix::unistd::Uid::effective().as_raw()` in `LogindSeatGate::new`. Implement polling with `nix::poll::{poll, PollFd, PollFlags}` and borrowed descriptors from `AsFd`; retry only `Errno::EINTR`.

Add `#![deny(unsafe_code)]` to both daemon binary crate roots so later handwritten unsafe additions fail compilation.

- [ ] **Step 4: Verify hotkey behavior and the unsafe-code gate**

Run: `cargo test --test rog_input_test --test hotkey_agent_test --manifest-path rust_daemon/Cargo.toml`

Run: `cargo clippy --bins --all-features --locked --manifest-path rust_daemon/Cargo.toml -- -D warnings`

Expected: tests PASS; Clippy may still report unrelated baseline findings assigned to Task 5, but it must report no `unsafe_code` or hotkey polling warning.

- [ ] **Step 5: Commit only the hotkey safety task**

```bash
git add rust_daemon/Cargo.toml rust_daemon/src/main.rs rust_daemon/src/bin/asus-control-hotkey-agent.rs rust_daemon/src/hotkey_agent.rs rust_daemon/src/rog_input.rs rust_daemon/tests/rog_input_test.rs
git commit -m "refactor: use safe hotkey system APIs"
```

### Task 4: Make Package-Update State Fail Without Panicking

**Files:**
- Modify: `rust_daemon/src/package_updates/job.rs`
- Modify: `rust_daemon/src/main.rs`
- Modify: `rust_daemon/tests/package_update_backend_test.rs`

**Interfaces:**
- Produces: `UpdateJobError`, with `Command(PackageCommandError)` and `StatePoisoned` variants.
- Produces: `start_update(PackageBackend) -> Result<String, UpdateJobError>`, `get_status(&str) -> Result<Option<UpdateStatus>, UpdateJobError>`, `cancel_update(&str) -> Result<bool, UpdateJobError>`, `start_update_for_test(PackageBackend) -> Result<String, UpdateJobError>`, and `complete_for_test(&str, i32, &str) -> Result<(), UpdateJobError>`.
- Consumes: the same fixed package-manager command allowlist; no shell strings or caller-provided executables.

- [ ] **Step 1: Add a failing poisoned-state test**

Add a test-only `poison_state_for_test(&self)` helper and this integration test:

```rust
#[test]
fn poisoned_update_state_returns_error_instead_of_panicking() {
    let manager = UpdateJobManager::new();
    manager.poison_state_for_test();
    let error = manager
        .start_update_for_test(PackageBackend::FedoraDnf5)
        .unwrap_err();
    assert!(matches!(error, UpdateJobError::StatePoisoned));
}
```

After poisoning the mutex, assert `start_update_for_test` returns `UpdateJobError::StatePoisoned`. Update existing tests to unwrap successful `Result` values explicitly.

- [ ] **Step 2: Run the focused package-update test and confirm failure**

Run: `cargo test --test package_update_backend_test poisoned_update_state_returns_error_instead_of_panicking --manifest-path rust_daemon/Cargo.toml`

Expected: FAIL because the test helper/error type does not exist.

- [ ] **Step 3: Implement typed state errors throughout the manager**

Make every lock acquisition return `StatePoisoned`; propagate command creation with the `Command` variant. Background completion must log a state error rather than panic or discard it silently. Update D-Bus handlers in `main.rs` to map `UpdateJobError` to `zbus::fdo::Error::Failed`.

- [ ] **Step 4: Run package-update and full daemon tests**

Run: `cargo test --test package_update_backend_test --manifest-path rust_daemon/Cargo.toml`

Run: `cargo test --all-targets --all-features --locked --manifest-path rust_daemon/Cargo.toml`

Expected: PASS.

- [ ] **Step 5: Commit only the update-state task**

```bash
git add rust_daemon/src/package_updates/job.rs rust_daemon/src/main.rs rust_daemon/tests/package_update_backend_test.rs
git commit -m "fix: return package update state errors"
```

### Task 5: Clear the Maintained-Source Analyzer and Production Clippy Baseline

**Files:**
- Modify: `analysis_options.yaml`
- Modify: `lib/blocs/package_updates/package_update_bloc.dart`
- Modify: `test/services/package_update_service_test.dart`
- Modify: `rust_core/Cargo.toml`
- Modify: `rust_core/src/api.rs`
- Modify: `rust_core/src/services/battery.rs`
- Modify: `rust_core/src/services/fan.rs`
- Modify: `rust_core/src/services/performance.rs`
- Modify: `rust_core/src/services/tdp.rs`
- Modify: `rust_core/tests/steam_input_tests.rs`
- Modify: `rust_daemon/src/hardware/mod.rs`
- Modify: all maintained `rust_daemon/src/hardware/*.rs` call sites affected by the error-variant rename
- Modify: `rust_daemon/src/hardware/cpu_boost.rs`
- Modify: `rust_daemon/src/hardware/temp.rs`
- Modify: `rust_daemon/src/hardware/fan.rs`
- Modify: `rust_daemon/src/hardware/rog_mkey.rs`
- Modify: `rust_daemon/src/rog_input.rs`

**Interfaces:**
- Produces: clean Flutter analysis for maintained sources, clean `rust_core` all-target Clippy, and clean `rust_daemon` production-binary Clippy.
- Preserves: generated FRB code and all existing public runtime behavior.

- [ ] **Step 1: Add narrow generated/build exclusions and fix Dart lint findings**

Exclude `build/**`, `dist/**`, `.dart_tool/**`, and `lib/bridge_generated.dart/**` in `analysis_options.yaml`. Use a super parameter for `PackageUpdateBloc.seeded` and replace the three underscore callback parameters with named ignored parameters that satisfy the lint.

- [ ] **Step 2: Run Dart formatting and analysis**

Run: `dart format --output=none --set-exit-if-changed lib test`

Run: `flutter analyze`

Expected: PASS with no findings in maintained source or tests.

- [ ] **Step 3: Fix `rust_core` formatting and Clippy findings without broad suppression**

Register the generator-emitted `cfg(frb_expand)` in `rust_core/Cargo.toml` using Cargo's `unexpected_cfgs` check configuration. Reorder imports, remove unused test imports, import Steam Input through the library instead of a test-only `#[path]` module, replace manual range checks with `RangeInclusive::contains`, replace each `io::Error::new(ErrorKind::Other, error)` call with `io::Error::other(error)`, and keep generated code untouched.

In `rust_core/src/api.rs`, introduce and test:

```rust
fn unix_timestamp_millis(now: SystemTime) -> Result<i64, String>;
fn checked_i32(value: u32, field: &str) -> Result<i32, String>;
fn checked_optional_u32(value: i32, field: &str) -> Result<Option<u32>, String>;
```

Use these helpers for sensor timestamps and TDP conversion while preserving the current `-1` FFI skip contract until Workstream 3 replaces it.

- [ ] **Step 4: Fix daemon production Clippy findings**

Rename `HardwareError::{ReadError, WriteError, ParseError}` to `{Read, Write, Parse}` and update maintained call sites. Use `io::Error::other`, remove needless borrows, and either use or remove the two genuinely dead production items (`FirmwareBindingGuard::transport`, `KEY_UNKNOWN`) based on existing tests. Do not add a crate-wide Clippy allow.

- [ ] **Step 5: Run the complete baseline commands**

Run: `cargo fmt --all --manifest-path rust_core/Cargo.toml -- --check`

Run: `cargo clippy --all-targets --all-features --locked --manifest-path rust_core/Cargo.toml -- -D warnings`

Run: `cargo test --all-targets --all-features --locked --manifest-path rust_core/Cargo.toml`

Run: `cargo fmt --all --manifest-path rust_daemon/Cargo.toml -- --check`

Run: `cargo clippy --bins --all-features --locked --manifest-path rust_daemon/Cargo.toml -- -D warnings`

Run: `cargo test --all-targets --all-features --locked --manifest-path rust_daemon/Cargo.toml`

Expected: all commands PASS. Daemon integration-test Clippy expands to `--all-targets` in Workstream 3 after `src/lib.rs` removes duplicate module imports.

- [ ] **Step 6: Commit only baseline-cleanup files**

```bash
git add analysis_options.yaml lib/blocs/package_updates/package_update_bloc.dart test/services/package_update_service_test.dart rust_core/Cargo.toml rust_core/src/api.rs rust_core/src/services/battery.rs rust_core/src/services/fan.rs rust_core/src/services/performance.rs rust_core/src/services/tdp.rs rust_core/tests/steam_input_tests.rs rust_daemon/src/hardware rust_daemon/src/rog_input.rs
git commit -m "chore: establish clean quality baseline"
```

### Task 6: Add the Unified Local Quality Entry Point

**Files:**
- Create: `tool/check_system_integration.sh`
- Modify: `Makefile`
- Modify: `README.md`
- Modify: `test/install/package_metadata_test.dart`

**Interfaces:**
- Produces: `make check`, `make check-flutter`, `make check-rust`, and `make check-system-integration`.
- Produces: `tool/check_system_integration.sh`, whose mandatory syntax/security checks fail on errors and whose unavailable host-specific validators print `SKIP: <tool>`.

- [ ] **Step 1: Add failing metadata tests for the quality targets**

Extend `test/install/package_metadata_test.dart` with assertions that:

```dart
expect(makefile, contains('check: check-flutter check-rust check-system-integration'));
expect(makefile, contains('flutter analyze'));
expect(makefile, contains('cargo clippy'));
expect(makefile, contains('tool/check_system_integration.sh'));
```

Also assert that the integration script exists, uses `bash -n`, reports optional validators as `SKIP`, and never invokes install, `sudo`, `systemctl enable`, package updates, or udev triggering.

- [ ] **Step 2: Run the metadata test and confirm failure**

Run: `flutter test test/install/package_metadata_test.dart`

Expected: FAIL because the quality targets/script do not exist.

- [ ] **Step 3: Implement the mandatory Make targets**

`check-flutter` runs formatting in check mode, `flutter analyze`, and `flutter test`.

`check-rust` runs the six Rust format/Clippy/test commands proven in Task 5.

`check-system-integration` invokes the new script.

`check` depends on the three targets and performs no installation or system mutation.

- [ ] **Step 4: Implement safe system-integration validation**

The script must always run `bash -n` over maintained shell/Arch-hook files. When available, run read-only validators for udev, D-Bus/Polkit XML, desktop metadata, systemd units, and Arch package syntax; a present validator error fails the script. Run AppStream and RPM policy validators in report-only mode during this workstream because their already-confirmed metadata findings belong to Workstream 4; prefix their non-zero summaries with `DEFERRED:` and preserve their output. A missing optional tool prints one `SKIP` line and continues. A systemd validator blocked specifically by sandbox/socket permissions prints `SKIP: systemd-analyze (environment)`; other systemd validation failures remain fatal.

- [ ] **Step 5: Document the quality command and current scope**

Add `make check` to README development/verification instructions. State that daemon production code is strict-Clippy checked now and integration-test Clippy becomes all-target after the library extraction in Workstream 3.

- [ ] **Step 6: Run focused tests and the complete quality entry point**

Run: `flutter test test/install`

Run: `make check`

Expected: install tests PASS; `make check` exits 0, with explicit `SKIP` messages only for validators not installed on the host.

- [ ] **Step 7: Commit only the quality-entry-point task**

```bash
git add tool/check_system_integration.sh Makefile README.md test/install/package_metadata_test.dart
git commit -m "build: add unified project quality checks"
```

### Task 7: Final Workstream Verification and Review

**Files:**
- Review only: all files changed by Tasks 1-6.

**Interfaces:**
- Produces: verified Workstream 1 completion evidence; no new product interface.

- [ ] **Step 1: Re-read the specification and check Workstream 1 coverage**

Confirm device permissions, caller authorization, handwritten daemon unsafe removal, panic-free update state, maintained-source lint cleanliness, and the unified command are all represented in code and tests.

- [ ] **Step 2: Run fresh full verification**

Run: `make check`

Run: `git diff --check "$QUALITY_BASE_SHA"..HEAD`, using the base SHA recorded before Task 1.

Expected: both commands exit 0.

- [ ] **Step 3: Audit forbidden production patterns**

Run repository searches that exclude generated files and tests for handwritten `unsafe`, runtime `unwrap`/`expect`/panic macros, `MODE="0666"`, and sysfs permission-value assignments.

Expected: no Workstream 1 forbidden pattern remains. Any finding outside this workstream is recorded for its owning later plan rather than hidden.

- [ ] **Step 4: Request independent code review**

Use `superpowers:requesting-code-review` with the Workstream 1 base and head SHAs. Fix all Critical and Important findings, then rerun `make check`.

- [ ] **Step 5: Record the handoff for Workstream 2**

Report verification command results, explicit optional-validator skips, remaining findings assigned to Flutter boundary/decomposition work, and confirmation that pre-existing user changes were not committed.
