# JSON Settings Persistence Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Persist UI and hardware desired settings as a versioned JSON file, read it on startup, log mismatches, and provide explicit migration and rollback interfaces.

**Architecture:** Add a small settings domain under `lib/services/settings/`: immutable settings models, a file-backed XDG JSON store, a migration pipeline, and hardware restore helpers. Existing blocs remain the integration points: they save desired hardware settings when the user applies changes and compare/restore safe settings after hardware load.

**Tech Stack:** Flutter/Dart, `dart:io`, `dart:convert`, existing BLoC tests, existing `BackendLogger`.

---

### Task 1: Versioned Settings Store and Migrations

**Files:**
- Create: `lib/services/settings/app_settings.dart`
- Create: `lib/services/settings/settings_migration.dart`
- Create: `lib/services/settings/app_settings_store.dart`
- Test: `test/services/app_settings_store_test.dart`

- [ ] Write failing tests for default load, JSON round trip, invalid JSON logging, schema mismatch logging, migration, and rollback interface availability.
- [ ] Implement `AppSettings`, `UiSettings`, `HardwareSettings`, and nested hardware setting value objects with `schemaVersion = 1`.
- [ ] Implement `SettingsMigration` with `version`, `migrate`, and `rollback`.
- [ ] Implement `SettingsMigrator` to migrate up to the current schema and roll back one step when asked.
- [ ] Implement `FileAppSettingsStore` using `$XDG_CONFIG_HOME/asus-linux-control/settings.json`, falling back to `$HOME/.config/asus-linux-control/settings.json`.
- [ ] Run `flutter test test/services/app_settings_store_test.dart`.

### Task 2: UI Startup Persistence

**Files:**
- Modify: `lib/main.dart`
- Test: `test/screens/app_shell_state_persistence_test.dart`

- [ ] Write failing widget tests showing `AppShell` reads `selectedNavigationIndex` and saves changes.
- [ ] Add injectable `AppSettingsStore` to `AsusLinuxControlApp` and `AppShell`.
- [ ] Load settings during `AppShell.initState`, validate destination index against current navigation count, and log mismatches.
- [ ] Save updated UI settings on navigation changes.
- [ ] Run `flutter test test/screens/app_shell_state_persistence_test.dart`.

### Task 3: Hardware Desired State Persistence

**Files:**
- Modify: `lib/blocs/performance/performance_bloc.dart`
- Modify: `lib/blocs/tdp/tdp_bloc.dart`
- Modify: `lib/blocs/cpu_boost/cpu_boost_bloc.dart`
- Modify: `lib/blocs/battery/battery_bloc.dart`
- Modify: `lib/blocs/gpu/gpu_bloc.dart`
- Modify: `lib/blocs/fan/fan_bloc.dart`
- Tests: relevant bloc tests under `test/blocs/`

- [ ] Write failing bloc tests for saving applied hardware settings and logging mismatches on load.
- [ ] On safe hardware settings, compare loaded hardware state with saved desired state and apply saved settings when supported.
- [ ] For GPU, compare and log mismatch only; do not auto-apply at startup.
- [ ] For fans, persist selected profile/fan/control mode and restore dynamic engine only when a saved `dynamicEngineConfig` explicitly exists.
- [ ] Run targeted bloc tests.

### Task 4: Full Verification

**Files:**
- No source files beyond previous tasks.

- [ ] Run formatting for touched Dart files.
- [ ] Run targeted Flutter tests through a subagent.
- [ ] Run `flutter analyze` through a subagent.
- [ ] Report any verification gaps clearly.
