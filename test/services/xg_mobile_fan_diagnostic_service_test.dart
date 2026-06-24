import 'package:flutter_test/flutter_test.dart';

import 'package:flutter_application_1/services/asusd_fan_client.dart';
import 'package:flutter_application_1/services/xg_mobile_fan_diagnostic_service.dart';

class _FakeCurvePort implements XgFanCurvePort {
  final List<int> writtenPwm = [];
  final List<List<int>> writtenCurves = [];
  final List<int> defaultCpuPwm;
  final List<int> defaultCpuTemps;
  bool throwOnWrite = false;
  int restoreCount = 0;
  List<int>? cpuReadbackPwm;

  _FakeCurvePort({
    this.defaultCpuPwm = const [20, 43, 66, 81, 117, 193, 193, 193],
    this.defaultCpuTemps = const [53, 58, 62, 65, 68, 73, 73, 73],
  });

  @override
  Future<List<FanCurveData>> getBalancedCurves() async {
    return getCurves(PerfProfile.balanced);
  }

  @override
  Future<List<FanCurveData>> getCurves(PerfProfile profile) async {
    return [
      FanCurveData(
        name: 'CPU',
        pwmValues: cpuReadbackPwm ?? defaultCpuPwm,
        tempValues: defaultCpuTemps,
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
  Future<void> setBalancedGpuFlatPwm(int pwm) async {
    await setFlatPwm(PerfProfile.balanced, 'GPU', pwm);
  }

  @override
  Future<void> setBalancedCpuFlatPwm(int pwm) async {
    await setFlatPwm(PerfProfile.balanced, 'CPU', pwm);
  }

  @override
  Future<void> setFlatPwm(PerfProfile profile, String fanName, int pwm) async {
    if (throwOnWrite) {
      throw StateError('write failed');
    }
    writtenPwm.add(pwm);
  }

  @override
  Future<void> setCustomCurve(
    PerfProfile profile,
    String fanName,
    List<int> pwmValues, {
    List<int>? tempValues,
  }) async {
    if (throwOnWrite) {
      throw StateError('write failed');
    }
    writtenPwm.addAll(pwmValues);
    writtenCurves.add(List<int>.of(pwmValues));
    cpuReadbackPwm = List<int>.of(pwmValues);
  }

  @override
  Future<void> restoreDefaults(PerfProfile profile) async {
    restoreCount += 1;
  }

  @override
  Future<void> restoreBalancedDefaults() async {
    await restoreDefaults(PerfProfile.balanced);
  }
}

class _FakePlatformPort implements XgPlatformProfilePort {
  PerfProfile activeProfile = PerfProfile.balanced;

  @override
  Future<PerfProfile> getActiveProfile() async => activeProfile;

  @override
  Future<void> setActiveProfile(PerfProfile profile) async {
    activeProfile = profile;
  }
}

class _FakeProbe implements XgFanHwmonProbe {
  final List<XgFanHwmonSnapshot> snapshots;
  int _index = 0;

  _FakeProbe(this.snapshots);

  @override
  Future<XgFanHwmonSnapshot> readSnapshot() async {
    final snapshot = snapshots[_index.clamp(0, snapshots.length - 1)];
    _index += 1;
    return snapshot;
  }
}

XgFanHwmonSnapshot _snapshot({
  int? fan1Rpm = 3600,
  int? fan2Rpm = 3300,
  int pwm2Enable = 1,
  int pwm2Point8 = 220,
  int cpuTempMilliC = 56000,
  int gpuTempMilliC = 62000,
}) {
  return XgFanHwmonSnapshot(
    fan1Rpm: fan1Rpm,
    fan2Rpm: fan2Rpm,
    pwm1Enable: 2,
    pwm2Enable: pwm2Enable,
    pwm2Point8: pwm2Point8,
    cpuTempMilliC: cpuTempMilliC,
    gpuTempMilliC: gpuTempMilliC,
    labels: const {'fan1': 'cpu_fan', 'fan2': 'gpu_fan'},
  );
}

void main() {
  group('XgMobileFanDiagnosticService', () {
    test('restores balanced defaults after successful sweep', () async {
      final curves = _FakeCurvePort();
      final service = XgMobileFanDiagnosticService(
        curvePort: curves,
        platformPort: _FakePlatformPort(),
        probe: _FakeProbe(List.filled(8, _snapshot())),
        sampleInterval: Duration.zero,
        samplesPerPwm: 2,
        sweepPwmValues: const [180, 220],
      );

      final run = await service.runSweep(label: 'current');

      expect(curves.writtenPwm, [180, 220]);
      expect(curves.restoreCount, 1);
      expect(run.steps, hasLength(2));
    });

    test('restores balanced defaults after write failure', () async {
      final curves = _FakeCurvePort()..throwOnWrite = true;
      final service = XgMobileFanDiagnosticService(
        curvePort: curves,
        platformPort: _FakePlatformPort(),
        probe: _FakeProbe([_snapshot()]),
        sampleInterval: Duration.zero,
        samplesPerPwm: 1,
        sweepPwmValues: const [180],
      );

      await expectLater(
        service.runSweep(label: 'current'),
        throwsA(isA<StateError>()),
      );

      expect(curves.restoreCount, 1);
    });

    test(
      'classifies accepted curve with unchanged RPM as acceptedNoResponse',
      () {
        final step = XgFanSweepStep(
          requestedPwm: 220,
          samples: [
            _snapshot(pwm2Point8: 220),
            _snapshot(fan2Rpm: 3400, pwm2Point8: 220),
          ],
        );

        expect(step.classification, XgFanStepClassification.acceptedNoResponse);
        expect(step.fan2MinRpm, 3300);
        expect(step.fan2MaxRpm, 3400);
        expect(step.fan2DeltaRpm, 100);
      },
    );

    test('classifies changed routing when only one run responds', () {
      final noResponse = XgFanDiagnosticRun(
        label: 'connected',
        startedAt: DateTime(2026, 5, 21),
        finishedAt: DateTime(2026, 5, 21),
        baselineCurves: const [],
        baselineSnapshot: _snapshot(),
        steps: [
          XgFanSweepStep(
            requestedPwm: 220,
            samples: [_snapshot(), _snapshot(fan2Rpm: 3400)],
          ),
        ],
      );
      final responded = XgFanDiagnosticRun(
        label: 'disconnected',
        startedAt: DateTime(2026, 5, 21),
        finishedAt: DateTime(2026, 5, 21),
        baselineCurves: const [],
        baselineSnapshot: _snapshot(),
        steps: [
          XgFanSweepStep(
            requestedPwm: 220,
            samples: [_snapshot(), _snapshot(fan2Rpm: 4300)],
          ),
        ],
      );

      final diagnosis = XgFanDiagnosis.compare(noResponse, responded);

      expect(diagnosis.kind, XgFanDiagnosisKind.routeChanged);
    });

    test('serializes diagnostic run so it can survive reboot', () {
      final run = XgFanDiagnosticRun(
        label: 'Current State',
        startedAt: DateTime.utc(2026, 5, 21, 10),
        finishedAt: DateTime.utc(2026, 5, 21, 10, 1),
        baselineCurves: [
          FanCurveData(
            name: 'GPU',
            pwmValues: [15, 45, 66, 94, 155, 173, 173, 173],
            tempValues: [53, 58, 62, 65, 68, 73, 73, 73],
            enabled: false,
          ),
        ],
        baselineSnapshot: _snapshot(),
        steps: [
          XgFanSweepStep(
            requestedPwm: 220,
            samples: [_snapshot(pwm2Point8: 220)],
          ),
        ],
      );

      final restored = XgFanDiagnosticRun.fromJson(run.toJson());

      expect(restored.label, 'Current State');
      expect(restored.startedAt, DateTime.utc(2026, 5, 21, 10));
      expect(restored.baselineCurves.single.name, 'GPU');
      expect(restored.steps.single.requestedPwm, 220);
      expect(restored.steps.single.samples.single.fan2Rpm, 3300);
    });

    test('classifies XG Mobile dependency when fan2 baseline changes', () {
      final first = XgFanDiagnosticRun(
        label: 'xg connected',
        startedAt: DateTime(2026, 5, 21),
        finishedAt: DateTime(2026, 5, 21),
        baselineCurves: const [],
        baselineSnapshot: _snapshot(fan2Rpm: 3200),
        steps: [
          XgFanSweepStep(
            requestedPwm: 220,
            samples: [_snapshot(fan2Rpm: 3200), _snapshot(fan2Rpm: 3200)],
          ),
        ],
      );
      final second = XgFanDiagnosticRun(
        label: 'xg disconnected',
        startedAt: DateTime(2026, 5, 21),
        finishedAt: DateTime(2026, 5, 21),
        baselineCurves: const [],
        baselineSnapshot: _snapshot(fan2Rpm: 4900),
        steps: [
          XgFanSweepStep(
            requestedPwm: 220,
            samples: [_snapshot(fan2Rpm: 4900), _snapshot(fan2Rpm: 5000)],
          ),
        ],
      );

      final diagnosis = XgFanDiagnosis.compare(first, second);

      expect(diagnosis.kind, XgFanDiagnosisKind.gpuDependsOnXgMobile);
    });

    test('classifies GPU fan as CPU-coupled when fan2 follows CPU sweep', () {
      final result = FanHypothesisDiagnosis.cpuGpuCoupling(
        cpuOnlySteps: [
          FanHypothesisStep(
            label: 'CPU 255',
            requestedFan: 'CPU',
            requestedPwm: 255,
            samples: [_snapshot(fan2Rpm: 3200), _snapshot(fan2Rpm: 4300)],
          ),
        ],
        gpuOnlySteps: [
          FanHypothesisStep(
            label: 'GPU 255',
            requestedFan: 'GPU',
            requestedPwm: 255,
            samples: [_snapshot(fan2Rpm: 3200), _snapshot(fan2Rpm: 3300)],
          ),
        ],
      );

      expect(result.kind, FanHypothesisKind.gpuDependsOnCpuFan);
    });

    test('classifies profile floor or cap from per-profile RPM ranges', () {
      final result = FanHypothesisDiagnosis.profileBounds([
        ProfileFanBoundary(
          profile: PerfProfile.quiet,
          lowPwmStep: FanHypothesisStep(
            label: 'Quiet 40',
            requestedFan: 'CPU',
            requestedPwm: 40,
            samples: [_snapshot(fan2Rpm: 3000), _snapshot(fan2Rpm: 3100)],
          ),
          highPwmStep: FanHypothesisStep(
            label: 'Quiet 255',
            requestedFan: 'CPU',
            requestedPwm: 255,
            samples: [_snapshot(fan2Rpm: 5000), _snapshot(fan2Rpm: 5200)],
          ),
        ),
        ProfileFanBoundary(
          profile: PerfProfile.performance,
          lowPwmStep: FanHypothesisStep(
            label: 'Performance 40',
            requestedFan: 'CPU',
            requestedPwm: 40,
            samples: [_snapshot(fan2Rpm: 4300), _snapshot(fan2Rpm: 4400)],
          ),
          highPwmStep: FanHypothesisStep(
            label: 'Performance 255',
            requestedFan: 'CPU',
            requestedPwm: 255,
            samples: [_snapshot(fan2Rpm: 7800), _snapshot(fan2Rpm: 8200)],
          ),
        ),
      ]);

      expect(result.kind, FanHypothesisKind.profileFloorOrCap);
    });

    test(
      'computes curve case PWM and RPM using CPU temperature interpolation',
      () {
        final observation = CurveRuleCaseObservation(
          label: 'low',
          defaultPwmValues: const [100, 200],
          requestedPwmValues: const [0, 255],
          readbackPwmValues: const [0, 255],
          tempValues: const [50, 100],
          before: _snapshot(fan2Rpm: 3200),
          samples: [
            _snapshot(
              fan2Rpm: 3300,
              cpuTempMilliC: 75000,
              gpuTempMilliC: 61000,
            ),
          ],
        );

        expect(observation.requestedPwmAtTemp!.round(), 128);
        expect(observation.expectedRequestedCpuRpm!.round(), 5000);
        expect(observation.readbackPwmAtTemp!.round(), 128);
        expect(observation.defaultPwmAtTemp!.round(), 150);
      },
    );

    test(
      'classifies below-default curve as runtime-clamped when readback stores it but CPU RPM does not drop',
      () {
        final result = FanHypothesisDiagnosis.curveRuleDiscovery(
          observations: [
            CurveRuleCaseObservation(
              label: 'flat_below_default_large_delta',
              defaultPwmValues: const [100, 180],
              requestedPwmValues: const [40, 80],
              readbackPwmValues: const [40, 80],
              tempValues: const [50, 70],
              before: _snapshot(fan1Rpm: 6000, cpuTempMilliC: 60000),
              samples: [_snapshot(fan1Rpm: 5900, cpuTempMilliC: 60000)],
            ),
          ],
        );

        expect(result.kind, FanHypothesisKind.downwardStoredButRuntimeClamped);
      },
    );

    test('classifies below-default curve as applied when CPU RPM drops', () {
      final result = FanHypothesisDiagnosis.curveRuleDiscovery(
        observations: [
          CurveRuleCaseObservation(
            label: 'current_bucket_below_default',
            defaultPwmValues: const [100, 180],
            requestedPwmValues: const [40, 80],
            readbackPwmValues: const [40, 80],
            tempValues: const [50, 70],
            before: _snapshot(fan1Rpm: 6000, cpuTempMilliC: 60000),
            samples: [_snapshot(fan1Rpm: 5200, cpuTempMilliC: 60000)],
          ),
        ],
      );

      expect(result.kind, FanHypothesisKind.downwardApplied);
    });

    test('includes curve rule numbers in below-default applied diagnosis', () {
      final result = FanHypothesisDiagnosis.curveRuleDiscovery(
        observations: [
          CurveRuleCaseObservation(
            label: 'flat_below_default_small_delta',
            defaultPwmValues: const [100, 180],
            requestedPwmValues: const [80, 160],
            readbackPwmValues: const [80, 160],
            tempValues: const [50, 70],
            before: _snapshot(fan1Rpm: 6000, cpuTempMilliC: 60000),
            samples: [_snapshot(fan1Rpm: 5400, cpuTempMilliC: 60000)],
          ),
        ],
      );

      expect(result.summary, contains('temp=60C'));
      expect(result.summary, contains('PWM default=140'));
      expect(result.summary, contains('requested=120'));
      expect(result.summary, contains('readback=120'));
      expect(result.summary, contains('CPU RPM 6000->5400'));
      expect(result.summary, contains('delta=-600'));
      expect(result.nextAction, contains('drop threshold=500 RPM'));
      expect(result.nextAction, contains('below-default threshold=20 PWM'));
    });

    test('prioritizes boundary curve case diagnosis over small flat delta', () {
      final result = FanHypothesisDiagnosis.curveRuleDiscovery(
        observations: [
          CurveRuleCaseObservation(
            label: 'flat_below_default_small_delta',
            defaultPwmValues: const [100, 180],
            requestedPwmValues: const [80, 160],
            readbackPwmValues: const [80, 160],
            tempValues: const [50, 70],
            before: _snapshot(fan1Rpm: 6600, cpuTempMilliC: 60000),
            samples: [_snapshot(fan1Rpm: 5800, cpuTempMilliC: 60000)],
          ),
          CurveRuleCaseObservation(
            label: 'zero_until_80c',
            defaultPwmValues: const [100, 180, 220],
            requestedPwmValues: const [0, 0, 220],
            readbackPwmValues: const [0, 0, 220],
            tempValues: const [50, 70, 90],
            before: _snapshot(fan1Rpm: 6600, cpuTempMilliC: 60000),
            samples: [_snapshot(fan1Rpm: 5200, cpuTempMilliC: 60000)],
          ),
        ],
      );

      expect(result.kind, FanHypothesisKind.downwardApplied);
      expect(result.summary, startsWith('zero_until_80c:'));
    });

    test(
      'classifies below-default curve as transformed to default when readback is clamped upward',
      () {
        final result = FanHypothesisDiagnosis.curveRuleDiscovery(
          observations: [
            CurveRuleCaseObservation(
              label: 'current_bucket_below_default',
              defaultPwmValues: const [100, 180],
              requestedPwmValues: const [40, 80],
              readbackPwmValues: const [100, 180],
              tempValues: const [50, 70],
              before: _snapshot(fan1Rpm: 6000, cpuTempMilliC: 60000),
              samples: [_snapshot(fan1Rpm: 5900, cpuTempMilliC: 60000)],
            ),
          ],
        );

        expect(result.kind, FanHypothesisKind.downwardTransformedToDefault);
      },
    );

    test(
      'curve rule discovery writes below-default curves and restores each case',
      () async {
        final curves = _FakeCurvePort();
        final service = XgMobileFanDiagnosticService(
          curvePort: curves,
          platformPort: _FakePlatformPort(),
          probe: _FakeProbe(List.filled(32, _snapshot(cpuTempMilliC: 66000))),
          sampleInterval: Duration.zero,
          samplesPerPwm: 1,
        );

        final result = await service.runCurveValidationTest();

        expect(result.name, 'Curve Rule Discovery');
        expect(curves.writtenCurves, isNotEmpty);
        expect(
          curves.writtenCurves.any((curve) => curve.every((pwm) => pwm < 117)),
          isTrue,
        );
        expect(curves.restoreCount, curves.writtenCurves.length + 1);
      },
    );

    test('curve rule discovery writes range cases up to 80C', () async {
      final curves = _FakeCurvePort(
        defaultCpuPwm: const [50, 100, 150, 200],
        defaultCpuTemps: const [40, 60, 80, 90],
      );
      final service = XgMobileFanDiagnosticService(
        curvePort: curves,
        platformPort: _FakePlatformPort(),
        probe: _FakeProbe(List.filled(48, _snapshot(cpuTempMilliC: 70000))),
        sampleInterval: Duration.zero,
        samplesPerPwm: 1,
      );

      await service.runCurveValidationTest();

      expect(curves.writtenCurves, contains(equals(const [0, 0, 0, 200])));
      expect(
        curves.writtenCurves,
        contains(equals(const [255, 255, 255, 200])),
      );
      expect(curves.writtenCurves, contains(equals(const [0, 20, 70, 200])));
      expect(
        curves.writtenCurves,
        contains(equals(const [130, 180, 230, 200])),
      );
    });

    test('curve rule discovery exposes per-case PWM and RPM deltas', () async {
      final curves = _FakeCurvePort(
        defaultCpuPwm: const [50, 100, 150, 200],
        defaultCpuTemps: const [40, 60, 80, 90],
      );
      final service = XgMobileFanDiagnosticService(
        curvePort: curves,
        platformPort: _FakePlatformPort(),
        probe: _FakeProbe([
          _snapshot(fan1Rpm: 5000, cpuTempMilliC: 60000),
          _snapshot(fan1Rpm: 5000, cpuTempMilliC: 60000),
          _snapshot(fan1Rpm: 5000, cpuTempMilliC: 60000),
          _snapshot(fan1Rpm: 5000, cpuTempMilliC: 60000),
          _snapshot(fan1Rpm: 4500, cpuTempMilliC: 60000),
          ...List.filled(64, _snapshot(fan1Rpm: 5000, cpuTempMilliC: 60000)),
        ]),
        sampleInterval: Duration.zero,
        samplesPerPwm: 1,
      );

      final result = await service.runCurveValidationTest();
      final smallDelta = result.steps.firstWhere(
        (step) => step.label == 'flat_below_default_small_delta',
      );

      expect(smallDelta.currentTempC, 60);
      expect(smallDelta.defaultPwm, 100);
      expect(smallDelta.requestedPwm, 80);
      expect(smallDelta.readbackPwm, 80);
      expect(smallDelta.cpuRpmBefore, 5000);
      expect(smallDelta.cpuRpmAfter, 4500);
      expect(smallDelta.cpuRpmDelta, -500);
    });

    test(
      'curve rule discovery restores defaults after case write failure',
      () async {
        final curves = _FakeCurvePort()..throwOnWrite = true;
        final service = XgMobileFanDiagnosticService(
          curvePort: curves,
          platformPort: _FakePlatformPort(),
          probe: _FakeProbe(List.filled(4, _snapshot(cpuTempMilliC: 66000))),
          sampleInterval: Duration.zero,
          samplesPerPwm: 1,
        );

        await expectLater(
          service.runCurveValidationTest(),
          throwsA(isA<StateError>()),
        );

        expect(curves.restoreCount, 2);
      },
    );
  });
}
