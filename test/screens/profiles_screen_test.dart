import 'package:flutter/material.dart';
import 'package:flutter_application_1/screens/profiles_screen.dart';
import 'package:flutter_application_1/services/settings/app_settings.dart';
import 'package:flutter_application_1/services/settings/hardware_profile_store.dart';
import 'package:flutter_test/flutter_test.dart';

import '../test_helpers.dart';

void main() {
  group('ProfilesScreen', () {
    testWidgets('shows add form, saves profile, and highlights active row', (
      tester,
    ) async {
      final settings = MemoryAppSettingsStore(
        const AppSettings(hardware: HardwareSettings(performanceMode: 2)),
      );
      final profiles = MemoryHardwareProfileStore();
      var appliedCount = 0;

      await tester.pumpWidget(
        MaterialApp(
          home: ProfilesScreen(
            settingsStore: settings,
            profileStore: profiles,
            onProfileApplied: () => appliedCount += 1,
          ),
        ),
      );

      await tester.tap(find.text('Add'));
      await tester.pump();
      await tester.enterText(find.byType(TextField), 'Gaming');
      await tester.tap(find.text('Save'));
      await tester.pumpAndSettle();

      expect(find.text('Gaming'), findsOneWidget);
      expect(profiles.profiles.single.hardware.performanceMode, 2);
      expect(settings.current.ui.activeHardwareProfileId, 'gaming');
      expect(
        find.byKey(const Key('active_profile_row_gaming')),
        findsOneWidget,
      );
      expect(find.text('Add'), findsOneWidget);
      expect(find.byType(TextField), findsNothing);
      expect(appliedCount, 1);
    });

    testWidgets('cancel returns to the add button state', (tester) async {
      final settings = MemoryAppSettingsStore(AppSettings.defaults);
      final profiles = MemoryHardwareProfileStore();

      await tester.pumpWidget(
        MaterialApp(
          home: ProfilesScreen(
            settingsStore: settings,
            profileStore: profiles,
            onProfileApplied: () {},
          ),
        ),
      );

      await tester.tap(find.text('Add'));
      await tester.pump();
      await tester.enterText(find.byType(TextField), 'Draft');
      await tester.tap(find.text('Cancel'));
      await tester.pumpAndSettle();

      expect(find.text('Add'), findsOneWidget);
      expect(find.byType(TextField), findsNothing);
      expect(profiles.profiles, isEmpty);
    });

    testWidgets('apply switches active profile and notifies shell', (
      tester,
    ) async {
      final settings = MemoryAppSettingsStore(
        const AppSettings(ui: UiSettings(activeHardwareProfileId: 'quiet')),
      );
      final profiles = MemoryHardwareProfileStore([
        const HardwareProfile(
          id: 'quiet',
          name: 'Quiet',
          hardware: HardwareSettings(performanceMode: 0),
        ),
        const HardwareProfile(
          id: 'gaming',
          name: 'Gaming',
          hardware: HardwareSettings(performanceMode: 2),
        ),
      ]);
      var appliedCount = 0;

      await tester.pumpWidget(
        MaterialApp(
          home: ProfilesScreen(
            settingsStore: settings,
            profileStore: profiles,
            onProfileApplied: () => appliedCount += 1,
          ),
        ),
      );
      await tester.pumpAndSettle();

      await tester.tap(find.byKey(const Key('apply_profile_gaming')));
      await tester.pumpAndSettle();

      expect(settings.current.ui.activeHardwareProfileId, 'gaming');
      expect(appliedCount, 1);
      expect(
        find.byKey(const Key('active_profile_row_gaming')),
        findsOneWidget,
      );
    });

    testWidgets('active profile delete button is disabled', (tester) async {
      final settings = MemoryAppSettingsStore(
        const AppSettings(ui: UiSettings(activeHardwareProfileId: 'quiet')),
      );
      final profiles = MemoryHardwareProfileStore([
        const HardwareProfile(
          id: 'quiet',
          name: 'Quiet',
          hardware: HardwareSettings(performanceMode: 0),
        ),
      ]);

      await tester.pumpWidget(
        MaterialApp(
          home: ProfilesScreen(
            settingsStore: settings,
            profileStore: profiles,
            onProfileApplied: () {},
          ),
        ),
      );
      await tester.pumpAndSettle();

      final deleteButton = tester.widget<TextButton>(
        find.byKey(const Key('delete_profile_quiet')),
      );

      expect(deleteButton.onPressed, isNull);
      expect(profiles.deletedProfileIds, isEmpty);
    });
  });
}
