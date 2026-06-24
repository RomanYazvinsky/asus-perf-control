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
