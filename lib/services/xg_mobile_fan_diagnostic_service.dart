import 'dart:async';
import 'dart:convert';
import 'dart:io';

import 'package:dbus/dbus.dart';

import 'asusd_fan_client.dart';

abstract class XgFanCurvePort {
  Future<List<FanCurveData>> getBalancedCurves();
  Future<List<FanCurveData>> getCurves(PerfProfile profile);
  Future<void> setFlatPwm(PerfProfile profile, String fanName, int pwm);
  Future<void> setCustomCurve(
    PerfProfile profile,
    String fanName,
    List<int> pwmValues, {
    List<int>? tempValues,
  });
  Future<void> restoreDefaults(PerfProfile profile);
  Future<void> setBalancedGpuFlatPwm(int pwm);
  Future<void> setBalancedCpuFlatPwm(int pwm);
  Future<void> restoreBalancedDefaults();
}

abstract class XgFanHwmonProbe {
  Future<XgFanHwmonSnapshot> readSnapshot();
}

abstract class XgPlatformProfilePort {
  Future<PerfProfile> getActiveProfile();
  Future<void> setActiveProfile(PerfProfile profile);
}

class AsusdXgFanCurvePort implements XgFanCurvePort {
  final AsusdFanClient _client;

  AsusdXgFanCurvePort({AsusdFanClient? client})
    : _client = client ?? AsusdFanClient();

  @override
  Future<List<FanCurveData>> getBalancedCurves() {
    return getCurves(PerfProfile.balanced);
  }

  @override
  Future<List<FanCurveData>> getCurves(PerfProfile profile) {
    return _client.getFanCurves(profile);
  }

  @override
  Future<void> setFlatPwm(PerfProfile profile, String fanName, int pwm) async {
    await setCustomCurve(profile, fanName, List.filled(8, pwm.clamp(0, 255)));
  }

  @override
  Future<void> setCustomCurve(
    PerfProfile profile,
    String fanName,
    List<int> pwmValues, {
    List<int>? tempValues,
  }) async {
    final curve = FanCurveData(
      name: fanName,
      pwmValues: pwmValues.map((value) => value.clamp(0, 255)).toList(),
      tempValues: tempValues ?? const [30, 40, 50, 60, 70, 80, 90, 100],
      enabled: true,
    );
    final asusdProfile = asusdProfileMap[profile]!;
    await _client.setFanCurve(asusdProfile, curve);
    await _client.setFanCurveEnabled(asusdProfile, fanName, true);
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
  Future<void> restoreDefaults(PerfProfile profile) {
    return _client.resetCurvesToDefaults(asusdProfileMap[profile]!);
  }

  @override
  Future<void> restoreBalancedDefaults() {
    return restoreDefaults(PerfProfile.balanced);
  }
}

class AsusdPlatformProfilePort implements XgPlatformProfilePort {
  static const _busName = 'xyz.ljones.Asusd';
  static const _objectPath = '/xyz/ljones';
  static const _interfaceName = 'xyz.ljones.Platform';

  final DBusClient _client;
  late final DBusRemoteObject _object;

  AsusdPlatformProfilePort() : _client = DBusClient.system() {
    _object = DBusRemoteObject(
      _client,
      name: _busName,
      path: DBusObjectPath(_objectPath),
    );
  }

  @override
  Future<PerfProfile> getActiveProfile() async {
    final result = await _object.getProperty(_interfaceName, 'PlatformProfile');
    final unwrapped = result is DBusVariant ? result.value : result;
    final value = (unwrapped as DBusUint32).value;
    return _profileFromAsusdId(value);
  }

  @override
  Future<void> setActiveProfile(PerfProfile profile) async {
    await _object.setProperty(
      _interfaceName,
      'PlatformProfile',
      DBusUint32(asusdProfileMap[profile]!),
    );
  }
}

class SysfsXgFanHwmonProbe implements XgFanHwmonProbe {
  const SysfsXgFanHwmonProbe();

  @override
  Future<XgFanHwmonSnapshot> readSnapshot() async {
    return XgFanHwmonSnapshot(
      fan1Rpm: await _readInt('/sys/class/hwmon/hwmon7/fan1_input'),
      fan2Rpm: await _readInt('/sys/class/hwmon/hwmon7/fan2_input'),
      pwm1Enable: await _readInt('/sys/class/hwmon/hwmon8/pwm1_enable'),
      pwm2Enable: await _readInt('/sys/class/hwmon/hwmon8/pwm2_enable'),
      pwm2Point8: await _readInt(
        '/sys/class/hwmon/hwmon8/pwm2_auto_point8_pwm',
      ),
      cpuTempMilliC: await _readInt('/sys/class/hwmon/hwmon5/temp1_input'),
      gpuTempMilliC: await _readInt('/sys/class/hwmon/hwmon6/temp1_input'),
      labels: {
        'fan1': await _readString('/sys/class/hwmon/hwmon7/fan1_label'),
        'fan2': await _readString('/sys/class/hwmon/hwmon7/fan2_label'),
      },
    );
  }

  static Future<int?> _readInt(String path) async {
    final value = await _readString(path);
    return value == null ? null : int.tryParse(value);
  }

  static Future<String?> _readString(String path) async {
    final file = File(path);
    if (!await file.exists()) {
      return null;
    }
    final value = await file.readAsString();
    return value.trim();
  }
}

class XgMobileFanDiagnosticService {
  final XgFanCurvePort curvePort;
  final XgFanHwmonProbe probe;
  final XgPlatformProfilePort platformPort;
  final Duration sampleInterval;
  final int samplesPerPwm;
  final List<int> sweepPwmValues;

  XgMobileFanDiagnosticService({
    XgFanCurvePort? curvePort,
    XgFanHwmonProbe? probe,
    XgPlatformProfilePort? platformPort,
    this.sampleInterval = const Duration(seconds: 1),
    this.samplesPerPwm = 15,
    this.sweepPwmValues = const [180, 220, 255],
  }) : curvePort = curvePort ?? AsusdXgFanCurvePort(),
       probe = probe ?? const SysfsXgFanHwmonProbe(),
       platformPort = platformPort ?? AsusdPlatformProfilePort();

