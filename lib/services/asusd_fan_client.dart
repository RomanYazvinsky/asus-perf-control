// D-Bus client for asusd FanCurves interface
// This is the PRIMARY control path for fan curves (including constant speed via flat curves)

import 'package:dbus/dbus.dart';

import 'backend_logger.dart';

/// Performance profile mapping to asusd profile IDs
enum PerfProfile { balanced, performance, quiet, lowPower }

/// asusd profile ID mapping
const asusdProfileMap = {
  PerfProfile.balanced: 0,
  PerfProfile.performance: 1,
  PerfProfile.quiet: 2,
  PerfProfile.lowPower: 3,
};

/// Fan curve data as returned by asusd
class FanCurveData {
  final String name; // "CPU" or "GPU"
  final List<int> pwmValues; // 8 PWM values (0-255)
  final List<int> tempValues; // 8 temperature values (°C)
  final bool enabled;

  FanCurveData({
    required this.name,
    required this.pwmValues,
    required this.tempValues,
    required this.enabled,
  });

  factory FanCurveData.fromDBusStruct(DBusStruct values) {
    final name = (values.children[0] as DBusString).value;
    // Handle both DBusVariant wrapping and direct values
    final pwmItem = values.children[1];
    final pwmArray = pwmItem is DBusVariant ? pwmItem.value : pwmItem;
    final tempItem = values.children[2];
    final tempArray = tempItem is DBusVariant ? tempItem.value : tempItem;
    final enabledItem = values.children[3];
    final enabled = enabledItem is DBusVariant
        ? (enabledItem.value as DBusBoolean).value
        : (enabledItem as DBusBoolean).value;

    // Handle both DBusArray and direct struct for byte arrays
    final pwmValues = pwmArray is DBusArray
        ? pwmArray.children.map((e) => (e as DBusByte).value).toList()
        : (pwmArray as DBusStruct).children
              .map((e) => (e as DBusByte).value)
              .toList();
    final tempValues = tempArray is DBusArray
        ? tempArray.children.map((e) => (e as DBusByte).value).toList()
        : (tempArray as DBusStruct).children
              .map((e) => (e as DBusByte).value)
              .toList();

    return FanCurveData(
      name: name,
      pwmValues: pwmValues,
      tempValues: tempValues,
      enabled: enabled,
    );
  }

  DBusStruct toDBusStruct() {
    return DBusStruct([
      DBusString(name),
      DBusStruct(pwmValues.map((v) => DBusByte(v)).toList()),
      DBusStruct(tempValues.map((v) => DBusByte(v)).toList()),
      DBusBoolean(enabled),
    ]);
  }

  /// Convert to list of curve points (temp, speed_percent)
  List<FanCurvePoint> toCurvePoints() {
    return List.generate(
      8,
      (i) => FanCurvePoint(
        tempC: tempValues[i].toDouble(),
        speedPercent: (pwmValues[i] / 255 * 100).round().toDouble(),
      ),
    );
  }

  /// Create from list of curve points
  factory FanCurveData.fromCurvePoints({
    required String name,
    required List<FanCurvePoint> points,
    required bool enabled,
  }) {
    if (points.length != 8) {
      throw ArgumentError('FanCurveData requires exactly 8 points');
    }

    return FanCurveData(
      name: name,
      pwmValues: points
          .map((p) => (p.speedPercent / 100 * 255).round())
          .toList(),
      tempValues: normalizedRoundedFanCurveTemps(points),
      enabled: enabled,
    );
  }

  @override
  String toString() {
    return 'FanCurveData(name: $name, enabled: $enabled, points: ${toCurvePoints()})';
  }
}

/// Fan curve point
class FanCurvePoint {
  final double tempC; // Temperature in °C
  final double speedPercent; // Speed percentage (0-100)

  FanCurvePoint({required this.tempC, required this.speedPercent});

  @override
  String toString() => '($tempC°C, $speedPercent%)';
}

List<List<int>> fanCurvePointsToJsonPairs(List<FanCurvePoint> curve) {
  final temps = normalizedRoundedFanCurveTemps(curve);
  return [
    for (var i = 0; i < curve.length; i++)
      [temps[i], curve[i].speedPercent.round().clamp(0, 100)],
  ];
}

List<int> normalizedRoundedFanCurveTemps(List<FanCurvePoint> curve) {
  final temps = curve.map((point) => point.tempC.round()).toList();
  for (var i = 1; i < temps.length; i++) {
    if (temps[i] <= temps[i - 1]) {
      temps[i] = temps[i - 1] + 1;
    }
  }
  return temps;
}

