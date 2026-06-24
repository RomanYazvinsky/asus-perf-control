import 'dbus_client.dart';

abstract class ITdpService {
  Future<TdpConfigDbus> getTdpConfig();

  Future<void> setPerformanceMode(int mode);

  Future<void> setPowerLimits({required int sustainedW, required int boostW});

  Future<void> clearPowerLimits();

  Future<void> setCpuTempLimit(int tempLimitC);
}