  Future<XgFanDiagnosticRun> runSweep({required String label}) async {
    final startedAt = DateTime.now();
    final baselineCurves = await curvePort.getBalancedCurves();
    final baselineSnapshot = await probe.readSnapshot();
    final steps = <XgFanSweepStep>[];

    try {
      for (final pwm in sweepPwmValues) {
        await curvePort.setBalancedGpuFlatPwm(pwm);
        final samples = <XgFanHwmonSnapshot>[];
        for (var i = 0; i < samplesPerPwm; i++) {
          samples.add(await probe.readSnapshot());
          if (sampleInterval > Duration.zero && i < samplesPerPwm - 1) {
            await Future<void>.delayed(sampleInterval);
          }
        }
        steps.add(XgFanSweepStep(requestedPwm: pwm, samples: samples));
      }
    } finally {
      await curvePort.restoreBalancedDefaults();
    }

    return XgFanDiagnosticRun(
      label: label,
      startedAt: startedAt,
      finishedAt: DateTime.now(),
      baselineCurves: baselineCurves,
      baselineSnapshot: baselineSnapshot,
      steps: steps,
    );
  }

  Future<FanHypothesisResult> runCpuGpuCouplingTest() async {
    final startedAt = DateTime.now();
    final cpuSteps = <FanHypothesisStep>[];
    final gpuSteps = <FanHypothesisStep>[];
    try {
      for (final pwm in sweepPwmValues) {
        await curvePort.setBalancedCpuFlatPwm(pwm);
        cpuSteps.add(
          FanHypothesisStep(
            label: 'CPU $pwm',
            requestedFan: 'CPU',
            requestedPwm: pwm,
            samples: await _collectSamples(),
          ),
        );
      }
      await curvePort.restoreBalancedDefaults();
      for (final pwm in sweepPwmValues) {
        await curvePort.setBalancedGpuFlatPwm(pwm);
        gpuSteps.add(
          FanHypothesisStep(
            label: 'GPU $pwm',
            requestedFan: 'GPU',
            requestedPwm: pwm,
            samples: await _collectSamples(),
          ),
        );
      }
    } finally {
      await curvePort.restoreBalancedDefaults();
    }
    final diagnosis = FanHypothesisDiagnosis.cpuGpuCoupling(
      cpuOnlySteps: cpuSteps,
      gpuOnlySteps: gpuSteps,
    );
    return FanHypothesisResult(
      name: 'CPU/GPU Coupling',
      startedAt: startedAt,
      finishedAt: DateTime.now(),
      diagnosis: diagnosis,
      steps: [...cpuSteps, ...gpuSteps],
    );
  }

  Future<FanHypothesisResult> runProfileBoundsTest() async {
    final startedAt = DateTime.now();
    final originalProfile = await platformPort.getActiveProfile();
    final boundaries = <ProfileFanBoundary>[];
    final steps = <FanHypothesisStep>[];
    final profiles = [
      PerfProfile.quiet,
      PerfProfile.balanced,
      PerfProfile.performance,
    ];
    try {
      for (final profile in profiles) {
        await platformPort.setActiveProfile(profile);
        await curvePort.setFlatPwm(profile, 'CPU', 40);
        final low = FanHypothesisStep(
          label: '${profile.name} CPU 40',
          requestedFan: 'CPU',
          requestedPwm: 40,
          samples: await _collectSamples(),
        );
        await curvePort.setFlatPwm(profile, 'CPU', 255);
        final high = FanHypothesisStep(
          label: '${profile.name} CPU 255',
          requestedFan: 'CPU',
          requestedPwm: 255,
          samples: await _collectSamples(),
        );
        steps.addAll([low, high]);
        boundaries.add(
          ProfileFanBoundary(
            profile: profile,
            lowPwmStep: low,
            highPwmStep: high,
          ),
        );
        await curvePort.restoreDefaults(profile);
      }
    } finally {
      for (final profile in profiles) {
        await curvePort.restoreDefaults(profile);
      }
      await platformPort.setActiveProfile(originalProfile);
    }
    return FanHypothesisResult(
      name: 'Profile Bounds',
      startedAt: startedAt,
      finishedAt: DateTime.now(),
      diagnosis: FanHypothesisDiagnosis.profileBounds(boundaries),
      steps: steps,
    );
  }

  Future<FanHypothesisResult> runCurveValidationTest() async {
    final startedAt = DateTime.now();
    final baselineCurves = await curvePort.getBalancedCurves();
    final defaultCurve = _findCurve(baselineCurves, 'CPU');
    final tempValues = defaultCurve.tempValues;
    final defaultPwmValues = defaultCurve.pwmValues;
    final baseline = await probe.readSnapshot();
    final cases = _buildCurveRuleCases(
      defaultPwmValues: defaultPwmValues,
      tempValues: tempValues,
      cpuTempMilliC: baseline.cpuTempMilliC,
    );
    final observations = <CurveRuleCaseObservation>[];
    try {
      for (final testCase in cases) {
        try {
          observations.add(
            await _runCurveRuleObservation(
              label: testCase.label,
              defaultPwmValues: defaultPwmValues,
              requestedPwmValues: testCase.pwmValues,
              tempValues: tempValues,
            ),
          );
        } finally {
          await curvePort.restoreBalancedDefaults();
        }
      }
      return FanHypothesisResult(
        name: 'Curve Rule Discovery',
        startedAt: startedAt,
        finishedAt: DateTime.now(),
        diagnosis: FanHypothesisDiagnosis.curveRuleDiscovery(
          observations: observations,
        ),
        steps: observations
            .map(
              (observation) => FanHypothesisStep(
                label: observation.label,
                requestedFan: 'CPU',
                requestedPwm: observation.requestedPwmAtTemp?.round() ?? 0,
                currentTempC: observation.currentTempC,
                defaultPwm: observation.defaultPwmAtTemp?.round(),
                readbackPwm: observation.readbackPwmAtTemp?.round(),
                cpuRpmBefore: observation.cpuRpmBefore,
                cpuRpmAfter: observation.cpuRpmAfter,
                cpuRpmDelta: observation.cpuRpmDelta,
                samples: observation.samples,
              ),
            )
            .toList(),
      );
    } finally {
      await curvePort.restoreBalancedDefaults();
    }
  }

