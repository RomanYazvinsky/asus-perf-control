# Settings Profiles Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add hardware-only settings profiles stored as one JSON file per profile, with a profiles page for creating, applying, and deleting profiles.

**Architecture:** Extend `AppSettings` with a global `ui.activeHardwareProfileId`, add a file-backed `HardwareProfileStore`, and wrap the existing settings store with `ProfiledAppSettingsStore` so existing hardware blocs keep saving through the same interface. Add a `ProfilesScreen` and let `AppShell` rebuild hardware screens after profile activation so saved hardware is reapplied through existing bloc load behavior.

**Tech Stack:** Flutter/Dart, `dart:io`, `dart:convert`, existing `AppSettingsStore`, existing widget and service tests.

---

### Task 1: Profile Models And File Store

**Files:**
- Modify: `lib/services/settings/app_settings.dart`
- Create: `lib/services/settings/hardware_profile_store.dart`
- Test: `test/services/hardware_profile_store_test.dart`

- [ ] **Step 1: Write failing profile store tests**

Create `test/services/hardware_profile_store_test.dart` with tests for:

```dart
test('creates one JSON file per profile and lists it', () async {
  final store = FileHardwareProfileStore(directory: profilesDir);
  final profile = await store.create(
    name: 'Gaming Mode',
    hardware: const HardwareSettings(performanceMode: 2),
  );

  final profiles = await store.list();
  final raw = jsonDecode(
    await File('${profilesDir.path}/${profile.id}.json').readAsString(),
  );

  expect(profile.id, 'gaming-mode');
  expect(profiles.single.name, 'Gaming Mode');
  expect(profiles.single.hardware.performanceMode, 2);
  expect(raw['hardware']['performanceMode'], 2);
});
```

Also test id collision (`Gaming Mode` then `Gaming Mode` creates `gaming-mode-2`), invalid JSON logging/skipping, and deletion.

- [ ] **Step 2: Run test to verify RED**

Run through a subagent because project instructions require test/build commands in subagents:

`flutter test test/services/hardware_profile_store_test.dart`

Expected: FAIL because `hardware_profile_store.dart` does not exist.

- [ ] **Step 3: Implement minimal model and store**

Add `HardwareProfile` with `schemaVersion`, `id`, `name`, `hardware`, `fromJson`, and `toJson`.

Add `FileHardwareProfileStore` with:

```dart
Future<List<HardwareProfile>> list();
Future<HardwareProfile> create({
  required String name,
  required HardwareSettings hardware,
});
Future<HardwareProfile?> load(String id);
Future<void> save(HardwareProfile profile);
Future<void> delete(String id);
```

Use validated ids for filenames and formatted JSON writes.

- [ ] **Step 4: Run test to verify GREEN**

Run through a subagent:

`flutter test test/services/hardware_profile_store_test.dart`

Expected: PASS.

### Task 2: Active Profile In App Settings And Profiled Store

**Files:**
- Modify: `lib/services/settings/app_settings.dart`
- Modify: `lib/services/settings/settings_migration.dart`
- Modify: `lib/services/settings/app_settings_store.dart`
- Modify: `test/test_helpers.dart`
- Test: `test/services/profiled_app_settings_store_test.dart`
- Test: `test/services/app_settings_store_test.dart`

- [ ] **Step 1: Write failing settings/profiled store tests**

Add tests proving:

```dart
test('updates global ui and active profile hardware separately', () async {
  final global = MemoryAppSettingsStore(
    const AppSettings(
      ui: UiSettings(activeHardwareProfileId: 'gaming'),
      hardware: HardwareSettings(performanceMode: 0),
    ),
  );
  final profiles = MemoryHardwareProfileStore([
    const HardwareProfile(
      id: 'gaming',
      name: 'Gaming',
      hardware: HardwareSettings(performanceMode: 2),
    ),
  ]);
  final store = ProfiledAppSettingsStore(
    globalStore: global,
    profileStore: profiles,
  );

  await store.update((settings) {
    return settings.copyWith(
      ui: settings.ui.copyWith(selectedNavigationIndex: 7),
      hardware: settings.hardware.copyWith(performanceMode: 1),
    );
  });

  expect(global.savedSettings.last.ui.selectedNavigationIndex, 7);
  expect(global.savedSettings.last.hardware.performanceMode, 0);
  expect(profiles.savedProfiles.single.hardware.performanceMode, 1);
});
```

Also test `load()` merges global UI with active profile hardware, and no active profile preserves existing plain settings behavior.

- [ ] **Step 2: Run tests to verify RED**

Run through a subagent:

`flutter test test/services/profiled_app_settings_store_test.dart test/services/app_settings_store_test.dart`

Expected: FAIL because `activeHardwareProfileId`, memory profile store, and `ProfiledAppSettingsStore` do not exist.

- [ ] **Step 3: Implement settings schema update**

Bump `AppSettings.currentSchemaVersion` to `2`. Add nullable `UiSettings.activeHardwareProfileId`, JSON read/write, `copyWith`, equality, and hash code.

Add a `SettingsMigration` from 1 to 2 that writes `schemaVersion: 2` and leaves `ui.activeHardwareProfileId` absent.

- [ ] **Step 4: Implement `ProfiledAppSettingsStore`**

