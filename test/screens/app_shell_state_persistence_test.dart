import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:flutter_application_1/main.dart';
import 'package:flutter_application_1/screens/home_screen.dart';
import 'package:flutter_application_1/screens/tdp_screen.dart';
import 'package:flutter_application_1/screens/gpu_screen.dart';
import 'package:flutter_application_1/screens/profiles_screen.dart';
import 'package:flutter_application_1/screens/xg_mobile_fan_diagnostic_screen.dart';
import 'package:flutter_application_1/services/settings/app_settings.dart';
import 'package:flutter_application_1/services/settings/hardware_profile_store.dart';

import '../test_helpers.dart';

void main() {
  group('AppShell state persistence', () {
    testWidgets('renders tab content through IndexedStack', (tester) async {
      await tester.pumpWidget(const MaterialApp(home: AppShell()));

      expect(find.byType(IndexedStack), findsOneWidget);
    });

    testWidgets('keeps previously visited screen mounted after tab switch', (
      tester,
    ) async {
      await tester.pumpWidget(const MaterialApp(home: AppShell()));
      await tester.pump(const Duration(milliseconds: 200));

      final initialRail = tester.widget<NavigationRail>(
        find.byType(NavigationRail),
      );
      expect(initialRail.selectedIndex, 0);

      expect(find.byType(HomeScreen, skipOffstage: false), findsOneWidget);

      initialRail.onDestinationSelected?.call(1);
      await tester.pump(const Duration(milliseconds: 200));

      final switchedRail = tester.widget<NavigationRail>(
        find.byType(NavigationRail),
      );
      expect(switchedRail.selectedIndex, 1);

      expect(find.byType(TdpScreen, skipOffstage: false), findsOneWidget);
      expect(find.byType(HomeScreen, skipOffstage: false), findsOneWidget);
      expect(find.byType(GpuScreen, skipOffstage: false), findsOneWidget);
    });

    testWidgets('opens XG Mobile diagnostic from navigation rail', (
      tester,
    ) async {
      await tester.pumpWidget(const MaterialApp(home: AppShell()));
      await tester.pump(const Duration(milliseconds: 200));

      expect(find.text('XG Diag'), findsOneWidget);

      final rail = tester.widget<NavigationRail>(find.byType(NavigationRail));
      rail.onDestinationSelected?.call(5);
      await tester.pump(const Duration(milliseconds: 200));

      final switchedRail = tester.widget<NavigationRail>(
        find.byType(NavigationRail),
      );
      expect(switchedRail.selectedIndex, 5);
      expect(
        find.byType(XgMobileFanDiagnosticScreen, skipOffstage: false),
        findsOneWidget,
      );
    });

    testWidgets('opens profiles page from navigation rail', (tester) async {
      await tester.pumpWidget(const MaterialApp(home: AppShell()));
      await tester.pump(const Duration(milliseconds: 200));

      expect(find.text('Profiles'), findsOneWidget);

      final rail = tester.widget<NavigationRail>(find.byType(NavigationRail));
      rail.onDestinationSelected?.call(7);
      await tester.pump(const Duration(milliseconds: 200));

      final switchedRail = tester.widget<NavigationRail>(
        find.byType(NavigationRail),
      );
      expect(switchedRail.selectedIndex, 7);
      expect(find.byType(ProfilesScreen, skipOffstage: false), findsOneWidget);
    });

    testWidgets(
      'applying a profile notifies shell to reload hardware screens',
      (tester) async {
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
        var reloads = 0;

        await tester.pumpWidget(
          MaterialApp(
            home: AppShell(
              settingsStore: settings,
              profileStore: profiles,
              onProfileReloadForTest: () => reloads += 1,
            ),
          ),
        );
        await tester.pump(const Duration(milliseconds: 200));

        final rail = tester.widget<NavigationRail>(find.byType(NavigationRail));
        rail.onDestinationSelected?.call(7);
        await tester.pump(const Duration(milliseconds: 200));
        final applyButton = tester.widget<TextButton>(
          find.byKey(const Key('apply_profile_gaming')),
        );
        applyButton.onPressed?.call();
        await tester.pump(const Duration(milliseconds: 200));

        expect(settings.current.ui.activeHardwareProfileId, 'gaming');
        expect(reloads, 1);
      },
    );

    testWidgets('loads selected navigation index from settings on startup', (
      tester,
    ) async {
      final store = MemoryAppSettingsStore(
        const AppSettings(ui: UiSettings(selectedNavigationIndex: 2)),
      );

      await tester.pumpWidget(
        MaterialApp(home: AppShell(settingsStore: store)),
      );
      await tester.pump(const Duration(milliseconds: 200));

      final rail = tester.widget<NavigationRail>(find.byType(NavigationRail));
      expect(rail.selectedIndex, 2);
    });

    testWidgets('saves selected navigation index when destination changes', (
      tester,
    ) async {
      final store = MemoryAppSettingsStore(AppSettings.defaults);

      await tester.pumpWidget(
        MaterialApp(home: AppShell(settingsStore: store)),
      );
      await tester.pump(const Duration(milliseconds: 200));

      final rail = tester.widget<NavigationRail>(find.byType(NavigationRail));
      rail.onDestinationSelected?.call(4);
      await tester.pump(const Duration(milliseconds: 200));

      expect(store.savedSettings.last.ui.selectedNavigationIndex, 4);
    });
  });
}
