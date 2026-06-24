import 'package:dbus/dbus.dart';

import 'backend_logger.dart';

/// D-Bus client for ASUS Linux Control Daemon (temp-limit only).
/// All other hardware control goes through AsusdClient.
class DbusClient {
  static const String busName = 'com.asuslinuxcontrol.Daemon';
  static const String objectPath = '/com/asuslinuxcontrol/Daemon';
  static const String interfaceName = 'com.asuslinuxcontrol.Daemon';

  final DBusClient _client;
  late final DBusRemoteObject _remoteObject;

  DbusClient() : _client = DBusClient.system() {
    _remoteObject = DBusRemoteObject(
      _client,
      name: busName,
      path: DBusObjectPath(objectPath),
    );
  }

  /// Get saved CPU temperature limit (0 = no saved limit).
  Future<int> getTempLimit() async {
    return BackendLogger.trace('DbusClient.getTempLimit', () async {
      final result = await _remoteObject.callMethod(
        interfaceName,
        'GetTempLimit',
        [],
        replySignature: DBusSignature('i'),
      );
      return result.returnValues[0].asInt32();
    });
  }

  /// Set CPU temperature limit via our daemon (persists to file).
  /// tempLimitC=0 means "clear saved limit, use default".
  /// Requires polkit authorization.
  Future<bool> setTempLimit(int tempLimitC) async {
    return BackendLogger.trace(
      'DbusClient.setTempLimit',
      fields: {'tempLimitC': tempLimitC},
      () async {
        final result = await _remoteObject.callMethod(
          interfaceName,
          'SetTempLimit',
          [DBusInt32(tempLimitC)],
          replySignature: DBusSignature('b'),
        );
        return result.returnValues[0].asBoolean();
      },
    );
  }

  /// Close D-Bus connection
  Future<void> close() async {
    await BackendLogger.trace('DbusClient.close', _client.close);
  }
}

/// TDP configuration from D-Bus
class TdpConfigDbus {
  final bool isSupported;
  final int sustainedW;
  final int boostW;
  final int tempLimitC;
  final int activePerformanceMode;
  final int minSustainedW;
  final int maxSustainedW;
  final int minBoostW;
  final int maxBoostW;
  final bool powerLimitsEnabled;

  TdpConfigDbus({
    required this.isSupported,
    required this.sustainedW,
    required this.boostW,
    required this.tempLimitC,
    this.activePerformanceMode = 1,
    this.minSustainedW = 5,
    this.maxSustainedW = 54,
    this.minBoostW = 5,
    this.maxBoostW = 54,
    this.powerLimitsEnabled = false,
  });

  @override
  String toString() {
    return 'TdpConfigDbus(supported=$isSupported, sustained=${sustainedW}W, boost=${boostW}W, '
        'temp=$tempLimitC°C, activeMode=$activePerformanceMode, range=$minSustainedW–${maxSustainedW}W, '
        'powerLimitsEnabled=$powerLimitsEnabled)';
  }
}

/// D-Bus client for asusd (xyz.ljones.Asusd)
/// Reads/writes TDP via org.freedesktop.DBus.Properties on AsusArmoury objects.
/// Values are persistent — asusd saves them to its profile.
class AsusdClient {
  static const String _busName = 'xyz.ljones.Asusd';
  static const String _armouryIface = 'xyz.ljones.AsusArmoury';
  static const String _platformIface = 'xyz.ljones.Platform';
  static const String _propsIface = 'org.freedesktop.DBus.Properties';
  static const String _platformPath = '/xyz/ljones';
  static const String _pl1Path = '/xyz/ljones/asus_armoury/ppt_pl1_spl';
  static const String _pl2Path = '/xyz/ljones/asus_armoury/ppt_pl2_sppt';
  // Battery charge limit is on Platform interface, not Armoury
  static const String _chargeLimitProp = 'ChargeControlEndThreshold';
  static const String _enablePptGroupProp = 'EnablePptGroup';

