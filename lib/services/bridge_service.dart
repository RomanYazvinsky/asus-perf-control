// Flutter bridge service wrapper
// Provides a clean interface to asusd D-Bus calls

import 'battery_service.dart';
import 'cpu_boost_daemon_client.dart';
import 'cpu_boost_service.dart';
import 'dbus_client.dart';
import 'package_update_service.dart';
import 'performance_service.dart';
import 'tdp_service.dart';

class BridgeService
    implements
        IPerformanceService,
        ITdpService,
        IBatteryService,
        ICpuBoostService {
  // Our daemon client: temperature limit (persistent via file)
  final DbusClient _dbusClient;
  // Our daemon client: CPU boost mode sysfs controls (persistent via file)
  final CpuBoostDaemonClient _cpuBoostClient;
  // asusd client: performance mode + TDP (persistent, hardware-aware)
  final AsusdClient _asusdClient;
  final PackageUpdateService packageUpdates;

  // Singleton pattern
  static final BridgeService _instance = BridgeService._internal();
  factory BridgeService() => _instance;

  BridgeService._internal()
    : _dbusClient = DbusClient(),
      _cpuBoostClient = CpuBoostDaemonClient(),
      _asusdClient = AsusdClient(),
      packageUpdates = DbusPackageUpdateService();

  BridgeService.test({
    required DbusClient dbusClient,
    required AsusdClient asusdClient,
    CpuBoostDaemonClient? cpuBoostClient,
    PackageUpdateService? packageUpdateService,
  }) : _dbusClient = dbusClient,
       _cpuBoostClient = cpuBoostClient ?? CpuBoostDaemonClient(),
       _asusdClient = asusdClient,
       packageUpdates = packageUpdateService ?? DbusPackageUpdateService();

  /// Get current performance mode (0=Silent, 1=Balanced, 2=Turbo) via asusd.
  @override
  Future<int> getPerformanceMode() async {
    try {
      return await _asusdClient.getPerformanceMode();
    } catch (e) {
      throw Exception(
        'Failed to get performance mode from asusd: $e. Is asusd running?',
      );
    }
  }

  /// Set performance mode (0=Silent, 1=Balanced, 2=Turbo) via asusd.
  @override
  Future<void> setPerformanceMode(int mode) async {
    try {
      await _asusdClient.setPerformanceMode(mode);
    } catch (e) {
      throw Exception('Failed to set performance mode via asusd: $e');
    }
  }

  // ---------------------------------------------------------------------------
  // TDP & CPU Temperature Limits (User Story 7)
  // Uses D-Bus daemon with polkit authorization instead of direct sysfs access
  // ---------------------------------------------------------------------------

  /// Get current TDP limits from asusd + saved temp limit from our daemon.
  @override
  Future<TdpConfigDbus> getTdpConfig() async {
    TdpConfigDbus config;
    try {
      config = await _asusdClient.getTdpConfig();
    } catch (e) {
      throw Exception(
        'Failed to get TDP config from asusd: $e. Is asusd running?',
      );
    }

    // Merge saved temperature limit from our daemon
    int tempLimitC = 0;
    try {
      tempLimitC = await _dbusClient.getTempLimit();
    } catch (e) {
      // Non-critical: if daemon is down, just use 0 (no saved limit)
    }
    int activePerformanceMode = 1;
    try {
      activePerformanceMode = await _asusdClient.getPerformanceMode();
    } catch (e) {
      // Non-critical for TDP controls; keep Balanced as a neutral fallback.
    }

    return TdpConfigDbus(
      isSupported: config.isSupported,
      sustainedW: config.sustainedW,
      boostW: config.boostW,
      tempLimitC: tempLimitC,
      activePerformanceMode: activePerformanceMode,
      minSustainedW: config.minSustainedW,
      maxSustainedW: config.maxSustainedW,
      minBoostW: config.minBoostW,
      maxBoostW: config.maxBoostW,
      powerLimitsEnabled: config.powerLimitsEnabled,
    );
  }

  /// Apply TDP power limits via asusd. Persistent across reboots through asusd.
  @override
  Future<void> setPowerLimits({
    required int sustainedW,
    required int boostW,
  }) async {
    try {
      await _asusdClient.setTdpConfig(sustainedW: sustainedW, boostW: boostW);
      await _asusdClient.setPowerLimitsEnabled(true);
    } catch (e) {
      throw Exception('Failed to set TDP via asusd: $e');
    }
  }

  @override
  Future<void> clearPowerLimits() async {
    try {
      await _asusdClient.resetPowerLimitsToProfileDefaults();
      await _asusdClient.setPowerLimitsEnabled(false);
    } catch (e) {
      throw Exception('Failed to clear TDP power limits: $e');
    }
  }

  /// Apply CPU temperature limit via our daemon. Persistent through daemon file.
  @override
  Future<void> setCpuTempLimit(int tempLimitC) async {
    try {
      await _dbusClient.setTempLimit(tempLimitC);
    } catch (e) {
      throw Exception('Failed to set temperature limit via daemon: $e');
    }
  }

  // ---------------------------------------------------------------------------
  // CPU Boost Modes
  // Uses our daemon because these controls require privileged sysfs writes.
  // ---------------------------------------------------------------------------

  @override
  Future<CpuBoostStatus> getCpuBoostStatus() async {
    try {
      return await _cpuBoostClient.getCpuBoostStatus();
    } catch (e) {
      throw Exception('Failed to get CPU boost status via daemon: $e');
    }
  }

  @override
  Future<void> setCpuBoostMode(CpuBoostMode mode) async {
    try {
      await _cpuBoostClient.setCpuBoostMode(mode);
    } catch (e) {
      throw Exception('Failed to set CPU boost mode via daemon: $e');
    }
  }

  // ---------------------------------------------------------------------------
  // Battery Charge Limit (User Story 3)
  // Uses asusd D-Bus with polkit authorization (no direct sysfs access)
  // ---------------------------------------------------------------------------

  /// Get current battery charge limit via asusd
  @override
  Future<int> getChargeLimit() async {
    try {
      return await _asusdClient.getChargeLimit();
    } catch (e) {
      throw Exception('Failed to get battery charge limit: $e');
    }
  }

  /// Set battery charge limit via asusd (requires polkit authorization)
  @override
  Future<void> setChargeLimit(int limit) async {
    try {
      await _asusdClient.setChargeLimit(limit);
    } catch (e) {
      throw Exception('Failed to set battery charge limit: $e');
    }
  }

  /// Get battery status (limit only via asusd)
  @override
  Future<BatteryStatus> getBatteryStatus() async {
    try {
      final limit = await _asusdClient.getChargeLimit();
      return BatteryStatus(chargeLimit: limit, percentage: null, state: null);
    } catch (e) {
      throw Exception('Failed to get battery status: $e');
    }
  }

  /// Check if charge limit control is available via asusd
  @override
  Future<bool> isChargeLimitSupported() async {
    return await _asusdClient.isChargeLimitSupported();
  }

  /// Dispose resources
  Future<void> dispose() async {
    await _dbusClient.close();
    await _cpuBoostClient.close();
    await _asusdClient.close();
    await packageUpdates.close();
  }
}
