import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import 'package:flutter_application_1/blocs/fan/fan_bloc.dart';
import 'package:flutter_application_1/blocs/fan/fan_event.dart';
import 'package:flutter_application_1/blocs/fan/fan_state.dart';
import 'package:flutter_application_1/screens/fan_screen.dart';
import 'package:flutter_application_1/services/asusd_fan_client.dart';
import 'package:flutter_application_1/services/fan_service.dart';
import 'package:flutter_application_1/widgets/fan/fan_curve_editor.dart';

class _NoopFanService implements IFanService {
  @override
  Future<FanCapabilities> getCapabilities() async => FanCapabilities(
    fans: [FanInfo(id: 'cpu', label: 'CPU Fan', hasRpm: true, hasCurve: true)],
  );

  @override
  Future<List<FanReading>> getFanRpms() async => [
    FanReading(id: 'cpu', rpm: 2100),
  ];

  @override
  Future<FanTemperatures> getTemperatures() async =>
      const FanTemperatures(cpuTempC: 45, gpuTempC: 0);

  @override
  Future<List<FanCurveData>> getAsusdCurves(PerfProfile profile) async => [];

  @override
  Future<void> resetToDefaults(PerfProfile profile) async {}

  @override
  Future<int> runMaxSpeedTest(PerfProfile profile, FanId fan) async => 0;

  @override
  Future<void> setAsusdCurve(
    PerfProfile profile,
    FanId fan,
    List<FanCurvePoint> points,
  ) async {}

  @override
  Future<void> setConstantSpeed(
    PerfProfile profile,
    FanId fan,
    int speedPercent,
  ) async {}

  @override
  Future<void> startDynamicEngine(FanEngineConfig config) async {}

  @override
  Future<void> stopDynamicEngine() async {}

  @override
  Future<void> updateDynamicCurve(FanId fan, List<FanCurvePoint> curve) async {}

  @override
  Future<bool> isDynamicEngineRunning() async => false;

  @override
  Future<void> setActiveProfile(PerfProfile profile) async {}
}

class _PresetFanBloc extends FanBloc {
  final FanState _preset;
  final List<FanEvent> recordedEvents = [];

  _PresetFanBloc(this._preset) : super(_NoopFanService());

  @override
  FanState get state => _preset;

  @override
  void add(FanEvent event) {
    recordedEvents.add(event);
  }
}

FanLoaded _loadedState({
  FanControlMode cpuMode = FanControlMode.asusctlCurve,
  bool dynamicEngineRunning = false,
  PerfProfile selectedProfile = PerfProfile.balanced,
  FanProfileCurves? asusdCurves,
}) {
  return FanLoaded(
    capabilities: FanCapabilities(
      fans: [
        FanInfo(id: 'cpu', label: 'CPU Fan', hasRpm: true, hasCurve: true),
      ],
    ),
    fanRpms: const {FanId.cpu: 2200},
    asusdCurves:
        asusdCurves ??
        {
          FanId.cpu: {
            PerfProfile.balanced: FanCurveData(
              name: 'CPU',
              pwmValues: [90, 100, 110, 120, 130, 140, 150, 160],
              tempValues: [30, 40, 50, 60, 70, 80, 90, 100],
              enabled: true,
            ),
          },
        },
    selectedFan: FanId.cpu,
    selectedProfile: selectedProfile,
    controlModes: {FanId.cpu: cpuMode},
    dynamicEngineRunning: dynamicEngineRunning,
  );
}

