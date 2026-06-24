# Package Auto-Updates Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Deliver native Linux package distribution and repository-backed package updates for ASUS Linux Control without letting the Flutter GUI execute arbitrary privileged commands.

**Architecture:** Packaging is split into meta, GUI, and daemon packages for RPM and pacman ecosystems. The privileged Rust daemon owns backend detection, package-manager command allowlists, update job state, and polkit authorization. Flutter reads structured JSON over system D-Bus and presents update status plus a one-click update button.

**Tech Stack:** Flutter/Dart, flutter_bloc, dbus Dart package, Rust daemon, zbus, tokio, serde_json, systemd, polkit, RPM spec files, PKGBUILD, pacman install scripts.

---

## File Structure

- Create `packaging/rpm/asus-linux-control.spec`: RPM spec with three subpackages: `asus-linux-control`, `asus-linux-control-gui`, and `asus-linux-control-daemon`.
- Create `packaging/arch/PKGBUILD`: Arch split package for `asus-linux-control`, `asus-linux-control-gui`, and `asus-linux-control-daemon`.
- Create `packaging/arch/asus-linux-control-daemon.install`: pacman lifecycle hooks for daemon reload and user-facing enable/start instructions.
- Create `packaging/README.md`: repository setup, signing, install, update, and SteamOS/Bazzite caveats.
- Modify `install/dbus/com.asuslinuxcontrol.Daemon.conf`: allow update D-Bus method calls.
- Modify `install/dbus/com.asuslinuxcontrol.Daemon.policy`: add `com.asuslinuxcontrol.manage-updates` with admin authentication for every session type.
- Create `rust_daemon/src/package_updates/mod.rs`: module exports and shared constants.
- Create `rust_daemon/src/package_updates/status.rs`: versioned JSON response structs and error codes.
- Create `rust_daemon/src/package_updates/backend.rs`: OS detection and fixed backend selection.
- Create `rust_daemon/src/package_updates/commands.rs`: executable-plus-args allowlist; no shell command strings.
- Create `rust_daemon/src/package_updates/parser.rs`: backend-specific output parsing.
- Create `rust_daemon/src/package_updates/job.rs`: single-running-update state machine and process execution.
- Modify `rust_daemon/src/main.rs`: add update state to `AsusControlDaemon` and expose `GetPackageBackend`, `CheckForUpdates`, `StartUpdate`, `GetUpdateStatus`, and `CancelUpdate`.
- Create `rust_daemon/tests/package_update_backend_test.rs`: backend detection, command allowlist, parsing, schema, and job-state tests.
- Create `lib/services/package_update_service.dart`: Dart models and D-Bus client wrapper for update JSON.
- Modify `lib/services/bridge_service.dart`: own and expose `PackageUpdateService` for UI/bloc injection.
- Create `lib/blocs/package_updates/package_update_event.dart`: update screen events.
- Create `lib/blocs/package_updates/package_update_state.dart`: UI-ready update states.
- Create `lib/blocs/package_updates/package_update_bloc.dart`: check/start/poll/cancel orchestration.
- Create `lib/screens/package_updates_screen.dart`: update status area and action controls.
- Modify `lib/main.dart`: add `Updates` navigation destination and screen.
- Create `test/services/package_update_service_test.dart`: JSON parsing and D-Bus method mapping tests.
- Create `test/blocs/package_update_bloc_test.dart`: state transitions for checking, available, running, failed, unsupported, daemon unavailable, and reboot required.
- Create `test/screens/package_updates_screen_test.dart`: disabled button states and visible status copy.
- Modify `test/install/dbus_config_test.dart`: assert update methods and polkit action are present.

---

### Task 1: Add Native Package Skeletons

**Files:**
- Create: `packaging/rpm/asus-linux-control.spec`
- Create: `packaging/arch/PKGBUILD`
- Create: `packaging/arch/asus-linux-control-daemon.install`
- Create: `packaging/README.md`
- Test: `test/install/package_metadata_test.dart`