  Future<CurveRuleCaseObservation> _runCurveRuleObservation({
    required String label,
    required List<int> defaultPwmValues,
    required List<int> requestedPwmValues,
    required List<int> tempValues,
  }) async {
    final before = await probe.readSnapshot();
    await curvePort.setCustomCurve(
      PerfProfile.balanced,
      'CPU',
      requestedPwmValues,
      tempValues: tempValues,
    );
    final readback = _findCurve(await curvePort.getBalancedCurves(), 'CPU');
    return CurveRuleCaseObservation(
      label: label,
      defaultPwmValues: defaultPwmValues,
      requestedPwmValues: requestedPwmValues,
      readbackPwmValues: readback.pwmValues,
      tempValues: tempValues,
      before: before,
      samples: await _collectSamples(),
    );
  }

  List<_CurveRuleCase> _buildCurveRuleCases({
    required List<int> defaultPwmValues,
    required List<int> tempValues,
    required int? cpuTempMilliC,
  }) {
    List<int> shiftAll(int delta) {
      return defaultPwmValues
          .map((value) => (value + delta).clamp(0, 255))
          .toList();
    }

    List<int> shiftCurrentBucket(int delta) {
      final values = List<int>.of(defaultPwmValues);
      final index = _nearestCurvePointIndex(tempValues, cpuTempMilliC);
      for (final point in {index - 1, index}) {
        if (point >= 0 && point < values.length) {
          values[point] = (values[point] + delta).clamp(0, 255);
        }
      }
      return values;
    }

    List<int> transformUntilTemp(
      int maxTempC,
      int Function(int defaultPwm) transform,
    ) {
      final values = List<int>.of(defaultPwmValues);
      for (var i = 0; i < values.length && i < tempValues.length; i++) {
        if (tempValues[i] <= maxTempC) {
          values[i] = transform(values[i]).clamp(0, 255);
        }
      }
      return values;
    }

    final nonMonotonic = List<int>.of(defaultPwmValues);
    if (nonMonotonic.length >= 4) {
      nonMonotonic[1] = 220;
      nonMonotonic[2] = 60;
      nonMonotonic[3] = 200;
    }

    return [
      _CurveRuleCase('default', List<int>.of(defaultPwmValues)),
      _CurveRuleCase('flat_below_default_small_delta', shiftAll(-20)),
      _CurveRuleCase('flat_below_default_large_delta', shiftAll(-80)),
      _CurveRuleCase('current_bucket_below_default', shiftCurrentBucket(-80)),
      _CurveRuleCase('current_bucket_above_default', shiftCurrentBucket(40)),
      _CurveRuleCase('all_above_default', shiftAll(40)),
      _CurveRuleCase('zero_until_80c', transformUntilTemp(80, (_) => 0)),
      _CurveRuleCase('max_until_80c', transformUntilTemp(80, (_) => 255)),
      _CurveRuleCase(
        'below_default_until_80c',
        transformUntilTemp(80, (value) => value - 80),
      ),
      _CurveRuleCase(
        'above_default_until_80c',
        transformUntilTemp(80, (value) => value + 80),
      ),
      _CurveRuleCase('flat_low', List<int>.filled(defaultPwmValues.length, 40)),
      _CurveRuleCase(
        'flat_high',
        List<int>.filled(defaultPwmValues.length, 255),
      ),
      _CurveRuleCase('non_monotonic_pwm', nonMonotonic),
    ];
  }

  int _nearestCurvePointIndex(List<int> tempValues, int? cpuTempMilliC) {
    if (tempValues.isEmpty || cpuTempMilliC == null) {
      return 0;
    }
    final tempC = (cpuTempMilliC / 1000).round();
    var bestIndex = 0;
    var bestDelta = (tempValues.first - tempC).abs();
    for (var i = 1; i < tempValues.length; i++) {
      final delta = (tempValues[i] - tempC).abs();
      if (delta < bestDelta) {
        bestIndex = i;
        bestDelta = delta;
      }
    }
    return bestIndex;
  }

  Future<List<XgFanHwmonSnapshot>> _collectSamples() async {
    final samples = <XgFanHwmonSnapshot>[];
    for (var i = 0; i < samplesPerPwm; i++) {
      samples.add(await probe.readSnapshot());
      if (sampleInterval > Duration.zero && i < samplesPerPwm - 1) {
        await Future<void>.delayed(sampleInterval);
      }
    }
    return samples;
  }
}

class _CurveRuleCase {
  final String label;
  final List<int> pwmValues;

  const _CurveRuleCase(this.label, this.pwmValues);
}

class XgFanHwmonSnapshot {
  final int? fan1Rpm;
  final int? fan2Rpm;
  final int? pwm1Enable;
  final int? pwm2Enable;
  final int? pwm2Point8;
  final int? cpuTempMilliC;
  final int? gpuTempMilliC;
  final Map<String, String?> labels;

  const XgFanHwmonSnapshot({
    required this.fan1Rpm,
    required this.fan2Rpm,
    required this.pwm1Enable,
    required this.pwm2Enable,
    required this.pwm2Point8,
    required this.cpuTempMilliC,
    required this.gpuTempMilliC,
    required this.labels,
  });

  factory XgFanHwmonSnapshot.fromJson(Map<String, dynamic> json) {
    return XgFanHwmonSnapshot(
      fan1Rpm: json['fan1Rpm'] as int?,
      fan2Rpm: json['fan2Rpm'] as int?,
      pwm1Enable: json['pwm1Enable'] as int?,
      pwm2Enable: json['pwm2Enable'] as int?,
      pwm2Point8: json['pwm2Point8'] as int?,
      cpuTempMilliC: json['cpuTempMilliC'] as int?,
      gpuTempMilliC: json['gpuTempMilliC'] as int?,
      labels: (json['labels'] as Map<String, dynamic>).map(
        (key, value) => MapEntry(key, value as String?),
      ),
    );
  }

  Map<String, dynamic> toJson() {
    return {
      'fan1Rpm': fan1Rpm,
      'fan2Rpm': fan2Rpm,
      'pwm1Enable': pwm1Enable,
      'pwm2Enable': pwm2Enable,
      'pwm2Point8': pwm2Point8,
      'cpuTempMilliC': cpuTempMilliC,
      'gpuTempMilliC': gpuTempMilliC,
      'labels': labels,
    };
  }
}

class XgFanSweepStep {
  final int requestedPwm;
  final List<XgFanHwmonSnapshot> samples;

  const XgFanSweepStep({required this.requestedPwm, required this.samples});

  factory XgFanSweepStep.fromJson(Map<String, dynamic> json) {
    return XgFanSweepStep(
      requestedPwm: json['requestedPwm'] as int,
      samples: (json['samples'] as List<dynamic>)
          .map(
            (item) => XgFanHwmonSnapshot.fromJson(item as Map<String, dynamic>),
          )
          .toList(),
    );
  }