void main() {
  group('FanScreen widget regressions', () {
    testWidgets('does not render mode selector with TODO handlers', (
      tester,
    ) async {
      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: FanTabContent(
              fanId: FanId.cpu,
              state: _loadedState(cpuMode: FanControlMode.asusctlCurve),
            ),
          ),
        ),
      );
      await tester.pump(const Duration(milliseconds: 100));

      expect(find.text('Control Mode'), findsNothing);
      expect(find.text('Constant Speed'), findsNothing);
      expect(find.text('Dynamic Engine'), findsNothing);
    });

    testWidgets('does not render constant-speed placeholder slider', (
      tester,
    ) async {
      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: FanTabContent(
              fanId: FanId.cpu,
              state: _loadedState(cpuMode: FanControlMode.constantSpeed),
            ),
          ),
        ),
      );
      await tester.pump(const Duration(milliseconds: 100));

      expect(find.byType(Slider), findsNothing);
      expect(
        find.text('Constant speed control is not available in this build.'),
        findsOneWidget,
      );
    });

    testWidgets('keeps PWM curve visible in dynamic engine mode', (
      tester,
    ) async {
      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: FanTabContent(
              fanId: FanId.cpu,
              state: _loadedState(cpuMode: FanControlMode.dynamicEngine),
            ),
          ),
        ),
      );
      await tester.pump(const Duration(milliseconds: 100));

      expect(find.text('Fan Curve (8 points)'), findsOneWidget);
      expect(find.byType(Slider), findsNothing);
    });

    testWidgets('dynamic engine curve is editable and applies dynamic update', (
      tester,
    ) async {
      final bloc = _PresetFanBloc(
        _loadedState(
          cpuMode: FanControlMode.dynamicEngine,
          dynamicEngineRunning: true,
        ),
      );

      await tester.pumpWidget(
        MaterialApp(
          home: BlocProvider<FanBloc>.value(
            value: bloc,
            child: Scaffold(
              body: FanTabContent(
                fanId: FanId.cpu,
                state: bloc.state as FanLoaded,
              ),
            ),
          ),
        ),
      );
      await tester.pump(const Duration(milliseconds: 100));

      final editor = tester.widget<FanCurveEditor>(find.byType(FanCurveEditor));
      expect(editor.readOnly, isFalse);
      expect(find.widgetWithText(ElevatedButton, 'Apply'), findsOneWidget);

      await tester.tap(find.widgetWithText(ElevatedButton, 'Apply'));
      await tester.pump();

      expect(bloc.recordedEvents, hasLength(1));
      expect(bloc.recordedEvents.single, isA<UpdateDynamicCurve>());
    });

    testWidgets('uses selected profile curve without relying on list index', (
      tester,
    ) async {
      final performanceCurve = FanCurveData(
        name: 'CPU',
        pwmValues: [204, 204, 204, 204, 204, 204, 204, 204],
        tempValues: [35, 45, 55, 65, 75, 85, 95, 100],
        enabled: true,
      );

      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: FanTabContent(
              fanId: FanId.cpu,
              state: _loadedState(
                selectedProfile: PerfProfile.performance,
                asusdCurves: {
                  FanId.cpu: {PerfProfile.performance: performanceCurve},
                },
              ),
            ),
          ),
        ),
      );
      await tester.pump(const Duration(milliseconds: 100));

      final editor = tester.widget<FanCurveEditor>(find.byType(FanCurveEditor));
      expect(editor.curve.first.tempC, 35);
      expect(editor.curve.first.speedPercent, 80);
    });

    testWidgets('renders max-speed result state explicitly', (tester) async {
      final previous = _loadedState();
      final bloc = _PresetFanBloc(
        FanMaxSpeedResult(previous: previous, fanId: FanId.cpu, maxRpm: 4500),
      );

      await tester.pumpWidget(
        MaterialApp(
          home: BlocProvider<FanBloc>.value(
            value: bloc,
            child: const FanView(),
          ),
        ),
      );
      await tester.pump(const Duration(milliseconds: 100));

      expect(find.text('Max Speed Test Complete'), findsOneWidget);
      expect(find.textContaining('4500 RPM'), findsOneWidget);
    });

    testWidgets('dynamic engine switch value follows bloc state', (
      tester,
    ) async {
      final bloc = _PresetFanBloc(_loadedState(dynamicEngineRunning: true));

      await tester.pumpWidget(
        MaterialApp(
          home: BlocProvider<FanBloc>.value(
            value: bloc,
            child: const FanView(),
          ),
        ),
      );
      await tester.pump(const Duration(milliseconds: 100));

      final dynamicSwitch = tester.widget<Switch>(find.byType(Switch));
      expect(dynamicSwitch.value, isTrue);
    });

    testWidgets('renders profile tab bar and dispatches profile switch', (
      tester,
    ) async {
      final bloc = _PresetFanBloc(_loadedState());

      await tester.pumpWidget(
        MaterialApp(
          home: BlocProvider<FanBloc>.value(
            value: bloc,
            child: const FanView(),
          ),
        ),
      );
      await tester.pump(const Duration(milliseconds: 100));

      expect(find.byKey(const Key('profile_tab_bar')), findsOneWidget);
      await tester.tap(find.text('Turbo'));
      await tester.pump();

      final event = bloc.recordedEvents.single as SelectProfile;
      expect(event.profile, PerfProfile.performance);
    });

    testWidgets('dynamic engine switch starts with current CPU curve', (
      tester,
    ) async {
      final cpuCurve = FanCurveData(
        name: 'CPU',
        pwmValues: [158, 158, 158, 158, 158, 158, 158, 158],
        tempValues: [30, 40, 50, 60, 70, 80, 90, 100],
        enabled: true,
      );
      final bloc = _PresetFanBloc(
        _loadedState(
          asusdCurves: {
            FanId.cpu: {PerfProfile.balanced: cpuCurve},
          },
        ),
      );

      await tester.pumpWidget(
        MaterialApp(
          home: BlocProvider<FanBloc>.value(
            value: bloc,
            child: const FanView(),
          ),
        ),
      );
      await tester.pump(const Duration(milliseconds: 100));

      await tester.tap(find.byType(Switch));
      await tester.pump();

      final event = bloc.recordedEvents.single as StartDynamicEngine;
      expect(event.config.cpuCurve.first.speedPercent, 62);
    });

    testWidgets('does not poll FRB temperatures directly from widget', (
      tester,
    ) async {
      final prints = <String>[];

      await runZoned(
        () async {
          final bloc = _PresetFanBloc(_loadedState());
          await tester.pumpWidget(
            MaterialApp(
              home: BlocProvider<FanBloc>.value(
                value: bloc,
                child: const FanView(),
              ),
            ),
          );
          await tester.pump(const Duration(milliseconds: 100));
        },
        zoneSpecification: ZoneSpecification(
          print: (self, parent, zone, message) => prints.add(message),
        ),
      );

      expect(
        prints.where((message) => message.contains('flutter_rust_bridge')),
        isEmpty,
      );
    });
  });
}
