import 'dart:convert';

import 'package:dbus/dbus.dart';

import 'backend_logger.dart';
import 'cpu_boost_service.dart';
import 'dbus_client.dart';

class CpuBoostDaemonClient {
  final DBusClient _client;
  late final DBusRemoteObject _remoteObject;

  CpuBoostDaemonClient() : _client = DBusClient.system() {
    _remoteObject = DBusRemoteObject(
      _client,
      name: DbusClient.busName,
      path: DBusObjectPath(DbusClient.objectPath),
    );
  }

  Future<CpuBoostStatus> getCpuBoostStatus() async {
    return BackendLogger.trace(
      'CpuBoostDaemonClient.getCpuBoostStatus',
      () async {
        final result = await _remoteObject.callMethod(
          DbusClient.interfaceName,
          'GetCpuBoostStatus',
          [],
          replySignature: DBusSignature('s'),
        );
        final payload = jsonDecode(result.returnValues[0].asString());
        if (payload is! Map<String, dynamic>) {
          throw Exception('Invalid CPU boost status payload');
        }
        return _parseStatus(payload);
      },
    );
  }

  Future<void> setCpuBoostMode(CpuBoostMode mode) async {
    await BackendLogger.trace(
      'CpuBoostDaemonClient.setCpuBoostMode',
      fields: {'mode': mode.daemonId},
      () async {
        final result = await _remoteObject.callMethod(
          DbusClient.interfaceName,
          'SetCpuBoostMode',
          [DBusString(mode.daemonId)],
          replySignature: DBusSignature('b'),
        );
        if (!result.returnValues[0].asBoolean()) {
          throw Exception('CPU boost mode was not applied');
        }
      },
    );
  }

  Future<void> close() async {
    await BackendLogger.trace('CpuBoostDaemonClient.close', _client.close);
  }

  CpuBoostStatus _parseStatus(Map<String, dynamic> payload) {
    final availableModeIds = payload['availableModes'];
    final availableModes = availableModeIds is List
        ? availableModeIds
              .whereType<String>()
              .map(CpuBoostMode.fromDaemonId)
              .toList(growable: false)
        : CpuBoostMode.values;

    return CpuBoostStatus(
      isSupported: payload['isSupported'] == true,
      currentMode: CpuBoostMode.fromDaemonId(
        payload['currentMode'] as String? ?? CpuBoostMode.enabled.daemonId,
      ),
      boostEnabled: payload['boostEnabled'] == true,
      epp: payload['epp'] as String?,
      eppWritable: payload['eppWritable'] != false,
      availableModes: availableModes,
    );
  }
}