  Map<String, dynamic> toJson() {
    return {
      'requestedPwm': requestedPwm,
      'samples': samples.map((sample) => sample.toJson()).toList(),
    };
  }

  int? get fan2MinRpm => _fan2Values.isEmpty ? null : _fan2Values.reduce(_min);
  int? get fan2MaxRpm => _fan2Values.isEmpty ? null : _fan2Values.reduce(_max);
  int? get fan2LastRpm => _fan2Values.isEmpty ? null : _fan2Values.last;

  int? get fan2DeltaRpm {
    final min = fan2MinRpm;
    final max = fan2MaxRpm;
    return min == null || max == null ? null : max - min;
  }

  bool get acceptedByDriver {
    return samples.isNotEmpty &&
        samples.every(
          (sample) =>
              sample.pwm2Enable == 1 && sample.pwm2Point8 == requestedPwm,
        );
  }

  XgFanStepClassification get classification {
    final delta = fan2DeltaRpm;
    if (delta == null) {
      return XgFanStepClassification.inconclusive;
    }
    if (delta >= 500) {
      return XgFanStepClassification.responded;
    }
    if (acceptedByDriver) {
      return XgFanStepClassification.acceptedNoResponse;
    }
    return XgFanStepClassification.inconclusive;
  }

  List<int> get _fan2Values {
    return [
      for (final sample in samples)
        if (sample.fan2Rpm != null) sample.fan2Rpm!,
    ];
  }

  static int _min(int a, int b) => a < b ? a : b;
  static int _max(int a, int b) => a > b ? a : b;
}

enum XgFanStepClassification { responded, acceptedNoResponse, inconclusive }

class XgFanDiagnosticRun {
  final String label;
  final DateTime startedAt;
  final DateTime finishedAt;
  final List<FanCurveData> baselineCurves;
  final XgFanHwmonSnapshot baselineSnapshot;
  final List<XgFanSweepStep> steps;

  const XgFanDiagnosticRun({
    required this.label,
    required this.startedAt,
    required this.finishedAt,
    required this.baselineCurves,
    required this.baselineSnapshot,
    required this.steps,
  });

  factory XgFanDiagnosticRun.fromJson(Map<String, dynamic> json) {
    return XgFanDiagnosticRun(
      label: json['label'] as String,
      startedAt: DateTime.parse(json['startedAt'] as String),
      finishedAt: DateTime.parse(json['finishedAt'] as String),
      baselineCurves: (json['baselineCurves'] as List<dynamic>)
          .map((item) => _curveFromJson(item as Map<String, dynamic>))
          .toList(),
      baselineSnapshot: XgFanHwmonSnapshot.fromJson(
        json['baselineSnapshot'] as Map<String, dynamic>,
      ),
      steps: (json['steps'] as List<dynamic>)
          .map((item) => XgFanSweepStep.fromJson(item as Map<String, dynamic>))
          .toList(),
    );
  }

  Map<String, dynamic> toJson() {
    return {
      'label': label,
      'startedAt': startedAt.toIso8601String(),
      'finishedAt': finishedAt.toIso8601String(),
      'baselineCurves': baselineCurves.map(_curveToJson).toList(),
      'baselineSnapshot': baselineSnapshot.toJson(),
      'steps': steps.map((step) => step.toJson()).toList(),
    };
  }

  bool get hasFanResponse {
    return steps.any(
      (step) => step.classification == XgFanStepClassification.responded,
    );
  }

  bool get hasMissingFan2Rpm {
    return baselineSnapshot.fan2Rpm == null ||
        steps.any((step) => step.fan2LastRpm == null);
  }

  bool hasDifferentLabelsFrom(XgFanDiagnosticRun other) {
    return baselineSnapshot.labels.toString() !=
        other.baselineSnapshot.labels.toString();
  }

  int? fan2BaselineDeltaFrom(XgFanDiagnosticRun other) {
    final first = baselineSnapshot.fan2Rpm;
    final second = other.baselineSnapshot.fan2Rpm;
    if (first == null || second == null) {
      return null;
    }
    return (first - second).abs();
  }
}

FanCurveData _curveFromJson(Map<String, dynamic> json) {
  return FanCurveData(
    name: json['name'] as String,
    pwmValues: (json['pwmValues'] as List<dynamic>)
        .map((value) => value as int)
        .toList(),
    tempValues: (json['tempValues'] as List<dynamic>)
        .map((value) => value as int)
        .toList(),
    enabled: json['enabled'] as bool,
  );
}

Map<String, dynamic> _curveToJson(FanCurveData curve) {
  return {
    'name': curve.name,
    'pwmValues': curve.pwmValues,
    'tempValues': curve.tempValues,
    'enabled': curve.enabled,
  };
}

FanCurveData _findCurve(List<FanCurveData> curves, String name) {
  return curves.firstWhere(
    (curve) => curve.name == name,
    orElse: () => FanCurveData(
      name: name,
      pwmValues: const [],
      tempValues: const [],
      enabled: false,
    ),
  );
}

PerfProfile _profileFromAsusdId(int id) {
  for (final entry in asusdProfileMap.entries) {
    if (entry.value == id) {
      return entry.key;
    }
  }
  return PerfProfile.balanced;
}

class XgFanDiagnosticSession {
  final XgFanDiagnosticRun? currentRun;
  final XgFanDiagnosticRun? changedRun;

  const XgFanDiagnosticSession({this.currentRun, this.changedRun});

  factory XgFanDiagnosticSession.fromJson(Map<String, dynamic> json) {
    return XgFanDiagnosticSession(
      currentRun: json['currentRun'] == null
          ? null
          : XgFanDiagnosticRun.fromJson(
              json['currentRun'] as Map<String, dynamic>,
            ),
      changedRun: json['changedRun'] == null
          ? null
          : XgFanDiagnosticRun.fromJson(
              json['changedRun'] as Map<String, dynamic>,
            ),
    );
  }

  Map<String, dynamic> toJson() {
    return {
      'currentRun': currentRun?.toJson(),
      'changedRun': changedRun?.toJson(),
    };
  }
}

abstract class XgFanDiagnosticStore {
  Future<XgFanDiagnosticSession> load();
  Future<void> save(XgFanDiagnosticSession session);
  Future<void> clear();
}

class FileXgFanDiagnosticStore implements XgFanDiagnosticStore {
  final File file;

