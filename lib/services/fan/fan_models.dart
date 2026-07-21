import '../asusd_fan_client.dart';

enum FanId { cpu, gpu }

enum FanControlMode { asusctlCurve, constantSpeed, dynamicEngine }

typedef FanProfileCurves = Map<FanId, Map<PerfProfile, FanCurveData>>;

class FanEngineConfig {
  final PerfProfile profile;
  final int tickMs;
  final int hysteresisC;
  final int rampDownDelayMs;
  final int emergencyTempC;
  final List<FanCurvePoint> cpuCurve;
  final List<FanCurvePoint> gpuCurve;
  final bool crossFan;
  final List<FanCurvePoint>? crossFanCurve;

  FanEngineConfig({
    this.profile = PerfProfile.balanced,
    this.tickMs = 1000,
    this.hysteresisC = 2,
    this.rampDownDelayMs = 3000,
    this.emergencyTempC = 95,
    required this.cpuCurve,
    required this.gpuCurve,
    this.crossFan = false,
    this.crossFanCurve,
  });

  factory FanEngineConfig.fromJson(Map<String, dynamic> json) {
    return FanEngineConfig(
      profile: _profileFromName(json['profile'] as String?),
      tickMs: _intOrDefault(json['tickMs'], 1000),
      hysteresisC: _intOrDefault(json['hysteresisC'], 2),
      rampDownDelayMs: _intOrDefault(json['rampDownDelayMs'], 3000),
      emergencyTempC: _intOrDefault(json['emergencyTempC'], 95),
      cpuCurve: _curveFromJson(json['cpuCurve']),
      gpuCurve: _curveFromJson(json['gpuCurve']),
      crossFan: json['crossFan'] == true,
      crossFanCurve: json['crossFanCurve'] == null
          ? null
          : _curveFromJson(json['crossFanCurve']),
    );
  }

  Map<String, dynamic> toJson() {
    return {
      'profile': profile.name,
      'tickMs': tickMs,
      'hysteresisC': hysteresisC,
      'rampDownDelayMs': rampDownDelayMs,
      'emergencyTempC': emergencyTempC,
      'cpuCurve': fanCurvePointsToJsonPairs(cpuCurve),
      'gpuCurve': fanCurvePointsToJsonPairs(gpuCurve),
      'crossFan': crossFan,
      'crossFanCurve': crossFanCurve == null
          ? null
          : fanCurvePointsToJsonPairs(crossFanCurve!),
    };
  }
}

PerfProfile _profileFromName(String? name) {
  for (final profile in PerfProfile.values) {
    if (profile.name == name) return profile;
  }
  return PerfProfile.balanced;
}

int _intOrDefault(Object? value, int defaultValue) {
  return value is int ? value : defaultValue;
}

List<FanCurvePoint> _curveFromJson(Object? value) {
  if (value is! List) {
    return _defaultFanCurve();
  }
  final points = <FanCurvePoint>[];
  for (final entry in value) {
    if (entry is List && entry.length >= 2) {
      final temp = entry[0];
      final speed = entry[1];
      if (temp is num && speed is num) {
        points.add(
          FanCurvePoint(
            tempC: temp.toDouble(),
            speedPercent: speed.toDouble().clamp(0, 100).toDouble(),
          ),
        );
      }
    }
  }
  return points.length == 8 ? points : _defaultFanCurve();
}

List<FanCurvePoint> _defaultFanCurve() {
  return List.generate(8, (index) {
    return FanCurvePoint(
      tempC: 20.0 + index * 10.0,
      speedPercent: 30.0 + (index / 7.0) * 70.0,
    );
  });
}

class FanCapabilities {
  final List<FanInfo> fans;

  FanCapabilities({required this.fans});
}

class FanInfo {
  final String id;
  final String label;
  final bool hasRpm;
  final bool hasCurve;

  FanInfo({
    required this.id,
    required this.label,
    required this.hasRpm,
    required this.hasCurve,
  });
}

class FanReading {
  final String id;
  final int? rpm;

  FanReading({required this.id, this.rpm});
}

class FanTemperatures {
  final int cpuTempC;
  final int gpuTempC;

  const FanTemperatures({required this.cpuTempC, required this.gpuTempC});
}
