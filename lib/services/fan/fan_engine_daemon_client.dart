import 'dart:convert';
import 'package:dbus/dbus.dart';
import 'fan_models.dart';
import '../asusd_fan_client.dart';
import '../backend_logger.dart';

/// D-Bus client for ASUS Linux Control Daemon fan engine control.
/// Delegates dynamic fan engine operations to the privileged rust_daemon.
class FanEngineDaemonClient {
  static const String busName = 'com.asuslinuxcontrol.Daemon';
  static const String objectPath = '/com/asuslinuxcontrol/Daemon';
  static const String interfaceName = 'com.asuslinuxcontrol.Daemon';

  final DBusClient _client;
  late final DBusRemoteObject _remoteObject;

  FanEngineDaemonClient() : _client = DBusClient.system() {
    _remoteObject = DBusRemoteObject(
      _client,
      name: busName,
      path: DBusObjectPath(objectPath),
    );
  }

  /// Constructor for testing with injected client.
  FanEngineDaemonClient.withClient(this._client) {
    _remoteObject = DBusRemoteObject(
      _client,
      name: busName,
      path: DBusObjectPath(objectPath),
    );
  }

  /// Start the dynamic fan engine with given configuration.
  /// Requires polkit authorization for com.asuslinuxcontrol.manage-fan-engine.
  Future<void> startFanEngine(FanEngineConfig config) async {
    return BackendLogger.trace(
      'FanEngineDaemonClient.startFanEngine',
      fields: {
        'profile': config.profile.name,
        'tickMs': config.tickMs,
        'cpuPoints': config.cpuCurve.length,
        'gpuPoints': config.gpuCurve.length,
      },
      () async {
        final configJson = jsonEncode(config.toJson());
        final result = await _remoteObject.callMethod(
          interfaceName,
          'StartFanEngine',
          [DBusString(configJson)],
          replySignature: DBusSignature('b'),
        );
        final success = result.returnValues[0].asBoolean();
        if (!success) {
          throw Exception('Failed to start fan engine');
        }
      },
    );
  }

  /// Stop the dynamic fan engine and restore auto mode.
  /// Requires polkit authorization for com.asuslinuxcontrol.manage-fan-engine.
  Future<void> stopFanEngine() async {
    return BackendLogger.trace('FanEngineDaemonClient.stopFanEngine', () async {
      final result = await _remoteObject.callMethod(
        interfaceName,
        'StopFanEngine',
        [],
        replySignature: DBusSignature('b'),
      );
      final success = result.returnValues[0].asBoolean();
      if (!success) {
        throw Exception('Failed to stop fan engine');
      }
    });
  }

  /// Update fan curve while engine is running.
  /// Requires polkit authorization for com.asuslinuxcontrol.manage-fan-engine.
  Future<void> updateFanCurve(FanId fanId, List<FanCurvePoint> curve) async {
    return BackendLogger.trace(
      'FanEngineDaemonClient.updateFanCurve',
      fields: {'fanId': fanId.name, 'points': curve.length},
      () async {
        final fanIdStr = fanId == FanId.cpu ? 'cpu' : 'gpu';
        final curveJson = jsonEncode(fanCurvePointsToJsonPairs(curve));
        final result = await _remoteObject.callMethod(
          interfaceName,
          'UpdateFanEngineCurve',
          [DBusString(fanIdStr), DBusString(curveJson)],
          replySignature: DBusSignature('b'),
        );
        final success = result.returnValues[0].asBoolean();
        if (!success) {
          throw Exception('Failed to update fan curve');
        }
      },
    );
  }

  /// Check if the dynamic fan engine is currently running.
  /// No polkit authorization required (read-only).
  Future<bool> getRunning() async {
    return BackendLogger.trace('FanEngineDaemonClient.getRunning', () async {
      final result = await _remoteObject.callMethod(
        interfaceName,
        'GetFanEngineRunning',
        [],
        replySignature: DBusSignature('b'),
      );
      return result.returnValues[0].asBoolean();
    });
  }

  /// Close D-Bus connection.
  Future<void> close() async {
    await BackendLogger.trace('FanEngineDaemonClient.close', _client.close);
  }
}