- [ ] **Step 1: Write package metadata tests**

Create `test/install/package_metadata_test.dart` with assertions that:

- RPM spec contains `%package gui`, `%package daemon`, `Requires: asus-linux-control-gui`, and `Requires: asus-linux-control-daemon`.
- RPM daemon package owns `/usr/lib/systemd/system/asus-control-daemon.service`, `/usr/share/dbus-1/system-services/com.asuslinuxcontrol.Daemon.service`, `/usr/share/dbus-1/system.d/com.asuslinuxcontrol.Daemon.conf`, `/usr/share/polkit-1/actions/com.asuslinuxcontrol.Daemon.policy`, and `/usr/lib/udev/rules.d/99-asus-control.rules`.
- RPM daemon scriptlets use `%systemd_post`, `%systemd_preun`, and `%systemd_postun_with_restart`.
- PKGBUILD has `pkgbase=asus-linux-control`, split package functions for all three package names, `depends=('asus-linux-control-gui' 'asus-linux-control-daemon')` in the meta package, and `install=asus-linux-control-daemon.install` in the daemon package.
- pacman install script calls `systemctl daemon-reload` but does not call `systemctl enable`.

- [ ] **Step 2: Run test to verify it fails**

Run: `flutter test test/install/package_metadata_test.dart`

Expected: FAIL because packaging files do not exist yet.

- [ ] **Step 3: Add RPM spec**

Create `packaging/rpm/asus-linux-control.spec` with:

- `Name: asus-linux-control`
- one no-payload meta package
- `asus-linux-control-gui` subpackage owning the Flutter Linux bundle, launcher, desktop file, icon, and metainfo
- `asus-linux-control-daemon` subpackage owning the daemon binary and system integration files
- `BuildRequires: systemd`
- `%{?systemd_requires}`
- daemon `%post`, `%preun`, and `%postun` scriptlets using systemd macros
- no automatic service enablement

- [ ] **Step 4: Add Arch split package**

Create `packaging/arch/PKGBUILD` as a split package with:

- `pkgbase=asus-linux-control`
- `pkgname=('asus-linux-control' 'asus-linux-control-gui' 'asus-linux-control-daemon')`
- meta package depending on GUI and daemon
- GUI package depending on daemon
- daemon package installing daemon binary, systemd unit, D-Bus service/config, polkit policy, and udev rule under `/usr`
- `install=asus-linux-control-daemon.install` only for daemon package

- [ ] **Step 5: Add pacman install script**

Create `packaging/arch/asus-linux-control-daemon.install` with:

- `post_install()` and `post_upgrade()` running `systemctl daemon-reload`
- `post_install()` printing `systemctl enable --now asus-control-daemon.service`
- `post_remove()` running `systemctl daemon-reload`
- no unattended update enablement

- [ ] **Step 6: Add repository documentation**

Create `packaging/README.md` documenting:

- Fedora/Bazzite signed RPM repo deliverables and GPG fingerprint requirement
- Arch/SteamOS signed pacman repo block and package signing key fingerprint requirement
- local files work for install only, not auto-update checks
- Bazzite uses rpm-ostree layered packages and generally requires reboot
- SteamOS is advanced and may return unsupported

- [ ] **Step 7: Verify package metadata**

Run: `flutter test test/install/package_metadata_test.dart`

Expected: PASS.

- [ ] **Step 8: Commit**

Run:

```bash
git add packaging test/install/package_metadata_test.dart
git commit -m "feat: add native package metadata"
```

---

### Task 2: Add Polkit And D-Bus Contract For Updates

**Files:**
- Modify: `install/dbus/com.asuslinuxcontrol.Daemon.policy`
- Modify: `install/dbus/com.asuslinuxcontrol.Daemon.conf`
- Modify: `test/install/dbus_config_test.dart`

- [ ] **Step 1: Extend install config tests**

Modify `test/install/dbus_config_test.dart` to assert:

- policy contains action id `com.asuslinuxcontrol.manage-updates`
- `allow_any`, `allow_inactive`, and `allow_active` for that action are `auth_admin`
- D-Bus config allows `GetPackageBackend`, `CheckForUpdates`, `StartUpdate`, `GetUpdateStatus`, and `CancelUpdate`

- [ ] **Step 2: Run test to verify it fails**

Run: `flutter test test/install/dbus_config_test.dart`

Expected: FAIL because update action and methods are not present.

- [ ] **Step 3: Add polkit action**

Add action `com.asuslinuxcontrol.manage-updates` to `install/dbus/com.asuslinuxcontrol.Daemon.policy` with message `Authentication is required to install ASUS Linux Control package updates` and defaults:

```xml
<allow_any>auth_admin</allow_any>
<allow_inactive>auth_admin</allow_inactive>
<allow_active>auth_admin</allow_active>
```

- [ ] **Step 4: Add D-Bus allow entries**

Add `send_member` entries for the five update methods in `install/dbus/com.asuslinuxcontrol.Daemon.conf`.

- [ ] **Step 5: Verify install config**

Run: `flutter test test/install/dbus_config_test.dart`

Expected: PASS.

- [ ] **Step 6: Commit**

Run:

```bash
git add install/dbus test/install/dbus_config_test.dart
git commit -m "feat: add package update dbus policy"
```

---

### Task 3: Implement Daemon Update Models And Backend Detection

**Files:**
- Create: `rust_daemon/src/package_updates/mod.rs`
- Create: `rust_daemon/src/package_updates/status.rs`
- Create: `rust_daemon/src/package_updates/backend.rs`
- Create: `rust_daemon/src/package_updates/commands.rs`
- Test: `rust_daemon/tests/package_update_backend_test.rs`

- [ ] **Step 1: Write Rust tests**

Create tests covering:

- Bazzite is selected before generic Fedora when `/etc/os-release` contains Bazzite identifiers.
- SteamOS returns `steamos-pacman` only when pacman exists and the package database is writable.
- Fedora prefers `fedora-dnf5` over `fedora-dnf`.
- Arch returns `arch-pacman`.
- unknown OS returns `unsupported`.
- every backend command is represented as executable plus fixed argument vector.
- unknown backend command lookup returns an error instead of falling back to a shell string.
- serialized update status includes `schemaVersion: 1`, `backend`, `state`, `installedVersion`, `availableVersion`, `rebootRequired`, `summary`, `errorCode`, and `logTail`.

- [ ] **Step 2: Run tests to verify they fail**

Run: `cd rust_daemon && cargo test --test package_update_backend_test`

Expected: FAIL because `package_updates` module does not exist.

- [ ] **Step 3: Add status model**

Create `status.rs` with:

- `PackageBackend` enum: `FedoraDnf`, `FedoraDnf5`, `BazziteRpmOstree`, `ArchPacman`, `SteamOsPacman`, `Unsupported`
- `UpdateState` enum: `Idle`, `Checking`, `Available`, `UpToDate`, `Running`, `Succeeded`, `Failed`, `Unsupported`
- `UpdateErrorCode` enum matching the design
- `UpdateStatus` struct with schema version fixed to `1`
- `to_json()` function using `serde_json::to_string`

- [ ] **Step 4: Add backend detection**

Create `backend.rs` with pure functions that accept testable inputs:

- parsed os-release key/value map
- executable availability map
- pacman database writability flag

Production code may read `/etc/os-release`, search `PATH`, and inspect pacman database metadata, but tests must exercise pure functions.

- [ ] **Step 5: Add command allowlist**

Create `commands.rs` with fixed arrays for:

- `dnf5 check-upgrade asus-linux-control`
- `dnf5 upgrade --assumeyes asus-linux-control`
- `dnf check-update asus-linux-control`
- `dnf upgrade -y asus-linux-control`
- `rpm-ostree upgrade --check`
- `rpm-ostree upgrade`
- `pacman -Qu asus-linux-control`
- `pacman -Syu --needed --noconfirm asus-linux-control`

