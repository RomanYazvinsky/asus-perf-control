# Per-Mode Power Settings Design

## Goal

Add per-mode application behavior for TDP, CPU temperature limit, and fan settings while coexisting with `asusd`, `asusctl`, GNOME power modes, and ROG Control Center.

The active system power profile is the source of truth. The app must react when the profile changes outside the app, including GNOME power mode changes, `asusctl profile`, and other clients that write the same `asusd` platform profile.

## Options Considered

### Option A: App-Owned Settings For Everything

The app stores TDP, CPU temperature, and fan settings in its own config file and reapplies them on every profile change.

This is simple internally, but it fights `asusd/asusctl`. If `asusctl` or ROG Control Center changes TDP or fan curves, our daemon could overwrite those changes with stale app config.

### Option B: `asusd`-First With App-Owned CPU Temp

The app treats `asusd` as the owner for platform profile, TDP/PPT, and fan curves. The app stores only the settings that `asusd` does not own in the shape we need, currently CPU temperature limit via `ryzenadj`.

This avoids duplicate sources of truth. External changes are naturally picked up by reading current `asusd` state. The app still adds per-profile CPU temperature behavior that does not exist in `asusd`.

### Option C: Import/Export Sync Layer

The app stores its own per-profile config for all settings, but detects changes from `asusd` and imports them before applying anything.

This can support richer app metadata later, but it adds conflict resolution and race handling now. It is too much for the current requirement.

## Decision

Use Option B.

`asusd` remains the source of truth for:

- active platform profile;
- TDP/PPT values and enable state;
- fan curves and their enabled state.

Our daemon owns:

- per-profile CPU temperature limit;
- application of CPU temperature limit when active platform profile changes;
- a small app config file for CPU temperature settings and any app-only enable flags.

## Profile Model

Use the system `PlatformProfile` values exposed by `asusd`:

- `Balanced`
- `Performance`
- `Quiet`
- `LowPower`
- `Custom`, if reported by the platform

For the app UI, keep the current 3 user-facing modes:

- `Silent`
- `Balanced`
- `Turbo`

Mapping:

- `Silent` maps to `Quiet` when available, otherwise `LowPower`.
- `Balanced` maps to `Balanced`.
- `Turbo` maps to `Performance`.

For app-owned CPU temperature settings, store both `Quiet` and `LowPower` under the same quiet bucket unless a future device exposes both and we decide to split them. This matches `asusd` fan curve behavior, where `Quiet | LowPower` share the quiet fan curve set.

## Coexistence Rules

The app must not overwrite `asusd` state just because its own cache is old.

Rules:

1. On every screen load, TDP and fan data are read from `asusd`.
2. When the user edits TDP or fan settings in our app, the app writes through `asusd` APIs.
3. When `asusctl` or ROG Control Center changes TDP or fan curves, our app shows the new values on next reload/refresh.
4. The daemon only auto-applies our CPU temperature limit on profile changes.
5. The daemon does not auto-apply app-stored TDP or fan settings because those are not app-owned.
6. If applying CPU temperature limit fails, log the error and keep the daemon running.

## TDP/PPT Behavior

TDP remains `asusd`-owned.

The app should continue to use `asusd` Armoury PPT properties for:

- sustained power limit;
- boost power limit;
- power limit enable state;
- profile-specific values.

Where possible, align with the `asusd` model:

- TDP tuning is per `PlatformProfile`;
- `asusd` also separates AC/DC tuning internally;
- the UI should show the current active profile values, not a locally persisted copy.

If the current app D-Bus calls do not expose enough profile/AC/DC detail, add read/write adapter methods that call the existing `asusd` properties without creating an app-side TDP persistence file.

## CPU Temperature Behavior

CPU temperature is app-owned because it uses our daemon and `ryzenadj`.

Persist CPU temperature settings in a daemon-owned config file, for example:

`/var/lib/asus-linux-control/per-mode-settings.json`

Shape:

```json
{
  "version": 1,
  "cpuTemp": {
    "quiet": { "enabled": true, "limitC": 75 },
    "balanced": { "enabled": true, "limitC": 85 },
    "performance": { "enabled": false, "limitC": 95 },
    "custom": { "enabled": false, "limitC": 0 }
  }
}
```

Rules:

- `enabled=false` means do not apply a saved temperature limit for that profile.
- `limitC=0` means clear/no saved limit.
- valid non-zero limits remain `60..=100`.
- on daemon startup, detect the active platform profile and apply that profile's CPU temperature setting.
- on platform profile change, apply the new profile's CPU temperature setting.
- if a profile has no saved setting, do nothing rather than clearing hardware state unexpectedly.

## Fan Behavior

Fan curves stay `asusd`-owned.

The app should keep using `asusd` fan curve APIs:

- `FanCurveData(profile)`
- `SetFanCurve(profile, curve)`
- `SetProfileFanCurveEnabled(profile, fan, enabled)`
- `SetCurvesToDefaults(profile)`

Our daemon does not persist fan curves separately.

If the app has a dynamic fan engine mode, that remains a separate runtime feature. It should not be treated as the saved per-mode fan curve source unless a later design explicitly adds persistent dynamic engine profiles.

## Profile Change Detection

The daemon should observe profile changes from the same system source used by `asusd`:

- preferred: subscribe to `asusd`/platform profile D-Bus property changes if available;
- fallback: monitor `/sys/firmware/acpi/platform_profile` changes, or poll with a low-frequency interval if event monitoring is unreliable.

On change:

1. Resolve the active `PlatformProfile`.
2. Normalize `Quiet` and `LowPower` to the quiet bucket for app-owned CPU temperature settings.
3. Apply CPU temperature setting for the bucket if enabled.
4. Do not write TDP or fan settings.
5. Emit/log enough detail for diagnostics.

## UI Behavior

The existing TDP page should continue to show current values from `asusd`.

CPU temperature controls should become profile-aware:

- show which system profile is currently active;
- save the CPU temperature setting for that active profile bucket;
- optionally allow switching the edit target between `Silent`, `Balanced`, and `Turbo` without changing the active system profile, but the first implementation can edit only the active profile to reduce risk.

Fan UI should continue to expose profile selection for fan curves. It should make clear through state, not extra explanatory text, which profile's curve is being edited.

## Error Handling

- Invalid profile values are logged and ignored.
- Invalid CPU temperature limits are rejected before writing config.
- `ryzenadj` failures are surfaced to the app when initiated by the app and logged when triggered by background profile changes.
- Missing `asusd` keeps existing user-visible errors for TDP/fan/profile.
- Missing fan curve support keeps the fan screen unsupported path.

## Testing

Rust daemon tests:

- config read/write with missing file;
- profile normalization for `Quiet` and `LowPower`;
- CPU temperature validation;
- apply-on-startup chooses active profile setting;
- profile-change handler applies only CPU temp and never calls TDP/fan writers.

Dart tests:

- TDP bloc/service reads from `asusd` and does not use app persistence;
- CPU temperature save/load is profile-aware;
- UI state updates when current profile changes;
- fan service continues passing selected profile through to `asusd`.

Manual verification:

1. Set GNOME power mode to power saver.
2. Confirm app sees `Silent`/quiet profile.
3. Save CPU temp limit for Silent.
4. Switch GNOME power mode to balanced.
5. Confirm daemon applies Balanced CPU temp setting only.
6. Change fan curve with `asusctl`.
7. Reload app fan page and confirm the changed curve is shown, not overwritten.
8. Change TDP/PPT with ROG Control Center or `asusctl` where supported.
9. Reload app TDP page and confirm current `asusd` values are shown.

## Out Of Scope

- Replacing `asusd` fan curve persistence.
- Replacing `asusd` TDP/PPT persistence.
- Persistent dynamic fan engine profiles.
- Conflict-resolution UI for simultaneous edits.
- Direct fan duty/RPM control.
