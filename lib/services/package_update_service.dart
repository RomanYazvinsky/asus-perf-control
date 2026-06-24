import 'dart:convert';

import 'package:dbus/dbus.dart';

import 'backend_logger.dart';
import 'dbus_client.dart';

typedef PackageUpdateMethodCaller =
    Future<List<DBusValue>> Function(
      String method,
      List<DBusValue> args,
      DBusSignature replySignature,
    );

enum PackageUpdateBackend {
  fedoraDnf('fedora-dnf'),
  fedoraDnf5('fedora-dnf5'),
  bazziteRpmOstree('bazzite-rpm-ostree'),
  archPacman('arch-pacman'),
  steamOsPacman('steamos-pacman'),
  unsupported('unsupported');

  const PackageUpdateBackend(this.wireName);

  final String wireName;

  static PackageUpdateBackend fromWire(String value) =>
      PackageUpdateBackend.values.firstWhere(
        (backend) => backend.wireName == value,
        orElse: () => throw FormatException('Unknown package backend: $value'),
      );
}

enum PackageUpdateRunState {
  idle('idle'),
  checking('checking'),
  available('available'),
  upToDate('upToDate'),
  running('running'),
  succeeded('succeeded'),
  failed('failed'),
  unsupported('unsupported');

  const PackageUpdateRunState(this.wireName);

  final String wireName;

  static PackageUpdateRunState fromWire(String value) =>
      PackageUpdateRunState.values.firstWhere(
        (state) => state.wireName == value,
        orElse: () => throw FormatException('Unknown update state: $value'),
      );
}

enum PackageUpdateErrorCode {
  unsupportedOs('unsupported_os'),
  missingRepository('missing_repository'),
  packageManagerUnavailable('package_manager_unavailable'),
  packageDatabaseLocked('package_database_locked'),
  signatureError('signature_error'),
  networkError('network_error'),
  authorizationDenied('authorization_denied'),
  updateFailed('update_failed'),
  rebootRequired('reboot_required');

  const PackageUpdateErrorCode(this.wireName);

  final String wireName;

  static PackageUpdateErrorCode fromWire(String value) =>
      PackageUpdateErrorCode.values.firstWhere(
        (code) => code.wireName == value,
        orElse: () =>
            throw FormatException('Unknown update error code: $value'),
      );
}

class PackageUpdateStatus {
  const PackageUpdateStatus({
    required this.schemaVersion,
    required this.backend,
    required this.state,
    required this.installedVersion,
    required this.availableVersion,
    required this.rebootRequired,
    required this.summary,
    required this.errorCode,
    required this.logTail,
  });

  final int schemaVersion;
  final PackageUpdateBackend backend;
  final PackageUpdateRunState state;
  final String? installedVersion;
  final String? availableVersion;
  final bool? rebootRequired;
  final String summary;
  final PackageUpdateErrorCode? errorCode;
  final List<String> logTail;

  static PackageUpdateStatus fromJsonString(String value) {
    final decoded = jsonDecode(value);
    if (decoded is! Map<String, dynamic>) {
      throw const FormatException(
        'Package update status must be a JSON object',
      );
    }
    return PackageUpdateStatus.fromJson(decoded);
  }

  factory PackageUpdateStatus.fromJson(Map<String, dynamic> json) {
    final schemaVersion = json['schemaVersion'];
    if (schemaVersion != 1) {
      throw FormatException('Unsupported update status schema: $schemaVersion');
    }

    final errorCode = json['errorCode'];
    final logTail = json['logTail'];

    return PackageUpdateStatus(
      schemaVersion: schemaVersion as int,
      backend: PackageUpdateBackend.fromWire(json['backend'] as String),
      state: PackageUpdateRunState.fromWire(json['state'] as String),
      installedVersion: json['installedVersion'] as String?,
      availableVersion: json['availableVersion'] as String?,
      rebootRequired: json['rebootRequired'] as bool?,
      summary: json['summary'] as String,
      errorCode: errorCode == null
          ? null
          : PackageUpdateErrorCode.fromWire(errorCode as String),
      logTail: logTail is List
          ? logTail.map((line) => line.toString()).toList(growable: false)
          : const [],
    );
  }
}

abstract class PackageUpdateService {
  Future<PackageUpdateBackend> getPackageBackend();
  Future<PackageUpdateStatus> checkForUpdates();
  Future<String> startUpdate();
  Future<PackageUpdateStatus> getUpdateStatus(String updateId);
  Future<bool> cancelUpdate(String updateId);
  Future<void> close();
}

class PackageUpdateServiceException implements Exception {
  const PackageUpdateServiceException(this.message);

  final String message;

  @override
  String toString() => 'PackageUpdateServiceException: $message';
}

class DbusPackageUpdateService implements PackageUpdateService {
  DbusPackageUpdateService()
    : _client = DBusClient.system(),
      _methodCaller = null {
    _remoteObject = DBusRemoteObject(
      _client!,
      name: DbusClient.busName,
      path: DBusObjectPath(DbusClient.objectPath),
    );
  }

  DbusPackageUpdateService.test({
    required PackageUpdateMethodCaller methodCaller,
  }) : _client = null,
       _methodCaller = methodCaller;

  final DBusClient? _client;
  final PackageUpdateMethodCaller? _methodCaller;
  late final DBusRemoteObject _remoteObject;

  @override
  Future<PackageUpdateBackend> getPackageBackend() async {
    final values = await _call(
      'GetPackageBackend',
      const [],
      DBusSignature('s'),
    );
    return PackageUpdateBackend.fromWire(values.single.asString());
  }

  @override
  Future<PackageUpdateStatus> checkForUpdates() async {
    final values = await _call('CheckForUpdates', const [], DBusSignature('s'));
    return PackageUpdateStatus.fromJsonString(values.single.asString());
  }

  @override
  Future<String> startUpdate() async {
    final values = await _call('StartUpdate', const [], DBusSignature('s'));
    return values.single.asString();
  }

  @override
  Future<PackageUpdateStatus> getUpdateStatus(String updateId) async {
    final values = await _call('GetUpdateStatus', [
      DBusString(updateId),
    ], DBusSignature('s'));
    return PackageUpdateStatus.fromJsonString(values.single.asString());
  }

  @override
  Future<bool> cancelUpdate(String updateId) async {
    final values = await _call('CancelUpdate', [
      DBusString(updateId),
    ], DBusSignature('b'));
    return values.single.asBoolean();
  }

  Future<List<DBusValue>> _call(
    String method,
    List<DBusValue> args,
    DBusSignature replySignature,
  ) async {
    return BackendLogger.trace(
      'DbusPackageUpdateService.$method',
      fields: {'args': args.length},
      () async {
        try {
          final methodCaller = _methodCaller;
          if (methodCaller != null) {
            return await methodCaller(method, args, replySignature);
          }

          final result = await _remoteObject.callMethod(
            DbusClient.interfaceName,
            method,
            args,
            replySignature: replySignature,
          );
          return result.returnValues;
        } catch (e) {
          throw PackageUpdateServiceException(
            'Package update daemon unavailable: $e',
          );
        }
      },
    );
  }

  @override
  Future<void> close() async {
    await _client?.close();
  }
}