`load()` reads global settings and, when `ui.activeHardwareProfileId` points to a valid profile, returns:

```dart
global.copyWith(hardware: profile.hardware)
```

`save()` and `update(...)` preserve global hardware in `settings.json` when a profile is active, while saving the resulting hardware into the active profile file.

- [ ] **Step 5: Run tests to verify GREEN**

Run through a subagent:

`flutter test test/services/profiled_app_settings_store_test.dart test/services/app_settings_store_test.dart`

Expected: PASS.

### Task 3: Profiles Screen

**Files:**
- Create: `lib/screens/profiles_screen.dart`
- Test: `test/screens/profiles_screen_test.dart`

- [ ] **Step 1: Write failing widget tests**

Cover:

```dart
testWidgets('shows add form, saves profile, and highlights active row', (tester) async {
  final global = MemoryAppSettingsStore(
    const AppSettings(hardware: HardwareSettings(performanceMode: 2)),
  );
  final profiles = MemoryHardwareProfileStore();

  await tester.pumpWidget(MaterialApp(
    home: ProfilesScreen(
      settingsStore: global,
      profileStore: profiles,
      onProfileApplied: () {},
    ),
  ));

  await tester.tap(find.text('Add'));
  await tester.enterText(find.byType(TextField), 'Gaming');
  await tester.tap(find.text('Save'));
  await tester.pumpAndSettle();

  expect(find.text('Gaming'), findsOneWidget);
  expect(profiles.profiles.single.hardware.performanceMode, 2);
  expect(global.current.ui.activeHardwareProfileId, 'gaming');
  expect(find.byKey(const Key('active_profile_row_gaming')), findsOneWidget);
});
```

Also test `Cancel` returns to the `Add` state, `Apply` switches active profile and calls `onProfileApplied`, and active profile delete is disabled.

- [ ] **Step 2: Run tests to verify RED**

Run through a subagent:

`flutter test test/screens/profiles_screen_test.dart`

Expected: FAIL because `ProfilesScreen` does not exist.

- [ ] **Step 3: Implement screen**

Build a `StatefulWidget` that loads profiles in `initState`, renders the add form/table, writes active profile id through `settingsStore.update(...)`, and calls `onProfileApplied` after apply/save.

Use a translucent green row decoration for active rows and disable delete for the active profile.

- [ ] **Step 4: Run tests to verify GREEN**

Run through a subagent:

`flutter test test/screens/profiles_screen_test.dart`

Expected: PASS.

### Task 4: App Shell Integration

**Files:**
- Modify: `lib/main.dart`
- Modify: `test/screens/app_shell_state_persistence_test.dart`

- [ ] **Step 1: Write failing shell integration tests**

Add tests proving the navigation rail includes `Profiles`, and profile apply recreates hardware screens by changing a profile reload token/key.

Expose an optional `onProfileReloadForTest` callback on `AppShell` and assert
that applying a profile calls it exactly once. This observes shell reload
behavior without touching hardware services.

- [ ] **Step 2: Run tests to verify RED**

Run through a subagent:

`flutter test test/screens/app_shell_state_persistence_test.dart`

Expected: FAIL because `Profiles` navigation and shell profile callback do not exist.

- [ ] **Step 3: Wire app and shell**

Create one `FileAppSettingsStore`, derive the profile directory from it, create `FileHardwareProfileStore`, wrap settings in `ProfiledAppSettingsStore`, add the `Profiles` destination, and pass stores into `ProfilesScreen`.

When `ProfilesScreen` reports profile application, increment a reload token and rebuild the hardware screens with keys based on the token.

- [ ] **Step 4: Run tests to verify GREEN**

Run through a subagent:

`flutter test test/screens/app_shell_state_persistence_test.dart`

Expected: PASS.

### Task 5: Formatting And Verification

**Files:**
- All touched Dart files.

- [ ] **Step 1: Format touched Dart files**

Run:

`dart format lib/services/settings/app_settings.dart lib/services/settings/app_settings_store.dart lib/services/settings/hardware_profile_store.dart lib/screens/profiles_screen.dart lib/main.dart test/test_helpers.dart test/services/hardware_profile_store_test.dart test/services/profiled_app_settings_store_test.dart test/services/app_settings_store_test.dart test/screens/profiles_screen_test.dart test/screens/app_shell_state_persistence_test.dart`

- [ ] **Step 2: Run targeted tests through subagents**

Run:

`flutter test test/services/hardware_profile_store_test.dart test/services/profiled_app_settings_store_test.dart test/services/app_settings_store_test.dart test/screens/profiles_screen_test.dart test/screens/app_shell_state_persistence_test.dart`

- [ ] **Step 3: Run broader safety tests through subagents**

Run:

`flutter test test/blocs/performance_bloc_test.dart test/blocs/tdp/tdp_bloc_test.dart test/blocs/cpu_boost/cpu_boost_bloc_test.dart test/blocs/gpu_bloc_test.dart test/screens/home_screen_test.dart`

- [ ] **Step 4: Run analyzer through subagents**

Run:

`flutter analyze`

Expected: report any pre-existing analyzer diagnostics separately from profile changes.

- [ ] **Step 5: Final git status and deferred commit**

Run:

`git status --short`

After implementation is verified, create one final commit that includes the spec, plan, tests, and implementation.
