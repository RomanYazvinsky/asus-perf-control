import '../../bridge_api.dart' as ffi;
import '../asusd_fan_client.dart';
import '../backend_logger.dart';
import 'fan_engine_daemon_client.dart';
import 'fan_models.dart';
import 'fan_service_contract.dart';

class FanServiceImpl implements IFanService {
  final AsusdFanClient _asusdClient;
  FanEngineDaemonClient? _fanEngineClient;
  bool _dynamicEngineStarted = false;
  bool _dynamicEngineKnownRunning = false;

  FanServiceImpl({
    AsusdFanClient? asusdClient,
    FanEngineDaemonClient? fanEngineClient,
  }) : _asusdClient = asusdClient ?? AsusdFanClient(),
       _fanEngineClient = fanEngineClient;

  FanEngineDaemonClient get _daemonClient =>
      _fanEngineClient ??= FanEngineDaemonClient();

  @override
  Future<FanCapabilities> getCapabilities() async {
    return BackendLogger.traceSync('FanServiceImpl.getCapabilities', () {
      final capabilities = ffi.getFanCapabilities();
      return FanCapabilities(
        fans: capabilities.fans
            .map(
              (f) => FanInfo(
                id: f.id,
                label: f.label,
                hasRpm: f.hasRpm,
                hasCurve: f.hasCurve,
              ),
            )
            .toList(),
      );
    });
  }

  @override
  Future<List<FanReading>> getFanRpms() async {
    return BackendLogger.traceSync('FanServiceImpl.getFanRpms', () {
      final readings = ffi.getFanRpms();
      return readings.map((r) => FanReading(id: r.id, rpm: r.rpm)).toList();
    });
  }

  @override
  Future<FanTemperatures> getTemperatures() async {
    return BackendLogger.traceSync('FanServiceImpl.getTemperatures', () {
      final readings = ffi.getSensorReadings();
      return FanTemperatures(
        cpuTempC: readings.cpuTemp?.toInt() ?? 0,
        gpuTempC: readings.gpuTemp?.toInt() ?? 0,
      );
    });
  }

  @override
  Future<void> setActiveProfile(PerfProfile profile) async {
    await _asusdClient.setActiveProfile(profile);
  }

  @override
  Future<List<FanCurveData>> getAsusdCurves(PerfProfile profile) async {
    return _asusdClient.getFanCurves(profile);
  }

  @override
  Future<void> setAsusdCurve(
    PerfProfile profile,
    FanId fan,
    List<FanCurvePoint> points,
  ) async {
    final asusdProfile = asusdProfileMap[profile]!;
    final fanName = fan == FanId.cpu ? 'CPU' : 'GPU';
    final data = FanCurveData.fromCurvePoints(
      name: fanName,
      points: points,
      enabled: true,
    );
    await _asusdClient.setFanCurve(asusdProfile, data);
    await _asusdClient.setFanCurveEnabled(asusdProfile, fanName, true);
  }

  @override
  Future<void> setConstantSpeed(
    PerfProfile profile,
    FanId fan,
    int speedPercent,
  ) async {
    final asusdProfile = asusdProfileMap[profile]!;
    final fanName = fan == FanId.cpu ? 'CPU' : 'GPU';
    final flatCurve = AsusdFanClient.flatCurve(fanName, speedPercent);
    await _asusdClient.setFanCurve(asusdProfile, flatCurve);
    await _asusdClient.setFanCurveEnabled(asusdProfile, fanName, true);
  }

  @override
  Future<void> resetToDefaults(PerfProfile profile) async {
    final asusdProfile = asusdProfileMap[profile]!;
    await _asusdClient.resetCurvesToDefaults(asusdProfile);
  }

  @override
  Future<void> startDynamicEngine(FanEngineConfig config) async {
    await _daemonClient.startFanEngine(config);
    _dynamicEngineStarted = true;
    _dynamicEngineKnownRunning = true;
  }

  @override
  Future<void> stopDynamicEngine() async {
    if (!_dynamicEngineStarted && !_dynamicEngineKnownRunning) {
      return;
    }
    await _daemonClient.stopFanEngine();
    _dynamicEngineStarted = false;
    _dynamicEngineKnownRunning = false;
  }

  @override
  Future<void> updateDynamicCurve(FanId fan, List<FanCurvePoint> curve) async {
    await _daemonClient.updateFanCurve(fan, curve);
  }

  @override
  Future<bool> isDynamicEngineRunning() async {
    final running = await _daemonClient.getRunning();
    _dynamicEngineKnownRunning = running;
    return running;
  }

  @override
  Future<int> runMaxSpeedTest(PerfProfile profile, FanId fan) async {
    final asusdProfile = asusdProfileMap[profile]!;
    final fanName = fan == FanId.cpu ? 'CPU' : 'GPU';

    final currentCurves = await _asusdClient.getFanCurves(profile);
    final currentCurve = currentCurves.firstWhere(
      (c) => c.name == fanName,
      orElse: () => throw StateError('No current curve found for $fanName'),
    );

    final maxCurve = AsusdFanClient.flatCurve(fanName, 100);
    var shouldRestore = false;
    try {
      await _asusdClient.setFanCurve(asusdProfile, maxCurve);
      shouldRestore = true;
      await _asusdClient.setFanCurveEnabled(asusdProfile, fanName, true);

      await Future.delayed(const Duration(seconds: 2));

      final fanId = fan == FanId.cpu ? 'cpu' : 'gpu';
      final readings = await getFanRpms();
      final reading = readings.firstWhere(
        (r) => r.id == fanId,
        orElse: () => FanReading(id: fanId, rpm: null),
      );
      return reading.rpm ?? 0;
    } finally {
      if (shouldRestore) {
        await _asusdClient.setFanCurve(asusdProfile, currentCurve);
        await _asusdClient.setFanCurveEnabled(
          asusdProfile,
          fanName,
          currentCurve.enabled,
        );
      }
    }
  }

  Future<void> close() async {
    await _asusdClient.close();
    await _fanEngineClient?.close();
  }
}