  // asusd PlatformProfile values -> our mode.
  // Balanced=0 -> 1, Performance=1 -> 2 (Turbo), Quiet=2 -> 0 (Silent),
  // LowPower=3 -> 0 (Silent). Write Silent as Quiet because some machines
  // expose LowPower in asusd choices but reject kernel platform_profile=low-power.
  static const _asusdToMode = {0: 1, 1: 2, 2: 0, 3: 0};
  static const _modeToAsusd = {0: 2, 1: 0, 2: 1};

  final DBusClient _client;

  AsusdClient() : _client = DBusClient.system();

  static int modeFromPlatformProfileForTest(int asusdVal) {
    return _modeFromPlatformProfile(asusdVal);
  }

  static int platformProfileFromModeForTest(int mode) {
    return _platformProfileFromMode(mode);
  }

  static int _modeFromPlatformProfile(int asusdVal) {
    return _asusdToMode[asusdVal] ??
        (throw Exception('Unknown PlatformProfile value: $asusdVal'));
  }

  static int _platformProfileFromMode(int mode) {
    return _modeToAsusd[mode] ??
        (throw Exception('Invalid mode: $mode. Must be 0, 1, or 2'));
  }

  /// Get current platform profile as our mode (0=Silent, 1=Balanced, 2=Turbo).
  Future<int> getPerformanceMode() async {
    return BackendLogger.trace('AsusdClient.getPerformanceMode', () async {
      final obj = DBusRemoteObject(
        _client,
        name: _busName,
        path: DBusObjectPath(_platformPath),
      );
      final result = await obj.callMethod(_propsIface, 'Get', [
        DBusString(_platformIface),
        DBusString('PlatformProfile'),
      ], replySignature: DBusSignature('v'));
      final asusdVal = (result.returnValues[0] as DBusVariant).value.asUint32();
      return _modeFromPlatformProfile(asusdVal);
    });
  }

  /// Set platform profile from our mode (0=Silent, 1=Balanced, 2=Turbo).
  Future<void> setPerformanceMode(int mode) async {
    await BackendLogger.trace(
      'AsusdClient.setPerformanceMode',
      fields: {'mode': mode},
      () async {
        final asusdVal = _platformProfileFromMode(mode);
        final obj = DBusRemoteObject(
          _client,
          name: _busName,
          path: DBusObjectPath(_platformPath),
        );
        await obj.callMethod(_propsIface, 'Set', [
          DBusString(_platformIface),
          DBusString('PlatformProfile'),
          DBusVariant(DBusUint32(asusdVal)),
        ], replySignature: DBusSignature(''));
      },
    );
  }

  /// Check if a D-Bus object path exists.
  Future<bool> _objectExists(String path) async {
    return BackendLogger.trace(
      'AsusdClient.objectExists',
      fields: {'path': path},
      () async {
        try {
          final obj = DBusRemoteObject(
            _client,
            name: _busName,
            path: DBusObjectPath(path),
          );
          await obj.callMethod(
            'org.freedesktop.DBus.Introspectable',
            'Introspect',
            [],
            replySignature: DBusSignature('s'),
          );
          return true;
        } catch (_) {
          return false;
        }
      },
    );
  }

