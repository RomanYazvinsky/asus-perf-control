import '../asusd_fan_client.dart';
import 'fan_models.dart';

abstract class IFanService {
  Future<FanCapabilities> getCapabilities();
  Future<List<FanReading>> getFanRpms();
  Future<FanTemperatures> getTemperatures();

  Future<void> setActiveProfile(PerfProfile profile);

  Future<List<FanCurveData>> getAsusdCurves(PerfProfile profile);
  Future<void> setAsusdCurve(
    PerfProfile profile,
    FanId fan,
    List<FanCurvePoint> points,
  );
  Future<void> setConstantSpeed(
    PerfProfile profile,
    FanId fan,
    int speedPercent,
  );
  Future<void> resetToDefaults(PerfProfile profile);

  Future<void> startDynamicEngine(FanEngineConfig config);
  Future<void> stopDynamicEngine();
  Future<void> updateDynamicCurve(FanId fan, List<FanCurvePoint> curve);
  Future<bool> isDynamicEngineRunning();

  Future<int> runMaxSpeedTest(PerfProfile profile, FanId fan);
}