/// D-Bus client for asusd FanCurves interface
class AsusdFanClient {
  static const String _busName = 'xyz.ljones.Asusd';
  static const String _objectPath = '/xyz/ljones';
  static const String _interfaceName = 'xyz.ljones.FanCurves';
  static const String _platformInterfaceName = 'xyz.ljones.Platform';
  static const String _propsInterfaceName = 'org.freedesktop.DBus.Properties';

  final DBusClient _client;
  late final DBusRemoteObject _fanCurves;

  AsusdFanClient() : _client = DBusClient.system() {
    _fanCurves = DBusRemoteObject(
      _client,
      name: _busName,
      path: DBusObjectPath(_objectPath),
    );
  }

  Future<void> setActiveProfile(PerfProfile profile) async {
    await BackendLogger.trace(
      'AsusdFanClient.setActiveProfile',
      fields: {'profile': profile.name},
      () => _fanCurves.callMethod(_propsInterfaceName, 'Set', [
        DBusString(_platformInterfaceName),
        DBusString('PlatformProfile'),
        DBusVariant(DBusUint32(asusdProfileMap[profile]!)),
      ], replySignature: DBusSignature('')),
    );
  }

  /// Read all fan curves for a profile
  /// Returns list of FanCurveData for CPU and GPU fans
  Future<List<FanCurveData>> getFanCurves(PerfProfile profile) async {
    return BackendLogger.trace(
      'AsusdFanClient.getFanCurves',
      fields: {'profile': profile.name},
      () async {
        final asusdProfile = asusdProfileMap[profile]!;
        final result = await _fanCurves.callMethod(
          _interfaceName,
          'FanCurveData',
          [DBusUint32(asusdProfile)],
          replySignature: DBusSignature('a(s(yyyyyyyy)(yyyyyyyy)b)'),
        );

        final returnValue = result.returnValues[0];

        // Handle both array and single struct
        if (returnValue is DBusArray) {
          return returnValue.children.map((item) {
            // Handle both DBusVariant wrapping and direct DBusStruct
            if (item is DBusVariant) {
              return FanCurveData.fromDBusStruct(item.value as DBusStruct);
            } else {
              return FanCurveData.fromDBusStruct(item as DBusStruct);
            }
          }).toList();
        } else if (returnValue is DBusStruct) {
          // Single struct returned
          return [FanCurveData.fromDBusStruct(returnValue)];
        } else {
          throw FormatException(
            'Unexpected D-Bus return type: ${returnValue.runtimeType}',
          );
        }
      },
    );
  }

  /// Set fan curve for a specific fan in a profile
  /// To set constant speed: pass flat curve with all 8 pwm values identical
  Future<void> setFanCurve(int asusdProfile, FanCurveData data) async {
    await BackendLogger.trace(
      'AsusdFanClient.setFanCurve',
      fields: {
        'asusdProfile': asusdProfile,
        'fan': data.name,
        'enabled': data.enabled,
      },
      () => _fanCurves.callMethod(_interfaceName, 'SetFanCurve', [
        DBusUint32(asusdProfile),
        data.toDBusStruct(),
      ], replySignature: DBusSignature('')),
    );
  }

  /// Enable or disable a fan's custom curve
  /// false = factory auto mode
  Future<void> setFanCurveEnabled(
    int asusdProfile,
    String fanName,
    bool enabled,
  ) async {
    await BackendLogger.trace(
      'AsusdFanClient.setFanCurveEnabled',
      fields: {
        'asusdProfile': asusdProfile,
        'fan': fanName,
        'enabled': enabled,
      },
      () => _fanCurves.callMethod(
        _interfaceName,
        'SetProfileFanCurveEnabled',
        [DBusUint32(asusdProfile), DBusString(fanName), DBusBoolean(enabled)],
        replySignature: DBusSignature(''),
      ),
    );
  }

  /// Reset all curves for a profile to factory defaults
  Future<void> resetCurvesToDefaults(int asusdProfile) async {
    await BackendLogger.trace(
      'AsusdFanClient.resetCurvesToDefaults',
      fields: {'asusdProfile': asusdProfile},
      () => _fanCurves.callMethod(_interfaceName, 'SetCurvesToDefaults', [
        DBusUint32(asusdProfile),
      ], replySignature: DBusSignature('')),
    );
  }

  /// Create a flat curve for constant speed mode
  /// All 8 points have the same PWM value
  static FanCurveData flatCurve(String fanName, int speedPercent) {
    final pwm = (speedPercent * 255 / 100).clamp(0, 255).round();
    return FanCurveData(
      name: fanName,
      pwmValues: List.filled(8, pwm),
      tempValues: [30, 40, 50, 60, 70, 80, 90, 100],
      enabled: true,
    );
  }

  Future<void> close() async {
    await BackendLogger.trace('AsusdFanClient.close', _client.close);
  }
}