  FileXgFanDiagnosticStore({File? file}) : file = file ?? _defaultFile();

  @override
  Future<XgFanDiagnosticSession> load() async {
    if (!await file.exists()) {
      return const XgFanDiagnosticSession();
    }
    final decoded = jsonDecode(await file.readAsString());
    return XgFanDiagnosticSession.fromJson(decoded as Map<String, dynamic>);
  }

  @override
  Future<void> save(XgFanDiagnosticSession session) async {
    await file.parent.create(recursive: true);
    await file.writeAsString(
      const JsonEncoder.withIndent('  ').convert(session.toJson()),
    );
  }

  @override
  Future<void> clear() async {
    if (await file.exists()) {
      await file.delete();
    }
  }

  static File _defaultFile() {
    final stateHome =
        Platform.environment['XDG_STATE_HOME'] ??
        '${Platform.environment['HOME'] ?? '.'}/.local/state';
    return File('$stateHome/asus-linux-control/xg_mobile_fan_diagnostic.json');
  }
}

enum XgFanDiagnosisKind {
  responded,
  acceptedNoResponse,
  routeChanged,
  gpuDependsOnXgMobile,
  newHwmonLikely,
  inconclusive,
}

class XgFanDiagnosis {
  final XgFanDiagnosisKind kind;
  final String summary;
  final String nextAction;

  const XgFanDiagnosis({
    required this.kind,
    required this.summary,
    required this.nextAction,
  });

  factory XgFanDiagnosis.single(XgFanDiagnosticRun run) {
    if (run.hasMissingFan2Rpm) {
      return const XgFanDiagnosis(
        kind: XgFanDiagnosisKind.inconclusive,
        summary: 'fan2 RPM was not available for this run.',
        nextAction: 'Check hwmon labels and repeat the sweep after reboot.',
      );
    }
    if (run.hasFanResponse) {
      return const XgFanDiagnosis(
        kind: XgFanDiagnosisKind.responded,
        summary: 'fan2 RPM changed during the GPU PWM sweep.',
        nextAction: 'Compare this result with the opposite XG Mobile state.',
      );
    }
    return const XgFanDiagnosis(
      kind: XgFanDiagnosisKind.acceptedNoResponse,
      summary:
          'The GPU curve was accepted, but fan2 RPM did not meaningfully change.',
      nextAction:
          'Run the changed-state sweep to see whether XG Mobile changes routing.',
    );
  }

  factory XgFanDiagnosis.compare(
    XgFanDiagnosticRun first,
    XgFanDiagnosticRun second,
  ) {
    if (first.hasDifferentLabelsFrom(second)) {
      return const XgFanDiagnosis(
        kind: XgFanDiagnosisKind.newHwmonLikely,
        summary: 'Fan labels changed between runs.',
        nextAction: 'Inspect the new hwmon device paths before trusting fan2.',
      );
    }
    if (first.hasMissingFan2Rpm || second.hasMissingFan2Rpm) {
      return const XgFanDiagnosis(
        kind: XgFanDiagnosisKind.inconclusive,
        summary: 'At least one run did not expose fan2 RPM.',
        nextAction: 'Record hwmon labels and repeat after a clean reboot.',
      );
    }
    if (first.hasFanResponse != second.hasFanResponse) {
      return const XgFanDiagnosis(
        kind: XgFanDiagnosisKind.routeChanged,
        summary: 'Only one XG Mobile state produced a fan2 RPM response.',
        nextAction:
            'Treat XG Mobile state as changing the GPU fan control route.',
      );
    }
    final baselineDelta = first.fan2BaselineDeltaFrom(second);
    if (baselineDelta != null && baselineDelta >= 500) {
      return const XgFanDiagnosis(
        kind: XgFanDiagnosisKind.gpuDependsOnXgMobile,
        summary: 'fan2 baseline RPM changed between XG Mobile states.',
        nextAction:
            'Treat XG Mobile state as changing firmware fan policy even if pwm2 is not direct control.',
      );
    }
    if (first.hasFanResponse && second.hasFanResponse) {
      return const XgFanDiagnosis(
        kind: XgFanDiagnosisKind.responded,
        summary: 'Both runs produced fan2 RPM response.',
        nextAction: 'GPU fan control appears independent of XG Mobile state.',
      );
    }
    return const XgFanDiagnosis(
      kind: XgFanDiagnosisKind.acceptedNoResponse,
      summary:
          'Both runs accepted GPU curve writes without a meaningful RPM change.',
      nextAction:
          'Look for a separate XG Mobile fan interface or EC/SAIO control path.',
    );
  }
}

enum FanHypothesisKind {
  gpuDependsOnCpuFan,
  profileFloorOrCap,
  downwardApplied,
  downwardStoredButRuntimeClamped,
  downwardTransformedToDefault,
  upwardApplied,
  curveNoRuntimeEffect,
  curveFollowsRuntime,
  curveTransformed,
  curveStoredButIgnored,
  curveInconclusive,
  noEvidence,
  inconclusive,
}

class FanHypothesisResult {
  final String name;
  final DateTime startedAt;
  final DateTime finishedAt;
  final FanHypothesisDiagnosis diagnosis;
  final List<FanHypothesisStep> steps;

  const FanHypothesisResult({
    required this.name,
    required this.startedAt,
    required this.finishedAt,
    required this.diagnosis,
    required this.steps,
  });
}

class FanHypothesisDiagnosis {
  static const int cpuFanMaxRpm = 10000;

  final FanHypothesisKind kind;
  final String summary;
  final String nextAction;

  const FanHypothesisDiagnosis({
    required this.kind,
    required this.summary,
    required this.nextAction,
  });