Do not store or construct shell command strings.

- [ ] **Step 6: Wire module exports**

Create `mod.rs` exporting `backend`, `commands`, and `status`; add `mod package_updates;` in `rust_daemon/src/main.rs`.

- [ ] **Step 7: Verify Rust tests**

Run: `cd rust_daemon && cargo test --test package_update_backend_test`

Expected: PASS.

- [ ] **Step 8: Commit**

Run:

```bash
git add rust_daemon/src/package_updates rust_daemon/src/main.rs rust_daemon/tests/package_update_backend_test.rs
git commit -m "feat: add package update backend detection"
```

---

### Task 4: Implement Check Parsing And Read-Only D-Bus Methods

**Files:**
- Create: `rust_daemon/src/package_updates/parser.rs`
- Modify: `rust_daemon/src/package_updates/mod.rs`
- Modify: `rust_daemon/src/main.rs`
- Test: `rust_daemon/tests/package_update_backend_test.rs`

- [ ] **Step 1: Add parser tests**

Extend Rust tests for:

- DNF5 JSON output with package `asus-linux-control` maps to `available`.
- DNF4 check output with `asus-linux-control.x86_64` maps to `available`.
- `dnf5` exit code `0` maps to `upToDate`; exit code `100` with matching package maps to `available`.
- pacman `asus-linux-control 1.0.0-1 -> 1.0.1-1` maps to `available`.
- rpm-ostree check output indicating no changes maps to `upToDate`.
- lock, signature, network, and missing repo text map to the design error codes.

- [ ] **Step 2: Run tests to verify they fail**

Run: `cd rust_daemon && cargo test --test package_update_backend_test`

Expected: FAIL because parser is missing.

- [ ] **Step 3: Add parser module**

Create `parser.rs` with backend-specific parsing functions returning `UpdateStatus`. Keep the parsing deterministic and unit-testable by passing stdout, stderr, and exit code as values.

- [ ] **Step 4: Add read-only daemon methods**

Modify `AsusControlDaemon` in `rust_daemon/src/main.rs` to expose:

- `get_package_backend() -> Result<String, zbus::fdo::Error>`
- `check_for_updates() -> Result<String, zbus::fdo::Error>`

`CheckForUpdates` should:

- detect backend
- return `unsupported` JSON for unknown platforms
- run only the allowlisted check command for supported backends
- never require polkit
- serialize all results through schema version `1`

- [ ] **Step 5: Verify daemon tests**

Run:

```bash
cd rust_daemon && cargo test
```

Expected: PASS.

- [ ] **Step 6: Commit**

Run:

```bash
git add rust_daemon/src/package_updates rust_daemon/src/main.rs rust_daemon/tests/package_update_backend_test.rs
git commit -m "feat: add package update checks"
```

---

### Task 5: Implement One-Click Update Job Execution

**Files:**
- Create: `rust_daemon/src/package_updates/job.rs`
- Modify: `rust_daemon/src/package_updates/mod.rs`
- Modify: `rust_daemon/src/main.rs`
- Test: `rust_daemon/tests/package_update_backend_test.rs`

- [ ] **Step 1: Add job tests**

Add tests for:

- `StartUpdate` creates one update id when idle.
- second `StartUpdate` while running returns the existing id.
- status transitions from `running` to `succeeded` on exit code `0`.
- failed command maps to `update_failed` unless stderr matches a more specific error.
- Bazzite/rpm-ostree successful update reports `rebootRequired: true`.
- cancel marks job failed or canceled without spawning a second process.

- [ ] **Step 2: Run tests to verify they fail**

Run: `cd rust_daemon && cargo test --test package_update_backend_test`

Expected: FAIL because job state is missing.

- [ ] **Step 3: Add job manager**

Create `job.rs` with:

- in-memory `UpdateJobManager`
- one active job slot
- opaque update ids
- progress log tail capped to a fixed number of lines
- process spawning through `tokio::process::Command`
- no `sh -c`
- no user-supplied executable, args, package names, URLs, or shell fragments

- [ ] **Step 4: Add privileged daemon methods**

Modify `AsusControlDaemon` to expose:

- `start_update() -> Result<String, zbus::fdo::Error>`
- `get_update_status(update_id: String) -> Result<String, zbus::fdo::Error>`
- `cancel_update(update_id: String) -> Result<bool, zbus::fdo::Error>`

`StartUpdate` and `CancelUpdate` must call polkit with `com.asuslinuxcontrol.manage-updates`. `GetUpdateStatus` is read-only and must not require polkit.

- [ ] **Step 5: Verify daemon tests**

Run:

```bash
cd rust_daemon && cargo test
```

Expected: PASS.

- [ ] **Step 6: Commit**

Run:

```bash
git add rust_daemon/src/package_updates rust_daemon/src/main.rs rust_daemon/tests/package_update_backend_test.rs
git commit -m "feat: add daemon package update jobs"
```

---

### Task 6: Add Dart Update Service

**Files:**
- Create: `lib/services/package_update_service.dart`
- Modify: `lib/services/bridge_service.dart`
- Test: `test/services/package_update_service_test.dart`

- [ ] **Step 1: Write Dart service tests**

Create tests for:

- parsing every backend enum string
- parsing every state enum string
- parsing every error code string
- parsing `availableVersion: null` and `rebootRequired: null`
- method names and return signatures: `GetPackageBackend`, `CheckForUpdates`, `StartUpdate`, `GetUpdateStatus`, `CancelUpdate`
- daemon unavailable maps to a typed service exception

- [ ] **Step 2: Run tests to verify they fail**

Run: `flutter test test/services/package_update_service_test.dart`

Expected: FAIL because service does not exist.

- [ ] **Step 3: Add service and models**

Create `package_update_service.dart` with:

- `PackageUpdateBackend`, `PackageUpdateState`, and `PackageUpdateErrorCode` enums
- `PackageUpdateStatus` immutable model
- `PackageUpdateService` interface
- `DbusPackageUpdateService` implementation using existing `DbusClient` constants
- JSON parsing with explicit unknown-value errors

- [ ] **Step 4: Wire BridgeService**

Modify `BridgeService` to own a `PackageUpdateService`, accept it in `BridgeService.test`, dispose resources where needed, and expose update methods through a small interface.

- [ ] **Step 5: Verify Dart service tests**

Run: `flutter test test/services/package_update_service_test.dart`

Expected: PASS.

- [ ] **Step 6: Commit**

Run:

```bash
git add lib/services test/services/package_update_service_test.dart
git commit -m "feat: add package update dart service"
```

---

### Task 7: Add Update Bloc And UI

**Files:**
- Create: `lib/blocs/package_updates/package_update_event.dart`
- Create: `lib/blocs/package_updates/package_update_state.dart`
- Create: `lib/blocs/package_updates/package_update_bloc.dart`
- Create: `lib/screens/package_updates_screen.dart`
- Modify: `lib/main.dart`
- Test: `test/blocs/package_update_bloc_test.dart`
- Test: `test/screens/package_updates_screen_test.dart`

- [ ] **Step 1: Write Bloc tests**

Cover:

- initial load requests backend and then checks for updates
- unsupported backend emits disabled update state
- available update enables update button
- running update disables update button and polls status
- succeeded with reboot required shows reboot-required state
- daemon unavailable emits actionable unavailable state
- authorization denied maps to a failed state with `authorization_denied`

- [ ] **Step 2: Write screen tests**

Cover visible UI for:

- backend name
- installed version
- available version
- up to date
- unsupported OS
- missing repository
- signature error
- package database lock
- running update progress
- reboot required

Assert the update button is disabled for unsupported, no update, running, and daemon unavailable states.

- [ ] **Step 3: Run tests to verify they fail**

