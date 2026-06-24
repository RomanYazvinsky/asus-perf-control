import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mocktail/mocktail.dart';

import 'package:flutter_application_1/blocs/cpu_boost/cpu_boost_bloc.dart';
import 'package:flutter_application_1/blocs/cpu_boost/cpu_boost_event.dart';
import 'package:flutter_application_1/services/cpu_boost_service.dart';
import 'package:flutter_application_1/blocs/performance/performance_bloc.dart';
import 'package:flutter_application_1/blocs/performance/performance_event.dart';
import 'package:flutter_application_1/screens/home_screen.dart';

import '../mocks.dart';

class _FakeCpuBoostService implements ICpuBoostService {
  @override
  Future<CpuBoostStatus> getCpuBoostStatus() async {
    return const CpuBoostStatus(
      isSupported: true,
      currentMode: CpuBoostMode.enabled,
      boostEnabled: true,
      availableModes: CpuBoostMode.values,
    );
  }

  @override
  Future<void> setCpuBoostMode(CpuBoostMode mode) async {}
}

Widget _buildHome(PerformanceBloc bloc, {CpuBoostBloc? cpuBoostBloc}) =>
    MaterialApp(
      home: HomeScreen(
        testBloc: bloc,
        testCpuBoostBloc:
            cpuBoostBloc ?? CpuBoostBloc(service: _FakeCpuBoostService()),
      ),
    );

Future<void> _pumpUi(WidgetTester tester) async {
  await tester.pump();
  await tester.pump(const Duration(milliseconds: 300));
}