  factory FanHypothesisDiagnosis.cpuGpuCoupling({
    required List<FanHypothesisStep> cpuOnlySteps,
    required List<FanHypothesisStep> gpuOnlySteps,
  }) {
    final cpuFan2Delta = _maxFan2Delta(cpuOnlySteps);
    final gpuFan2Delta = _maxFan2Delta(gpuOnlySteps);
    if (cpuFan2Delta == null || gpuFan2Delta == null) {
      return const FanHypothesisDiagnosis(
        kind: FanHypothesisKind.inconclusive,
        summary: 'fan2 RPM data was missing during coupling test.',
        nextAction: 'Repeat after verifying fan2_input is readable.',
      );
    }
    if (cpuFan2Delta >= 500 && gpuFan2Delta < 500) {
      return const FanHypothesisDiagnosis(
        kind: FanHypothesisKind.gpuDependsOnCpuFan,
        summary: 'fan2 changed during CPU-only sweep but not GPU-only sweep.',
        nextAction:
            'Treat fan2 as coupled to CPU/common EC policy, not direct GPU pwm2 control.',
      );
    }
    return const FanHypothesisDiagnosis(
      kind: FanHypothesisKind.noEvidence,
      summary:
          'CPU-only sweep did not show stronger fan2 response than GPU-only sweep.',
      nextAction: 'Use profile bounds and curve validation tests next.',
    );
  }

  factory FanHypothesisDiagnosis.profileBounds(
    List<ProfileFanBoundary> boundaries,
  ) {
    if (boundaries.length < 2) {
      return const FanHypothesisDiagnosis(
        kind: FanHypothesisKind.inconclusive,
        summary: 'At least two profiles are needed to compare floor/cap.',
        nextAction: 'Run the bounds test for Quiet, Balanced, and Performance.',
      );
    }
    final lowLasts = [
      for (final boundary in boundaries)
        if (boundary.lowPwmStep.fan2LastRpm != null)
          boundary.lowPwmStep.fan2LastRpm!,
    ];
    final highLasts = [
      for (final boundary in boundaries)
        if (boundary.highPwmStep.fan2LastRpm != null)
          boundary.highPwmStep.fan2LastRpm!,
    ];
    if (lowLasts.length < 2 || highLasts.length < 2) {
      return const FanHypothesisDiagnosis(
        kind: FanHypothesisKind.inconclusive,
        summary: 'RPM data was missing for profile bounds.',
        nextAction: 'Repeat after verifying fan RPM sensors are readable.',
      );
    }
    if ((_maxValue(lowLasts) - _minValue(lowLasts)).abs() >= 500 ||
        (_maxValue(highLasts) - _minValue(highLasts)).abs() >= 500) {
      return const FanHypothesisDiagnosis(
        kind: FanHypothesisKind.profileFloorOrCap,
        summary: 'Different profiles produced different RPM floors or caps.',
        nextAction:
            'Model fan percentages as profile-limited requests, not absolute speeds.',
      );
    }
    return const FanHypothesisDiagnosis(
      kind: FanHypothesisKind.noEvidence,
      summary: 'Profile RPM floors/caps were not distinguishable in this run.',
      nextAction: 'Repeat with higher thermal load if needed.',
    );
  }

  factory FanHypothesisDiagnosis.runtimeCurveValidation({
    required RuntimeCurveObservation low,
    required RuntimeCurveObservation high,
  }) {
    if (low.curveTransformed || high.curveTransformed) {
      return const FanHypothesisDiagnosis(
        kind: FanHypothesisKind.curveTransformed,
        summary: 'Readback curve differs from requested curve.',
        nextAction:
            'Log requested/readback pairs and clamp UI to values firmware preserves.',
      );
    }
    final lowExpected = low.expectedCpuRpmLast;
    final highExpected = high.expectedCpuRpmLast;
    if (lowExpected == null || highExpected == null) {
      return const FanHypothesisDiagnosis(
        kind: FanHypothesisKind.curveInconclusive,
        summary:
            'Current temperature could not be mapped to expected curve RPM.',
        nextAction:
            'Check temperature telemetry and curve point definitions before repeating.',
      );
    }
    if ((highExpected - lowExpected).abs() < 1500) {
      return const FanHypothesisDiagnosis(
        kind: FanHypothesisKind.curveInconclusive,
        summary:
            'Runtime curves did not create a clear expected RPM contrast for the observed temperature range.',
        nextAction:
            'Adjust curve points so low/high curves differ around the observed temperature range.',
      );
    }
    final rpmDelta = (high.cpuRpmAfter ?? 0) - (low.cpuRpmAfter ?? 0);
    if (rpmDelta >= 500 && highExpected > lowExpected) {
      return const FanHypothesisDiagnosis(
        kind: FanHypothesisKind.curveFollowsRuntime,
        summary:
            'RPM increased when expected PWM at current temperature increased.',
        nextAction:
            'Curve runtime behavior is visible; compare remaining limits with profile bounds.',
      );
    }
    return const FanHypothesisDiagnosis(
      kind: FanHypothesisKind.curveStoredButIgnored,
      summary:
          'Readback matched and expected PWM differed, but RPM did not meaningfully follow.',
      nextAction:
          'Treat stored curve as advisory data that EC may clamp or ignore at runtime.',
    );
  }

