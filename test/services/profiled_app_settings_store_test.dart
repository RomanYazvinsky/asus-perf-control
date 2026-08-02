import 'package:flutter_application_1/services/settings/app_settings.dart';
import 'package:flutter_application_1/services/settings/app_settings_store.dart';
import 'package:flutter_application_1/services/settings/hardware_profile_store.dart';
import 'package:flutter_test/flutter_test.dart';

import '../test_helpers.dart';

void main() {
  group('ProfiledAppSettingsStore', () {
    test('loads global UI merged with active profile hardware', () async {
      final global = MemoryAppSettingsStore(
        const AppSettings(
          ui: UiSettings(
            selectedNavigationIndex: 3,
            activeHardwareProfileId: 'gaming',
          ),
          hardware: HardwareSettings(performanceMode: 0),
        ),
      );
      final profiles = MemoryHardwareProfileStore([
        const HardwareProfile(
          id: 'gaming',
          name: 'Gaming',
          hardware: HardwareSettings(
            performanceMode: 2,
            batteryChargeLimit: 80,
          ),
        ),
      ]);
      final store = ProfiledAppSettingsStore(
        globalStore: global,
        profileStore: profiles,
      );

      final settings = await store.load();

      expect(settings.ui.selectedNavigationIndex, 3);
      expect(settings.ui.activeHardwareProfileId, 'gaming');
      expect(settings.hardware.performanceMode, 2);
      expect(settings.hardware.batteryChargeLimit, 80);
    });

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

      final updated = await store.update((settings) {
        return settings.copyWith(
          ui: settings.ui.copyWith(selectedNavigationIndex: 7),
          hardware: settings.hardware.copyWith(performanceMode: 1),
        );
      });

      expect(updated.ui.selectedNavigationIndex, 7);
      expect(updated.hardware.performanceMode, 1);
      expect(global.savedSettings.last.ui.selectedNavigationIndex, 7);
      expect(global.savedSettings.last.hardware.performanceMode, 0);
      expect(profiles.savedProfiles.single.hardware.performanceMode, 1);
    });

    test(
      'preserves plain settings behavior without an active profile',
      () async {
        final global = MemoryAppSettingsStore(
          const AppSettings(hardware: HardwareSettings(performanceMode: 0)),
        );
        final profiles = MemoryHardwareProfileStore();
        final store = ProfiledAppSettingsStore(
          globalStore: global,
          profileStore: profiles,
        );

        final updated = await store.update((settings) {
          return settings.copyWith(
            ui: settings.ui.copyWith(selectedNavigationIndex: 4),
            hardware: settings.hardware.copyWith(performanceMode: 2),
          );
        });

        expect(updated.ui.selectedNavigationIndex, 4);
        expect(updated.hardware.performanceMode, 2);
        expect(global.savedSettings.last.ui.selectedNavigationIndex, 4);
        expect(global.savedSettings.last.hardware.performanceMode, 2);
        expect(profiles.savedProfiles, isEmpty);
      },
    );
  });
}