Run:

```bash
flutter test test/blocs/package_update_bloc_test.dart test/screens/package_updates_screen_test.dart
```

Expected: FAIL because Bloc and screen do not exist.

- [ ] **Step 4: Add Bloc**

Implement events:

- `LoadPackageUpdateStatus`
- `CheckPackageUpdatesRequested`
- `StartPackageUpdateRequested`
- `PollPackageUpdateStatusRequested`
- `CancelPackageUpdateRequested`

Implement states with fields needed by the screen:

- status model
- `isChecking`
- `isUpdateRunning`
- `canStartUpdate`
- user-facing summary

- [ ] **Step 5: Add screen**

Create `PackageUpdatesScreen` matching existing Material 3 dark UI patterns:

- compact status cards
- backend/version rows
- specific error text
- refresh button
- update button
- cancel button only while running
- reboot-required indicator

Do not expose command names, arguments, package names beyond `asus-linux-control`, URLs, or shell fragments as UI inputs.

- [ ] **Step 6: Add navigation**

Modify `lib/main.dart` to add an `Updates` `NavigationRailDestination` and append `PackageUpdatesScreen()` to the `IndexedStack`.

- [ ] **Step 7: Verify UI tests**

Run:

```bash
flutter test test/blocs/package_update_bloc_test.dart test/screens/package_updates_screen_test.dart test/widget_test.dart
```

Expected: PASS.

- [ ] **Step 8: Commit**

Run:

```bash
git add lib/blocs/package_updates lib/screens/package_updates_screen.dart lib/main.dart test/blocs/package_update_bloc_test.dart test/screens/package_updates_screen_test.dart
git commit -m "feat: add package update screen"
```

---

### Task 8: End-To-End Verification And Rollout Docs

**Files:**
- Modify: `packaging/README.md`
- Modify: `README.md`
- Optional create: `docs/research/package-update-manual-verification.md`

- [ ] **Step 1: Add manual verification checklist**

Document exact manual checks for:

- Fedora RPM install and update from signed repository
- Bazzite rpm-ostree install and staged update
- Arch pacman install and update from signed repository
- SteamOS supported update or clear unsupported status
- polkit denial
- package manager lock
- missing repository
- signature failure

- [ ] **Step 2: Run full automated verification**

Run:

```bash
flutter test
cd rust_daemon && cargo test
dart format lib test
cargo fmt --manifest-path rust_daemon/Cargo.toml -- --check
```

Expected: all PASS and formatting check clean.

- [ ] **Step 3: Build release artifacts locally**

Run:

```bash
make build-daemon
flutter build linux --release
```

Expected: daemon binary and Flutter Linux release bundle are produced.

- [ ] **Step 4: Update user docs**

Update `README.md` with:

- recommended install target `asus-linux-control`
- native package manager update model
- no Flatpak support for this feature
- no custom downloader
- Bazzite reboot expectation
- SteamOS advanced target warning

- [ ] **Step 5: Commit**

Run:

```bash
git add README.md packaging/README.md docs/research/package-update-manual-verification.md
git commit -m "docs: document package update rollout"
```

---

## Review Checklist

- Packaging installs one top-level package that pulls GUI and daemon.
- RPM packages use systemd macros and do not silently enable daemon or unattended updates.
- pacman lifecycle script reloads systemd and prints enable/start instructions without enabling unattended updates.
- `StartUpdate` and `CancelUpdate` require `com.asuslinuxcontrol.manage-updates`.
- GUI never sends commands, args, package manager names, URLs, or shell fragments.
- Daemon chooses commands only from backend allowlist.
- Unknown OS returns `unsupported`.
- Missing repository returns `missing_repository`.
- Bazzite successful rpm-ostree update reports reboot required.
- SteamOS either uses the allowlisted pacman path or returns unsupported with a clear reason.
- Automated tests cover Rust backend detection, command allowlist, parsing, job state, Dart JSON parsing, Bloc mapping, and UI disabled states.