  factory FanHypothesisDiagnosis.curveRuleDiscovery({
    required List<CurveRuleCaseObservation> observations,
  }) {
    if (observations.isEmpty) {
      return const FanHypothesisDiagnosis(
        kind: FanHypothesisKind.inconclusive,
        summary: 'No curve rule cases were collected.',
        nextAction: 'Repeat the curve rule discovery test.',
      );
    }

    final transformedToDefault = observations
        .where((observation) => observation.downwardTransformedToDefault)
        .toList();
    if (transformedToDefault.isNotEmpty) {
      return FanHypothesisDiagnosis(
        kind: FanHypothesisKind.downwardTransformedToDefault,
        summary:
            '${transformedToDefault.first.label}: below-default request was read back near the default curve.',
        nextAction:
            'Treat lower-than-default points as rejected or clamped by asusd/EC storage validation.',
      );
    }

    final transformed = observations
        .where((observation) => observation.curveTransformed)
        .toList();
    if (transformed.isNotEmpty) {
      return FanHypothesisDiagnosis(
        kind: FanHypothesisKind.curveTransformed,
        summary:
            '${transformed.first.label}: readback curve differs from the requested curve.',
        nextAction:
            'Compare requested/readback points to identify the exact EC/asusd rewrite rule.',
      );
    }

    final downwardApplied = observations
        .where((observation) => observation.downwardApplied)
        .toList();
    if (downwardApplied.isNotEmpty) {
      final observation = _preferredCurveRuleObservation(downwardApplied);
      return FanHypothesisDiagnosis(
        kind: FanHypothesisKind.downwardApplied,
        summary:
            '${observation.label}: below-default curve was stored and CPU RPM dropped. '
            '${_curveRuleNumbers(observation)}',
        nextAction:
            'Below-default points can affect runtime in this case; compare which shapes remain accepted. '
            'drop threshold=500 RPM; below-default threshold=20 PWM.',
      );
    }

    final downwardClamped = observations
        .where((observation) => observation.downwardStoredButRuntimeClamped)
        .toList();
    if (downwardClamped.isNotEmpty) {
      return FanHypothesisDiagnosis(
        kind: FanHypothesisKind.downwardStoredButRuntimeClamped,
        summary:
            '${downwardClamped.first.label}: below-default curve was stored, but CPU RPM did not drop.',
        nextAction:
            'Treat the stored curve as subject to a runtime floor or EC clamp below the default policy.',
      );
    }

    final upwardApplied = observations
        .where((observation) => observation.upwardApplied)
        .toList();
    if (upwardApplied.isNotEmpty) {
      return FanHypothesisDiagnosis(
        kind: FanHypothesisKind.upwardApplied,
        summary:
            '${upwardApplied.first.label}: above-default curve raised CPU RPM.',
        nextAction:
            'Upward curve changes are visible; focus further tests on downward clamps and profile caps.',
      );
    }

    final noEffect = observations
        .where((observation) => observation.curveNoRuntimeEffect)
        .toList();
    if (noEffect.isNotEmpty) {
      return FanHypothesisDiagnosis(
        kind: FanHypothesisKind.curveNoRuntimeEffect,
        summary:
            '${noEffect.first.label}: requested/readback PWM differed from default, but CPU RPM barely moved.',
        nextAction:
            'Treat this as no runtime effect for that curve shape and compare with profile bounds.',
      );
    }

    return const FanHypothesisDiagnosis(
      kind: FanHypothesisKind.curveInconclusive,
      summary:
          'Curve cases did not produce enough requested/default PWM or RPM separation to infer a rule.',
      nextAction:
          'Repeat with a stronger thermal load or longer sample window around a known CPU temperature range.',
    );
  }

  static CurveRuleCaseObservation _preferredCurveRuleObservation(
    List<CurveRuleCaseObservation> observations,
  ) {
    return observations.reduce((best, candidate) {
      final bestScore = _curveRuleObservationPriority(best);
      final candidateScore = _curveRuleObservationPriority(candidate);
      return candidateScore > bestScore ? candidate : best;
    });
  }

  static int _curveRuleObservationPriority(
    CurveRuleCaseObservation observation,
  ) {
    final isBoundaryCase = observation.label.contains('until_80c') ? 100000 : 0;
    final rpmDelta = observation.cpuRpmDelta?.abs() ?? 0;
    final pwmDelta =
        ((observation.defaultPwmAtTemp ?? 0) -
                (observation.requestedPwmAtTemp ?? 0))
            .abs()
            .round();
    return isBoundaryCase + rpmDelta + pwmDelta;
  }

  static String _curveRuleNumbers(CurveRuleCaseObservation observation) {
    final before = observation.cpuRpmBefore;
    final after = observation.cpuRpmAfter;
    final delta = observation.cpuRpmDelta;
    final rpmText = before == null || after == null
        ? 'CPU RPM unavailable'
        : 'CPU RPM $before->$after, delta=${_formatSignedInt(delta!)}';

    return 'temp=${_formatNullableInt(observation.currentTempC)}C; '
        'PWM default=${_formatNullableDouble(observation.defaultPwmAtTemp)}, '
        'requested=${_formatNullableDouble(observation.requestedPwmAtTemp)}, '
        'readback=${_formatNullableDouble(observation.readbackPwmAtTemp)}; '
        '$rpmText.';
  }

  static String _formatNullableInt(int? value) {
    return value == null ? 'unknown' : '$value';
  }

  static String _formatNullableDouble(double? value) {
    if (value == null) {
      return 'unknown';
    }
    if (value == value.roundToDouble()) {
      return value.round().toString();
    }
    return value.toStringAsFixed(1);
  }

  static String _formatSignedInt(int value) {
    return value > 0 ? '+$value' : '$value';
  }

  factory FanHypothesisDiagnosis.curveStorageValidation({
    required List<int> requested,
    required List<int> readback,
    required FanHypothesisStep step,
  }) {
    if (requested.length != readback.length) {
      return const FanHypothesisDiagnosis(
        kind: FanHypothesisKind.curveTransformed,
        summary: 'Readback curve shape length differs from requested curve.',
        nextAction: 'Treat EC/asusd as transforming the curve.',
      );
    }
    for (var i = 0; i < requested.length; i++) {
      if (requested[i] != readback[i]) {
        return const FanHypothesisDiagnosis(
          kind: FanHypothesisKind.curveTransformed,
          summary: 'Readback curve differs from requested curve.',
          nextAction:
              'Log requested/readback pairs and clamp UI to values firmware preserves.',
        );
      }
    }
    if ((step.fan2DeltaRpm ?? 0) < 500) {
      return const FanHypothesisDiagnosis(
        kind: FanHypothesisKind.curveStoredButIgnored,
        summary: 'Curve readback matched, but fan RPM did not follow.',
        nextAction:
            'Treat stored fan curve as advisory data that EC may ignore at runtime.',
      );
    }
    return const FanHypothesisDiagnosis(
      kind: FanHypothesisKind.noEvidence,
      summary: 'Curve was stored and fan RPM changed.',
      nextAction:
          'Compare against profile bounds to identify remaining clamps.',
    );
  }

  static int? _maxFan2Delta(List<FanHypothesisStep> steps) {
    final deltas = [
      for (final step in steps)
        if (step.fan2DeltaRpm != null) step.fan2DeltaRpm!,
    ];
    if (deltas.isEmpty) {
      return null;
    }
    return _maxValue(deltas);
  }
}

class FanHypothesisStep {
  final String label;
  final String requestedFan;
  final int requestedPwm;
  final int? currentTempC;
  final int? defaultPwm;
  final int? readbackPwm;
  final int? cpuRpmBefore;
  final int? cpuRpmAfter;
  final int? cpuRpmDelta;
  final List<XgFanHwmonSnapshot> samples;

  const FanHypothesisStep({
    required this.label,
    required this.requestedFan,
    required this.requestedPwm,
    this.currentTempC,
    this.defaultPwm,
    this.readbackPwm,
    this.cpuRpmBefore,
    this.cpuRpmAfter,
    this.cpuRpmDelta,
    required this.samples,
  });

