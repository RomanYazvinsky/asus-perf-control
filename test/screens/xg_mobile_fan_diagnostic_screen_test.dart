import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import 'package:flutter_application_1/services/asusd_fan_client.dart';
import 'package:flutter_application_1/services/xg_mobile_fan_diagnostic_service.dart';
import 'package:flutter_application_1/screens/xg_mobile_fan_diagnostic_screen.dart';

class _FakeCurvePort implements XgFanCurvePort {
  List<int>? cpuReadbackPwm;

  @override
  Future<List<FanCurveData>> getBalancedCurves() async {
    return getCurves(PerfProfile.balanced);
  }

  @override
  Future<List<FanCurveData>> getCurves(PerfProfile profile) async {
    return [
      FanCurveData(
        name: 'CPU',
        pwmValues: cpuReadbackPwm ?? [20, 43, 66, 81, 117, 193, 193, 193],
        tempValues: [53, 58, 62, 65, 68, 73, 73, 73],
        enabled: false,
      ),
      FanCurveData(
        name: 'GPU',
        pwmValues: [15, 45, 66, 94, 155, 173, 173, 173],
        tempValues: [53, 58, 62, 65, 68, 73, 73, 73],
        enabled: false,
      ),
    ];
  }

  @override
  Future<void> restoreBalancedDefaults() async {}

  @override
  Future<void> restoreDefaults(PerfProfile profile) async {}

  @override
  Future<void> setBalancedCpuFlatPwm(int pwm) async {}

  @override
  Future<void> setBalancedGpuFlatPwm(int pwm) async {}

  @override
  Future<void> setCustomCurve(
    PerfProfile profile,
    String fanName,
    List<int> pwmValues, {
    List<int>? tempValues,
  }) async {
    if (fanName == 'CPU') {
      cpuReadbackPwm = List<int>.of(pwmValues);
    }
  }

  @override
  Future<void> setFlatPwm(PerfProfile profile, String fanName, int pwm) async {}
}

class _FakeProbe implements XgFanHwmonProbe {
  @override
  Future<XgFanHwmonSnapshot> readSnapshot() async {
    return const XgFanHwmonSnapshot(
      fan1Rpm: 3600,
      fan2Rpm: 3300,
      pwm1Enable: 2,
      pwm2Enable: 1,
      pwm2Point8: 220,
      cpuTempMilliC: 56000,
      gpuTempMilliC: 62000,
      labels: {'fan1': 'cpu_fan', 'fan2': 'gpu_fan'},
    );
  }
}

class _MemoryStore implements XgFanDiagnosticStore {
  XgFanDiagnosticSession session;
  int saveCount = 0;

  _MemoryStore([this.session = const XgFanDiagnosticSession()]);

  @override
  Future<void> clear() async {
    session = const XgFanDiagnosticSession();
  }

  @override
  Future<XgFanDiagnosticSession> load() async => session;

  @override
  Future<void> save(XgFanDiagnosticSession session) async {
    this.session = session;
    saveCount += 1;
  }
}

Widget _buildScreen({_MemoryStore? store}) {
  return MaterialApp(
    home: XgMobileFanDiagnosticScreen(
      service: XgMobileFanDiagnosticService(
        curvePort: _FakeCurvePort(),
        probe: _FakeProbe(),
        sampleInterval: Duration.zero,
        samplesPerPwm: 1,
        sweepPwmValues: const [180],
      ),
      store: store ?? _MemoryStore(),
    ),
  );
}

XgFanDiagnosticRun _savedRun() {
  return XgFanDiagnosticRun(
    label: 'Current State',
    startedAt: DateTime.utc(2026, 5, 21, 10),
    finishedAt: DateTime.utc(2026, 5, 21, 10, 1),
    baselineCurves: const [],
    baselineSnapshot: const XgFanHwmonSnapshot(
      fan1Rpm: 3600,
      fan2Rpm: 3300,
      pwm1Enable: 2,
      pwm2Enable: 2,
      pwm2Point8: 173,
      cpuTempMilliC: 56000,
      gpuTempMilliC: 62000,
      labels: {'fan1': 'cpu_fan', 'fan2': 'gpu_fan'},
    ),
    steps: const [
      XgFanSweepStep(
        requestedPwm: 180,
        samples: [
          XgFanHwmonSnapshot(
            fan1Rpm: 3600,
            fan2Rpm: 3300,
            pwm1Enable: 2,
            pwm2Enable: 1,
            pwm2Point8: 180,
            cpuTempMilliC: 56000,
            gpuTempMilliC: 62000,
            labels: {'fan1': 'cpu_fan', 'fan2': 'gpu_fan'},
          ),
        ],
      ),
    ],
  );
}

void main() {
  testWidgets('renders separate XG Mobile diagnostic screen', (tester) async {
    await tester.pumpWidget(_buildScreen());

    expect(find.text('XG Mobile Diagnostic'), findsOneWidget);
    expect(find.text('Run Current State Sweep'), findsOneWidget);
    expect(find.text('Run Changed State Sweep'), findsOneWidget);
    expect(find.text('Run CPU/GPU Coupling'), findsOneWidget);
    expect(find.text('Run Profile Bounds'), findsOneWidget);
    expect(find.text('Run Curve Rule Discovery'), findsOneWidget);
  });

  testWidgets('shows every curve rule case after discovery', (tester) async {
    await tester.pumpWidget(_buildScreen());

    await tester.tap(find.text('Run Curve Rule Discovery'));
    await tester.pump();
    await tester.pump();

    expect(find.text('Curve Rule Discovery'), findsOneWidget);
    expect(
      find.text(
        'default: temp 56C, PWM default=34 requested=34 readback=34, '
        'CPU RPM 3600->3600 delta=0',
      ),
      findsOneWidget,
    );
    expect(
      find.textContaining('flat_below_default_small_delta: temp 56C'),
      findsOneWidget,
    );
    expect(find.textContaining('zero_until_80c: temp 56C'), findsOneWidget);
    expect(
      find.textContaining('below_default_until_80c: temp 56C'),
      findsOneWidget,
    );
  });

  testWidgets('runs current-state sweep and explains accepted no-response', (
    tester,
  ) async {
    await tester.pumpWidget(_buildScreen());

    await tester.tap(find.text('Run Current State Sweep'));
    await tester.pump();
    await tester.pump();

    expect(find.text('Current State'), findsOneWidget);
    expect(find.text('Accepted, no RPM response'), findsOneWidget);
    expect(find.textContaining('curve was accepted'), findsOneWidget);
  });

  testWidgets('loads saved current run after app restart', (tester) async {
    await tester.pumpWidget(
      _buildScreen(
        store: _MemoryStore(XgFanDiagnosticSession(currentRun: _savedRun())),
      ),
    );
    await tester.pump();

    expect(find.text('Baseline fan2: 3300'), findsOneWidget);
    expect(find.text('Accepted, no RPM response'), findsOneWidget);
  });

  testWidgets('saves current run for comparison after reboot', (tester) async {
    final store = _MemoryStore();
    await tester.pumpWidget(_buildScreen(store: store));

    await tester.tap(find.text('Run Current State Sweep'));
    await tester.pump();
    await tester.pump();

    expect(store.saveCount, 1);
    expect(store.session.currentRun, isNotNull);
    expect(store.session.changedRun, isNull);
  });
}
