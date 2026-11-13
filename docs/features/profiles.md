# Hardware profiles & settings

Screen: **Profiles** (`rust_ui/src/ui/screens/profiles.rs`)

A hardware profile is a named set of desired hardware settings. Applying a profile writes all
of them and reloads the hardware screens.

## Files

```text
$XDG_CONFIG_HOME/asus-linux-control/
├── settings.json          # UI state + id of the active profile
└── profiles/
    └── <id>.json          # one file per hardware profile
```

`settings.json`:

```json
{
  "schemaVersion": 1,
  "ui": { "selectedNavigationIndex": 0, "activeHardwareProfileId": "gaming" },
  "hardware": { }
}
```

Each profile:

```json
{
  "schemaVersion": 1,
  "id": "gaming",
  "name": "Gaming",
  "hardware": {
    "performanceMode": 2,
    "tdp": { "powerLimitsEnabled": true, "sustainedW": 45, "boostW": 54,
             "profileTempLimits": { "quiet": 0, "balanced": 90, "performance": 95 } },
    "cpuBoostMode": "aggressive",
    "batteryChargeLimit": 80,
    "gpuMode": "hybrid",
    "fan": { "selectedProfile": "...", "selectedFan": "cpu",
             "controlModes": { "cpu": "asusctlCurve", "gpu": "dynamicEngine" },
             "dynamicEngineConfig": { } }
  }
}
```

All fields are optional. A missing value means "don't touch this setting".

## Behaviour

- Hardware screens read and write the desired values through the **active** profile
  (`ProfiledAppSettingsStore`); UI-only state stays in `settings.json`. Without an active
  profile, `settings.json`'s own `hardware` section is used.
- GPU mode ids: `integrated`, `hybrid`, `ultimate`, `asusMuxDgpu`, `asusEgpu`, `vfio`.
  Fan control mode ids: `asusctlCurve`, `constantSpeed`, `dynamicEngine`.
- Reads are lenient: an invalid value is reported, replaced by its default, and the rest of
  the file still loads (`rust_ui/src/settings/json.rs`).
- Writes are serialised read-modify-write, so concurrent screens can't lose each other's edits.
- The JSON format matches the former Flutter app, so existing settings keep working.

## Code

| Layer | Files |
|---|---|
| UI | `rust_ui/src/ui/screens/profiles.rs` |
| Model | `rust_ui/src/settings/model.rs` |
| Persistence | `rust_ui/src/settings/store.rs`, `rust_ui/src/settings/json.rs` |
