# Settings Profiles Design

## Current State

The app already persists UI state and desired hardware state through the
versioned `AppSettings` model in `lib/services/settings/`. `AppShell` owns one
`AppSettingsStore` and passes it to the hardware screens. Hardware blocs read
desired settings on load, apply saved desired state to the backend when needed,
and call `settingsStore.update(...)` after successful user-driven hardware
changes.

The existing global settings file remains:

`$XDG_CONFIG_HOME/asus-linux-control/settings.json`

with fallback to:

`$HOME/.config/asus-linux-control/settings.json`

## Scope

Profiles cover only `AppSettings.hardware`.

`AppSettings.ui` remains global and is not copied into, restored from, or saved
inside profiles. This prevents profile activation from unexpectedly moving the
user to a different navigation page.

Each profile is stored as a separate JSON file.

## Storage

Profiles live beside the global settings file:

`$XDG_CONFIG_HOME/asus-linux-control/profiles/<profile-id>.json`

or, when using the existing fallback base:

`$HOME/.config/asus-linux-control/profiles/<profile-id>.json`

The profile file contains metadata and a hardware settings snapshot:

```json
{
  "schemaVersion": 1,
  "id": "gaming",
  "name": "Gaming",
  "hardware": {
    "performanceMode": 2,
    "batteryChargeLimit": 80,
    "gpuMode": "hybrid"
  }
}
```

Profile ids are generated from names by lowercasing, replacing unsupported
characters with `-`, trimming separators, and appending a numeric suffix on
collision. File names are derived only from validated ids, not raw user input.

The active profile id is stored in global `settings.json` at
`ui.activeHardwareProfileId`. The existing UI navigation setting continues to
behave globally.

Adding `ui.activeHardwareProfileId` requires bumping the app settings schema and
adding a migration that defaults the field to absent for existing users.

## Services

Add a profile model and store in `lib/services/settings/`:

- `HardwareProfile` holds `id`, `name`, and `HardwareSettings`.
- `HardwareProfileStore` lists, creates, saves, applies, and deletes profile
  files.
- `ProfiledAppSettingsStore` wraps the existing `AppSettingsStore`. `load()`
  merges global UI settings from `settings.json` with hardware settings from the
  active profile when one exists. `save()` and `update(...)` split writes so UI
  changes go to `settings.json` and hardware changes go to the active profile.

Existing hardware blocs keep using `AppSettingsStore`. This makes all successful
hardware changes automatically persist into the active profile without adding
profile-specific calls to each bloc.

When no active profile exists, `ProfiledAppSettingsStore` behaves like the
current plain settings store for hardware. This keeps first-run and upgrade
behavior safe.

## UI

Add a new `Profiles` page to the app navigation.

The page contains:

- An `Add` button above the table in its default state.
- After pressing `Add`, the button is replaced by a profile name text field plus
  `Save` and `Cancel` buttons.
- A table where each row represents one profile.
- Each row has `Apply` and `Delete` buttons.
- The active profile row is highlighted with a translucent green background.

`Cancel` returns the form to the default `Add` state and clears the draft name.

`Save` validates that the name is not blank, creates a profile by copying the
current hardware settings, immediately marks it active, refreshes the table,
and returns the form to the default `Add` state.

`Apply` marks the selected profile active and reloads the app shell's hardware
screens so their existing load behavior applies the newly selected desired
hardware settings.

`Delete` removes the profile file. The active profile cannot be deleted from the
UI; its delete button is disabled. This avoids leaving the app in an ambiguous
"active profile deleted" state.

## Application Flow

`AsusLinuxControlApp` creates the plain `FileAppSettingsStore`, then wraps it in
`ProfiledAppSettingsStore` before passing it to `AppShell`.

`AppShell` passes both the profiled settings store and the profile store to the
profiles page. When a profile is applied, `AppShell` rebuilds the hardware
screen stack with new keys so the blocs are recreated and reload desired
hardware from the newly active profile.

Existing screens that do not use hardware settings, such as updates and XG
diagnostics, do not need profile behavior.

## Error Handling

Invalid profile JSON is logged through `BackendLogger.logEvent` with a
profile-specific event or `Settings.readMismatch` fields that include the file
path. Invalid profile files are skipped in the table rather than crashing the
app.

Applying a missing or invalid profile leaves the current active profile
unchanged and shows an error in the profiles page.

Saving a duplicate visible name is allowed only after id de-duplication creates
a unique file id. The UI can show duplicate names because the id remains stable
internally.

## Testing

Add focused tests before implementation:

- Profile file store saves, lists, loads, and deletes one JSON file per profile.
- Creating a profile copies the current `HardwareSettings` and marks it active.
- `ProfiledAppSettingsStore.update(...)` writes UI changes to global settings
  and hardware changes to the active profile.
- Without an active profile, `ProfiledAppSettingsStore` preserves current
  behavior.
- Profiles page shows the add form state, saves a new profile, highlights the
  active row, applies another profile, and disables deletion of the active
  profile.
- App shell profile application recreates hardware screens so saved settings are
  reloaded.

## Out Of Scope

Renaming profiles, exporting/importing profiles, reordering profiles, and
confirming deletion are not part of the first implementation.
