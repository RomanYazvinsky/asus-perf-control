import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:flutter_application_1/main.dart';
import 'package:flutter_application_1/screens/home_screen.dart';
import 'package:flutter_application_1/screens/tdp_screen.dart';
import 'package:flutter_application_1/screens/gpu_screen.dart';
import 'package:flutter_application_1/screens/xg_mobile_fan_diagnostic_screen.dart';

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
  });
}
