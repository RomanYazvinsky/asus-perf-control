import 'package:mocktail/mocktail.dart';
import 'package:flutter_application_1/services/performance_service.dart';
import 'package:flutter_application_1/services/tdp_service.dart';
import 'package:flutter_application_1/services/gpu_service.dart';
import 'package:flutter_application_1/services/dbus_client.dart';
import 'package:flutter_application_1/services/cpu_boost_service.dart';

class MockPerformanceService extends Mock implements IPerformanceService {}

class MockTdpService extends Mock implements ITdpService {}

class MockGpuService extends Mock implements IGpuService {}

class MockCpuBoostService extends Mock implements ICpuBoostService {}

TdpConfigDbus fakeTdpConfig({
  int sustainedW = 20,
  int boostW = 70,
  int tempLimitC = 0,
  int activePerformanceMode = 1,
  int minSustainedW = 15,
  int maxSustainedW = 45,
  int minBoostW = 65,
  int maxBoostW = 80,
  bool powerLimitsEnabled = false,
}) => TdpConfigDbus(
  isSupported: true,
  sustainedW: sustainedW,
  boostW: boostW,
  tempLimitC: tempLimitC,
  activePerformanceMode: activePerformanceMode,
  minSustainedW: minSustainedW,
  maxSustainedW: maxSustainedW,
  minBoostW: minBoostW,
  maxBoostW: maxBoostW,
  powerLimitsEnabled: powerLimitsEnabled,
);