void main() {
  group('HomeScreen Widget Tests', () {
    late MockPerformanceService mockService;
    late MockCpuBoostService mockCpuBoostService;

    setUp(() {
      mockService = MockPerformanceService();
      when(() => mockService.getPerformanceMode()).thenAnswer((_) async => 1);
      when(
        () => mockService.setPerformanceMode(any()),
      ).thenAnswer((_) async {});
      mockCpuBoostService = MockCpuBoostService();
      when(() => mockCpuBoostService.getCpuBoostStatus()).thenAnswer(
        (_) async => const CpuBoostStatus(
          isSupported: true,
          currentMode: CpuBoostMode.efficientAggressive,
          boostEnabled: true,
          epp: 'balance_performance',
          availableModes: CpuBoostMode.values,
        ),
      );
      when(
        () => mockCpuBoostService.setCpuBoostMode(CpuBoostMode.disabled),
      ).thenAnswer((_) async {});
    });

    PerformanceBloc buildBloc() =>
        PerformanceBloc(service: mockService)..add(LoadCurrentModeEvent());

    CpuBoostBloc buildCpuBoostBloc() =>
        CpuBoostBloc(service: mockCpuBoostService)..add(LoadCpuBoostStatus());

    testWidgets('displays app title and subtitle', (tester) async {
      final bloc = buildBloc();

      await tester.pumpWidget(_buildHome(bloc));
      await _pumpUi(tester);

      expect(find.text('ASUS Linux Control Tool'), findsOneWidget);
      expect(find.text('MVP: Performance Mode Control'), findsOneWidget);
    });

    testWidgets('does not show debug alert testing controls', (tester) async {
      final bloc = buildBloc();

      await tester.pumpWidget(_buildHome(bloc));
      await _pumpUi(tester);

      expect(find.text('Тестирование алертов'), findsNothing);
      expect(find.text('Тест ошибки'), findsNothing);
      expect(find.text('Тест успеха'), findsNothing);
      expect(find.text('Тест info'), findsNothing);
    });

    testWidgets('shows Balanced on start', (tester) async {
      final bloc = buildBloc();

      await tester.pumpWidget(_buildHome(bloc));
      await _pumpUi(tester);

      // 'Balanced' appears in status card AND in mode button → findsWidgets is correct here
      expect(find.text('Balanced'), findsWidgets);
    });

    testWidgets('displays Current Mode status card', (tester) async {
      final bloc = buildBloc();

      await tester.pumpWidget(_buildHome(bloc));
      await _pumpUi(tester);

      expect(find.text('Current Mode'), findsOneWidget);
      expect(find.byIcon(Icons.info_outline), findsOneWidget);
    });

    testWidgets('displays all three mode buttons', (tester) async {
      final bloc = buildBloc();

      await tester.pumpWidget(_buildHome(bloc));
      await _pumpUi(tester);

      expect(find.text('Silent'), findsOneWidget);
      expect(find.text('Turbo'), findsOneWidget);
      // Balanced: button + status card = 2 occurrences
      expect(find.text('Balanced'), findsWidgets);
    });

    testWidgets('displays mode icons', (tester) async {
      final bloc = buildBloc();

      await tester.pumpWidget(_buildHome(bloc));
      await _pumpUi(tester);

      expect(find.byIcon(Icons.volume_off), findsOneWidget);
      expect(find.byIcon(Icons.balance), findsOneWidget);
      expect(find.byIcon(Icons.speed), findsOneWidget);
    });

    testWidgets('Balanced is highlighted with check_circle by default', (
      tester,
    ) async {
      final bloc = buildBloc();

      await tester.pumpWidget(_buildHome(bloc));
      await _pumpUi(tester);

      expect(find.byIcon(Icons.check_circle), findsOneWidget);
    });

    testWidgets('switching to Silent highlights Silent mode', (tester) async {
      final bloc = buildBloc();

      await tester.pumpWidget(_buildHome(bloc));
      await _pumpUi(tester);

      await tester.tap(find.text('Silent'));
      await tester.pump();

      await _pumpUi(tester);

      // check_circle must appear exactly once — at Silent button
      expect(find.byIcon(Icons.check_circle), findsOneWidget);
      // Balanced text in status card is now 'Silent'
      expect(find.text('Silent'), findsWidgets);
    });

    testWidgets('switching mode does not leave loading indicator visible', (
      tester,
    ) async {
      final bloc = buildBloc();

      await tester.pumpWidget(_buildHome(bloc));
      await _pumpUi(tester);

      await tester.tap(find.text('Turbo'));
      await _pumpUi(tester);

      expect(find.byType(CircularProgressIndicator), findsNothing);
    });

    testWidgets('Turbo highlighted after switching to Turbo', (tester) async {
      final bloc = buildBloc();

      await tester.pumpWidget(_buildHome(bloc));
      await _pumpUi(tester);

      await tester.tap(find.text('Turbo'));
      await _pumpUi(tester);

      expect(find.byIcon(Icons.check_circle), findsOneWidget);
      expect(find.text('Turbo'), findsWidgets);
    });

    testWidgets('maintains correct state after multiple switches', (
      tester,
    ) async {
      final bloc = buildBloc();

      await tester.pumpWidget(_buildHome(bloc));
      await _pumpUi(tester);

      await tester.tap(find.text('Turbo'));
      await _pumpUi(tester);

      await tester.tap(find.text('Silent'));
      await _pumpUi(tester);

      await tester.tap(find.text('Turbo'));
      await _pumpUi(tester);

      expect(find.byIcon(Icons.check_circle), findsOneWidget);
      expect(find.text('Turbo'), findsWidgets);
    });

    testWidgets('renders all supported CPU boost modes', (tester) async {
      final bloc = buildBloc();
      final cpuBoostBloc = buildCpuBoostBloc();

      await tester.pumpWidget(_buildHome(bloc, cpuBoostBloc: cpuBoostBloc));
      await _pumpUi(tester);

      expect(find.text('CPU Boost Mode'), findsOneWidget);
      expect(find.text('Disabled'), findsOneWidget);
      expect(find.text('Efficient'), findsOneWidget);
      expect(find.text('Efficient Enabled'), findsOneWidget);
      expect(find.text('Enabled'), findsOneWidget);
      expect(find.text('Efficient Aggressive'), findsOneWidget);
      expect(find.text('Aggressive'), findsOneWidget);
    });

    testWidgets('current CPU boost mode is selected', (tester) async {
      final bloc = buildBloc();
      final cpuBoostBloc = buildCpuBoostBloc();

      await tester.pumpWidget(_buildHome(bloc, cpuBoostBloc: cpuBoostBloc));
      await _pumpUi(tester);

      expect(find.text('Responsive efficient boost'), findsOneWidget);
      expect(find.byIcon(Icons.check_circle), findsWidgets);
    });

    testWidgets('tapping Disabled dispatches CPU boost mode change', (
      tester,
    ) async {
      final bloc = buildBloc();
      final cpuBoostBloc = buildCpuBoostBloc();

      await tester.pumpWidget(_buildHome(bloc, cpuBoostBloc: cpuBoostBloc));
      await _pumpUi(tester);

      await tester.scrollUntilVisible(find.text('Disabled'), 300);
      await tester.tap(find.text('Disabled'));
      await _pumpUi(tester);

      verify(
        () => mockCpuBoostService.setCpuBoostMode(CpuBoostMode.disabled),
      ).called(1);
    });

    testWidgets('unsupported CPU boost status shows unavailable message', (
      tester,
    ) async {
      when(() => mockCpuBoostService.getCpuBoostStatus()).thenAnswer(
        (_) async => const CpuBoostStatus(
          isSupported: false,
          currentMode: CpuBoostMode.enabled,
          boostEnabled: false,
          availableModes: [],
        ),
      );
      final bloc = buildBloc();
      final cpuBoostBloc = buildCpuBoostBloc();

      await tester.pumpWidget(_buildHome(bloc, cpuBoostBloc: cpuBoostBloc));
      await _pumpUi(tester);

      expect(find.text('CPU boost controls are unavailable'), findsOneWidget);
    });
  });
}
