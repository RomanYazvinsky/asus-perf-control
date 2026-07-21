# JSON Settings Persistence Design

## Current State

Implemented on 2026-07-21.

The application persists user UI settings and desired hardware settings in a versioned JSON file. The default Linux location follows the XDG Base Directory convention:

`$XDG_CONFIG_HOME/asus-linux-control/settings.json`

When `$XDG_CONFIG_HOME` is missing, empty, or relative, the fallback is:

`$HOME/.config/asus-linux-control/settings.json`

The settings layer lives in `lib/services/settings/`:

- `app_settings.dart` defines `AppSettings`, `UiSettings`, `HardwareSettings`, `TdpDesiredSettings`, and `FanDesiredSettings`.
- `app_settings_store.dart` defines `AppSettingsStore` and `FileAppSettingsStore`.
- `settings_migration.dart` defines the migration and rollback interfaces.

## JSON Schema

Current schema version: `1`.

Representative JSON:

```json
{
  "schemaVersion": 1,
  "ui": {
    "selectedNavigationIndex": 0
  },
  "hardware": {
    "performanceMode": 1,
    "tdp": {
      "powerLimitsEnabled": true,
      "sustainedW": 25,
      "boostW": 35,
      "tempLimitEnabled": true,
      "tempLimitC": 85
    },
    "cpuBoostMode": "efficient_enabled",
    "batteryChargeLimit": 80,
    "gpuMode": "hybrid",
    "fan": {
      "selectedProfile": "balanced",
      "selectedFan": "cpu",
      "controlModes": {
        "cpu": "asusctlCurve",
        "gpu": "asusctlCurve"
      },
      "dynamicEngineConfig": null
    }
  }
}
```

Unknown or invalid fields are not allowed to crash startup. The app logs a read mismatch and falls back to safe defaults for the invalid field or object.

Unset optional hardware settings are omitted from the saved JSON instead of being written as `null`. Missing hardware keys mean "no desired state has been recorded yet".

## Startup Behavior

`AppShell` loads settings on startup and restores `ui.selectedNavigationIndex` when it is within the current navigation destination range. Out-of-range values are ignored and logged.

Hardware blocs receive the same `AppSettingsStore` from `AppShell` through their screens. When a user applies a hardware setting, the bloc writes the desired state into JSON after the backend write succeeds.

When a hardware key is missing, the first successful hardware read seeds that key from the device state without logging a mismatch or applying anything back to hardware. This creates the baseline file for later comparisons.

On hardware load:

- Performance mode is seeded from device state when absent; otherwise it is compared with `hardware.performanceMode`, mismatches are logged, and the saved mode is applied.
- TDP power limits and CPU temperature limit are seeded from device state when absent; otherwise they are compared with `hardware.tdp`, mismatches are logged, and saved values are restored through the existing service APIs.
- CPU Boost mode is seeded from device state when absent; otherwise it is compared with `hardware.cpuBoostMode`, mismatches are logged, and the saved mode is restored only when supported by the current daemon status.
- Battery charge limit is seeded from device state when absent; otherwise it is compared with `hardware.batteryChargeLimit`, mismatches are logged, and the saved value is restored after support is confirmed.
- GPU mode is seeded from device state when absent; otherwise it is compared with `hardware.gpuMode`, mismatches are logged only. It is not auto-applied because changing GPU mode can require logout, reboot, or eGPU availability.
- Fan selected profile, selected fan, and control modes are seeded when `hardware.fan` is absent; otherwise they are restored from `hardware.fan`. Dynamic engine is restarted only when the saved control mode requires it and `dynamicEngineConfig` is explicitly present.

## Logging

Settings read/schema issues are logged through:

`BackendLogger.logEvent('Settings.readMismatch', ...)`

Hardware desired-state mismatches are logged through:

`BackendLogger.logEvent('Settings.hardwareMismatch', ...)`

Logs include keys plus expected/actual values where available.

## Migration And Rollback

`SettingsMigration` is the base migration contract:

```dart
abstract class SettingsMigration {
  int get fromVersion;
  int get toVersion;

  Map<String, dynamic> migrate(Map<String, dynamic> source);
  Map<String, dynamic> rollback(Map<String, dynamic> source);
}
```

`SettingsMigrator` applies migrations up to the current schema and can roll migrated JSON back to a requested target version. The file store persists migrated JSON back to disk after a successful load migration.

Rollback is intentionally explicit. It throws when a requested rollback path is missing, so future schema changes must provide both forward and reverse transforms.

## Verification State

Completed verification:

- `flutter test`: 206 passed, 0 failed after startup hardware seeding was added.
- `flutter test test/services/app_settings_store_test.dart test/blocs/performance_bloc_test.dart test/blocs/tdp/tdp_bloc_test.dart`: 33 passed, 0 failed after startup hardware seeding was added.
- `flutter build linux`: succeeded and built `build/linux/x64/release/bundle/asus-linux-control`.
- `git diff --check`: clean.

Known verification note:

- `flutter analyze` exits with code `1` because of pre-existing `info` diagnostics in `package_updates` and `dist/source`. No analyzer issues remain in the settings persistence changes.