  /// Get current TDP config from asusd.
  /// Returns real MinValue/MaxValue from hardware (model-specific).
  /// Note: ppt_pl2_sppt (boost) is optional - not all laptops have it.
  Future<TdpConfigDbus> getTdpConfig() async {
    return BackendLogger.trace('AsusdClient.getTdpConfig', () async {
      try {
        final pl1 = DBusRemoteObject(
          _client,
          name: _busName,
          path: DBusObjectPath(_pl1Path),
        );

        final sustainedW = await _getProp(pl1, 'CurrentValue');
        final sustainedMin = await _getProp(pl1, 'MinValue');
        final sustainedMax = await _getProp(pl1, 'MaxValue');
        bool powerLimitsEnabled = false;
        try {
          powerLimitsEnabled = await getPowerLimitsEnabled();
        } catch (_) {
          powerLimitsEnabled = false;
        }

        // Boost (ppt_pl2_sppt) is optional - not all laptops have it
        int boostW = 0;
        int boostMin = 0;
        int boostMax = 0;

        if (await _objectExists(_pl2Path)) {
          final pl2 = DBusRemoteObject(
            _client,
            name: _busName,
            path: DBusObjectPath(_pl2Path),
          );
          boostW = await _getProp(pl2, 'CurrentValue');
          boostMin = await _getProp(pl2, 'MinValue');
          boostMax = await _getProp(pl2, 'MaxValue');
        }

        return TdpConfigDbus(
          isSupported: true,
          sustainedW: sustainedW,
          boostW: boostW,
          tempLimitC: 0,
          minSustainedW: sustainedMin,
          maxSustainedW: sustainedMax,
          minBoostW: boostMin,
          maxBoostW: boostMax,
          powerLimitsEnabled: powerLimitsEnabled,
        );
      } catch (e) {
        final errorStr = e.toString();
        if (errorStr.contains('UnknownInterface') ||
            errorStr.contains('No such interface') ||
            errorStr.contains('UnknownMethod') ||
            errorStr.contains('InvalidArgs')) {
          throw Exception(
            'asusd TDP interface not found. '
            'Ensure asusd is running (systemctl status asusd) and '
            'kernel supports armoury class (5.17+). Original error: $e',
          );
        }
        throw Exception(
          'Failed to communicate with asusd. '
          'Is asusd running? (systemctl status asusd). Error: $e',
        );
      }
    });
  }

  /// Set TDP via asusd. Changes are persistent across reboots.
  /// Note: Boost is optional - only set if laptop supports ppt_pl2_sppt.
  Future<void> setTdpConfig({
    required int sustainedW,
    required int boostW,
  }) async {
    await BackendLogger.trace(
      'AsusdClient.setTdpConfig',
      fields: {'sustainedW': sustainedW, 'boostW': boostW},
      () async {
        try {
          await _setProp(_pl1Path, 'CurrentValue', sustainedW);
        } catch (e) {
          throw Exception('PL1/SPL set to ${sustainedW}W failed: $e');
        }

        // Only set boost if the object exists (not all laptops have ppt_pl2_sppt)
        if (await _objectExists(_pl2Path)) {
          try {
            await _setProp(_pl2Path, 'CurrentValue', boostW);
          } catch (e) {
            throw Exception('PL2/SPPT set to ${boostW}W failed: $e');
          }
        }
      },
    );
  }

  Future<void> resetPowerLimitsToProfileDefaults() async {
    await BackendLogger.trace(
      'AsusdClient.resetPowerLimitsToProfileDefaults',
      () async {
        await _restoreArmouryDefault(_pl1Path);
        if (await _objectExists(_pl2Path)) {
          await _restoreArmouryDefault(_pl2Path);
        }
      },
    );
  }

  Future<bool> getPowerLimitsEnabled() async {
    return BackendLogger.trace('AsusdClient.getPowerLimitsEnabled', () async {
      final obj = DBusRemoteObject(
        _client,
        name: _busName,
        path: DBusObjectPath(_platformPath),
      );
      final result = await obj.callMethod(_propsIface, 'Get', [
        DBusString(_platformIface),
        DBusString(_enablePptGroupProp),
      ], replySignature: DBusSignature('v'));
      return (result.returnValues[0] as DBusVariant).value.asBoolean();
    });
  }

  Future<void> setPowerLimitsEnabled(bool enabled) async {
    await BackendLogger.trace(
      'AsusdClient.setPowerLimitsEnabled',
      fields: {'enabled': enabled},
      () async {
        final obj = DBusRemoteObject(
          _client,
          name: _busName,
          path: DBusObjectPath(_platformPath),
        );
        await obj.callMethod(_propsIface, 'Set', [
          DBusString(_platformIface),
          DBusString(_enablePptGroupProp),
          DBusVariant(DBusBoolean(enabled)),
        ], replySignature: DBusSignature(''));
      },
    );
  }

  Future<int> _getProp(DBusRemoteObject obj, String prop) async {
    return BackendLogger.trace(
      'AsusdClient.getProp',
      fields: {'prop': prop},
      () async {
        final result = await obj.callMethod(_propsIface, 'Get', [
          DBusString(_armouryIface),
          DBusString(prop),
        ], replySignature: DBusSignature('v'));
        return (result.returnValues[0] as DBusVariant).value.asInt32();
      },
    );
  }