  int? get fan2MinRpm => _fan2Values.isEmpty ? null : _minValue(_fan2Values);
  int? get fan2MaxRpm => _fan2Values.isEmpty ? null : _maxValue(_fan2Values);
  int? get fan2LastRpm => _fan2Values.isEmpty ? null : _fan2Values.last;
  int? get cpuLastRpm {
    if (cpuRpmAfter != null) {
      return cpuRpmAfter;
    }
    final values = [
      for (final sample in samples)
        if (sample.fan1Rpm != null) sample.fan1Rpm!,
    ];
    return values.isEmpty ? null : values.last;
  }

  int? get fan2DeltaRpm {
    final min = fan2MinRpm;
    final max = fan2MaxRpm;
    if (min == null || max == null) {
      return null;
    }
    return max - min;
  }

  List<int> get _fan2Values {
    return [
      for (final sample in samples)
        if (sample.fan2Rpm != null) sample.fan2Rpm!,
    ];
  }
}

class ProfileFanBoundary {
  final PerfProfile profile;
  final FanHypothesisStep lowPwmStep;
  final FanHypothesisStep highPwmStep;

  const ProfileFanBoundary({
    required this.profile,
    required this.lowPwmStep,
    required this.highPwmStep,
  });
}

class CurveRuleCaseObservation {
  final String label;
  final List<int> defaultPwmValues;
  final List<int> requestedPwmValues;
  final List<int> readbackPwmValues;
  final List<int> tempValues;
  final XgFanHwmonSnapshot before;
  final List<XgFanHwmonSnapshot> samples;

  const CurveRuleCaseObservation({
    required this.label,
    required this.defaultPwmValues,
    required this.requestedPwmValues,
    required this.readbackPwmValues,
    required this.tempValues,
    required this.before,
    required this.samples,
  });

  int? get currentTempC {
    final temp = samples.isEmpty
        ? before.cpuTempMilliC
        : samples.last.cpuTempMilliC;
    return temp == null ? null : (temp / 1000).round();
  }

  double? get defaultPwmAtTemp {
    return _interpolatePwm(defaultPwmValues, currentTempC);
  }

  double? get requestedPwmAtTemp {
    return _interpolatePwm(requestedPwmValues, currentTempC);
  }

  double? get readbackPwmAtTemp {
    return _interpolatePwm(readbackPwmValues, currentTempC);
  }

  double? get expectedRequestedCpuRpm {
    final pwm = requestedPwmAtTemp;
    return pwm == null ? null : pwm / 255 * FanHypothesisDiagnosis.cpuFanMaxRpm;
  }

  double? get expectedPwmLast => requestedPwmAtTemp;
  double? get readbackPwmLast => readbackPwmAtTemp;
  double? get expectedCpuRpmLast => expectedRequestedCpuRpm;

  int? get cpuRpmBefore => before.fan1Rpm;
  int? get cpuRpmAfter => samples.isEmpty ? null : samples.last.fan1Rpm;
  int? get cpuRpmDelta {
    final before = cpuRpmBefore;
    final after = cpuRpmAfter;
    return before == null || after == null ? null : after - before;
  }

  bool get requestsBelowDefault {
    final requested = requestedPwmAtTemp;
    final baseline = defaultPwmAtTemp;
    return requested != null && baseline != null && requested <= baseline - 20;
  }

  bool get requestsAboveDefault {
    final requested = requestedPwmAtTemp;
    final baseline = defaultPwmAtTemp;
    return requested != null && baseline != null && requested >= baseline + 20;
  }

  bool get readbackNearDefault {
    final readback = readbackPwmAtTemp;
    final baseline = defaultPwmAtTemp;
    return readback != null &&
        baseline != null &&
        (readback - baseline).abs() <= 20;
  }

  bool get readbackNearRequested {
    final readback = readbackPwmAtTemp;
    final requested = requestedPwmAtTemp;
    return readback != null &&
        requested != null &&
        (readback - requested).abs() <= 20;
  }

  bool get downwardApplied {
    final delta = cpuRpmDelta;
    return requestsBelowDefault &&
        readbackNearRequested &&
        delta != null &&
        delta <= -500;
  }

  bool get downwardStoredButRuntimeClamped {
    final delta = cpuRpmDelta;
    return requestsBelowDefault &&
        readbackNearRequested &&
        delta != null &&
        delta > -500;
  }

  bool get downwardTransformedToDefault {
    return requestsBelowDefault &&
        readbackNearDefault &&
        !readbackNearRequested;
  }

  bool get upwardApplied {
    final delta = cpuRpmDelta;
    return requestsAboveDefault &&
        readbackNearRequested &&
        delta != null &&
        delta >= 500;
  }

  bool get curveNoRuntimeEffect {
    final delta = cpuRpmDelta;
    return (requestsBelowDefault || requestsAboveDefault) &&
        readbackNearRequested &&
        delta != null &&
        delta.abs() < 500;
  }

  bool get curveTransformed {
    if (requestedPwmValues.length != readbackPwmValues.length) {
      return true;
    }
    for (var i = 0; i < requestedPwmValues.length; i++) {
      if (requestedPwmValues[i] != readbackPwmValues[i]) {
        return true;
      }
    }
    return false;
  }

  double? _interpolatePwm(List<int> pwmValues, int? tempC) {
    if (tempC == null ||
        tempValues.isEmpty ||
        pwmValues.isEmpty ||
        tempValues.length != pwmValues.length) {
      return null;
    }
    if (tempC <= tempValues.first) {
      return pwmValues.first.toDouble();
    }
    for (var i = 1; i < tempValues.length; i++) {
      final upperTemp = tempValues[i];
      if (tempC <= upperTemp) {
        final lowerTemp = tempValues[i - 1];
        final lowerPwm = pwmValues[i - 1];
        final upperPwm = pwmValues[i];
        final span = upperTemp - lowerTemp;
        if (span <= 0) {
          return upperPwm.toDouble();
        }
        final ratio = (tempC - lowerTemp) / span;
        return lowerPwm + (upperPwm - lowerPwm) * ratio;
      }
    }
    return pwmValues.last.toDouble();
  }
}

typedef RuntimeCurveObservation = CurveRuleCaseObservation;

int _minValue(List<int> values) {
  return values.reduce((a, b) => a < b ? a : b);
}

int _maxValue(List<int> values) {
  return values.reduce((a, b) => a > b ? a : b);
}
