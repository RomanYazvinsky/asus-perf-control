import 'package:dbus/dbus.dart';

import 'backend_logger.dart';

/// GPU modes from supergfxctl (D-Bus byte values)
enum SupergfxMode {
  hybrid(0),
  integrated(1),
  nvidiaNoModeset(2),
  vfio(3),
  asusEgpu(4), // XG Mobile!
  asusMuxDgpu(5),
  none(6);

  final int value;
  const SupergfxMode(this.value);

  static SupergfxMode fromValue(int value) {
    return SupergfxMode.values.firstWhere(
      (m) => m.value == value,
      orElse: () => SupergfxMode.none,
    );
  }
}

/// User action required after mode change
enum SupergfxUserAction {
  logout(0),
  reboot(1),
  switchToIntegrated(2),
  asusEgpuDisable(3),
  nothing(4);

  final int value;
  const SupergfxUserAction(this.value);

  static SupergfxUserAction fromValue(int value) {
    return SupergfxUserAction.values.firstWhere(
      (a) => a.value == value,
      orElse: () => SupergfxUserAction.nothing,
    );
  }
}

/// D-Bus client for supergfxctl GPU mode switching
class SupergfxClient {
  static const String _busName = 'org.supergfxctl.Daemon';
  static const String _objectPath = '/org/supergfxctl/Gfx';
  static const String _interface = 'org.supergfxctl.Daemon';

  final DBusClient _client;
  late final DBusRemoteObject _remote;

  SupergfxClient() : _client = DBusClient.system() {
    _remote = DBusRemoteObject(
      _client,
      name: _busName,
      path: DBusObjectPath(_objectPath),
    );
  }

  /// Check if supergfxd is available on D-Bus
  Future<bool> isAvailable() async {
    return BackendLogger.trace('SupergfxClient.isAvailable', () async {
      try {
        await getMode();
        return true;
      } catch (e) {
        return false;
      }
    });
  }

  /// Get current GPU mode
  Future<SupergfxMode> getMode() async {
    return BackendLogger.trace('SupergfxClient.getMode', () async {
      final result = await _remote.callMethod(
        _interface,
        'Mode',
        [],
        replySignature: DBusSignature('u'),
      );
      return SupergfxMode.fromValue(result.returnValues[0].asUint32());
    });
  }

  /// Get list of supported GPU modes
  Future<List<SupergfxMode>> getSupportedModes() async {
    return BackendLogger.trace('SupergfxClient.getSupportedModes', () async {
      final result = await _remote.callMethod(
        _interface,
        'Supported',
        [],
        replySignature: DBusSignature('au'),
      );
      final values = (result.returnValues[0] as DBusArray).children
          .map((v) => v.asUint32())
          .toList();
      return values.map((v) => SupergfxMode.fromValue(v)).toList();
    });
  }

  /// Get GPU vendor (Nvidia/AMD/Unknown)
  Future<String> getVendor() async {
    return BackendLogger.trace('SupergfxClient.getVendor', () async {
      try {
        final result = await _remote.callMethod(
          _interface,
          'Vendor',
          [],
          replySignature: DBusSignature('s'),
        );
        return result.returnValues[0].asString();
      } catch (e) {
        return 'Unknown';
      }
    });
  }

  /// Set GPU mode (may require logout/reboot)
  /// Returns required user action
  Future<SupergfxUserAction> setMode(SupergfxMode mode) async {
    return BackendLogger.trace(
      'SupergfxClient.setMode',
      fields: {'mode': mode.name},
      () async {
        final result = await _remote.callMethod(_interface, 'SetMode', [
          DBusUint32(mode.value),
        ], replySignature: DBusSignature('u'));
        return SupergfxUserAction.fromValue(result.returnValues[0].asUint32());
      },
    );
  }

  /// Get pending mode (if reboot required)
  Future<SupergfxMode> getPendingMode() async {
    return BackendLogger.trace('SupergfxClient.getPendingMode', () async {
      final result = await _remote.callMethod(
        _interface,
        'PendingMode',
        [],
        replySignature: DBusSignature('u'),
      );
      return SupergfxMode.fromValue(result.returnValues[0].asUint32());
    });
  }

  /// Get pending user action
  Future<SupergfxUserAction> getPendingUserAction() async {
    return BackendLogger.trace('SupergfxClient.getPendingUserAction', () async {
      final result = await _remote.callMethod(
        _interface,
        'PendingUserAction',
        [],
        replySignature: DBusSignature('u'),
      );
      return SupergfxUserAction.fromValue(result.returnValues[0].asUint32());
    });
  }

  /// Subscribe to GPU mode change notifications
  Stream<SupergfxMode> watchModeChanges() {
    BackendLogger.logEvent('SupergfxClient.watchModeChanges');
    return DBusRemoteObjectSignalStream(
      object: _remote,
      interface: _interface,
      name: 'NotifyGfx',
      signature: DBusSignature('u'),
    ).map((signal) {
      final mode = SupergfxMode.fromValue(signal.values[0].asUint32());
      BackendLogger.logEvent(
        'SupergfxClient.watchModeChanges.signal',
        fields: {'mode': mode.name},
      );
      return mode;
    });
  }

  Future<void> close() async {
    await BackendLogger.trace('SupergfxClient.close', _client.close);
  }
}