  Future<void> _setProp(String path, String prop, int value) async {
    await BackendLogger.trace(
      'AsusdClient.setProp',
      fields: {'path': path, 'prop': prop, 'value': value},
      () async {
        final obj = DBusRemoteObject(
          _client,
          name: _busName,
          path: DBusObjectPath(path),
        );
        await obj.callMethod(_propsIface, 'Set', [
          DBusString(_armouryIface),
          DBusString(prop),
          DBusVariant(DBusInt32(value)),
        ], replySignature: DBusSignature(''));
      },
    );
  }

  Future<void> _restoreArmouryDefault(String path) async {
    await BackendLogger.trace(
      'AsusdClient.restoreArmouryDefault',
      fields: {'path': path},
      () async {
        final obj = DBusRemoteObject(
          _client,
          name: _busName,
          path: DBusObjectPath(path),
        );
        await obj.callMethod(
          _armouryIface,
          'RestoreDefault',
          [],
          replySignature: DBusSignature(''),
        );
      },
    );
  }

  /// Check if battery charge limit control is supported (ChargeControlEndThreshold property exists).
  Future<bool> isChargeLimitSupported() async {
    return BackendLogger.trace('AsusdClient.isChargeLimitSupported', () async {
      try {
        final obj = DBusRemoteObject(
          _client,
          name: _busName,
          path: DBusObjectPath(_platformPath),
        );
        await _getPlatformProp(obj, _chargeLimitProp);
        return true;
      } catch (_) {
        return false;
      }
    });
  }

  /// Get current battery charge limit (0-100, 0 = no limit) via Platform interface.
  Future<int> getChargeLimit() async {
    return BackendLogger.trace('AsusdClient.getChargeLimit', () async {
      try {
        final obj = DBusRemoteObject(
          _client,
          name: _busName,
          path: DBusObjectPath(_platformPath),
        );
        return await _getPlatformProp(obj, _chargeLimitProp);
      } catch (e) {
        throw Exception('Failed to get battery charge limit from asusd: $e');
      }
    });
  }

  /// Set battery charge limit (0-100, 0 = disable limit). Requires polkit authorization.
  Future<void> setChargeLimit(int limit) async {
    await BackendLogger.trace(
      'AsusdClient.setChargeLimit',
      fields: {'limit': limit},
      () async {
        if (limit < 0 || limit > 100) {
          throw Exception('Charge limit must be 0-100%, got $limit');
        }
        try {
          final obj = DBusRemoteObject(
            _client,
            name: _busName,
            path: DBusObjectPath(_platformPath),
          );
          await _setPlatformProp(obj, _chargeLimitProp, limit);
        } catch (e) {
          throw Exception('Failed to set battery charge limit via asusd: $e');
        }
      },
    );
  }

  Future<int> _getPlatformProp(DBusRemoteObject obj, String prop) async {
    return BackendLogger.trace(
      'AsusdClient.getPlatformProp',
      fields: {'prop': prop},
      () async {
        final result = await obj.callMethod(_propsIface, 'Get', [
          DBusString(_platformIface),
          DBusString(prop),
        ], replySignature: DBusSignature('v'));
        final value = (result.returnValues[0] as DBusVariant).value;
        // ChargeControlEndThreshold returns DBusByte (y), not DBusUint32 (u)
        if (value is DBusByte) {
          return value.value;
        }
        return value.asUint32();
      },
    );
  }

  Future<void> _setPlatformProp(
    DBusRemoteObject obj,
    String prop,
    int value,
  ) async {
    await BackendLogger.trace(
      'AsusdClient.setPlatformProp',
      fields: {'prop': prop, 'value': value},
      () => obj.callMethod(_propsIface, 'Set', [
        DBusString(_platformIface),
        DBusString(prop),
        DBusVariant(
          DBusByte(value),
        ), // ChargeControlEndThreshold expects byte (y)
      ], replySignature: DBusSignature('')),
    );
  }

  Future<void> close() async {
    await BackendLogger.trace('AsusdClient.close', _client.close);
  }
}
