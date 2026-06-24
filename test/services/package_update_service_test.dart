import 'package:dbus/dbus.dart';
import 'package:flutter_test/flutter_test.dart';

import 'package:flutter_application_1/services/package_update_service.dart';

void main() {
  group('PackageUpdateStatus parsing', () {
    test('parses every backend enum string', () {
      expect(
        PackageUpdateBackend.fromWire('fedora-dnf'),
        PackageUpdateBackend.fedoraDnf,
      );
      expect(
        PackageUpdateBackend.fromWire('fedora-dnf5'),
        PackageUpdateBackend.fedoraDnf5,
      );
      expect(
        PackageUpdateBackend.fromWire('bazzite-rpm-ostree'),
        PackageUpdateBackend.bazziteRpmOstree,
      );
      expect(
        PackageUpdateBackend.fromWire('arch-pacman'),
        PackageUpdateBackend.archPacman,
      );
      expect(
        PackageUpdateBackend.fromWire('steamos-pacman'),
        PackageUpdateBackend.steamOsPacman,
      );
      expect(
        PackageUpdateBackend.fromWire('unsupported'),
        PackageUpdateBackend.unsupported,
      );
    });

    test('parses every state enum string', () {
      expect(
        PackageUpdateRunState.fromWire('idle'),
        PackageUpdateRunState.idle,
      );
      expect(
        PackageUpdateRunState.fromWire('checking'),
        PackageUpdateRunState.checking,
      );
      expect(
        PackageUpdateRunState.fromWire('available'),
        PackageUpdateRunState.available,
      );
      expect(
        PackageUpdateRunState.fromWire('upToDate'),
        PackageUpdateRunState.upToDate,
      );
      expect(
        PackageUpdateRunState.fromWire('running'),
        PackageUpdateRunState.running,
      );
      expect(
        PackageUpdateRunState.fromWire('succeeded'),
        PackageUpdateRunState.succeeded,
      );
      expect(
        PackageUpdateRunState.fromWire('failed'),
        PackageUpdateRunState.failed,
      );
      expect(
        PackageUpdateRunState.fromWire('unsupported'),
        PackageUpdateRunState.unsupported,
      );
    });

    test('parses every error code string', () {
      expect(
        PackageUpdateErrorCode.fromWire('unsupported_os'),
        PackageUpdateErrorCode.unsupportedOs,
      );
      expect(
        PackageUpdateErrorCode.fromWire('missing_repository'),
        PackageUpdateErrorCode.missingRepository,
      );
      expect(
        PackageUpdateErrorCode.fromWire('package_manager_unavailable'),
        PackageUpdateErrorCode.packageManagerUnavailable,
      );
      expect(
        PackageUpdateErrorCode.fromWire('package_database_locked'),
        PackageUpdateErrorCode.packageDatabaseLocked,
      );
      expect(
        PackageUpdateErrorCode.fromWire('signature_error'),
        PackageUpdateErrorCode.signatureError,
      );
      expect(
        PackageUpdateErrorCode.fromWire('network_error'),
        PackageUpdateErrorCode.networkError,
      );
      expect(
        PackageUpdateErrorCode.fromWire('authorization_denied'),
        PackageUpdateErrorCode.authorizationDenied,
      );
      expect(
        PackageUpdateErrorCode.fromWire('update_failed'),
        PackageUpdateErrorCode.updateFailed,
      );
      expect(
        PackageUpdateErrorCode.fromWire('reboot_required'),
        PackageUpdateErrorCode.rebootRequired,
      );
    });

    test('parses nullable version and reboot fields', () {
      final status = PackageUpdateStatus.fromJsonString('''
{
  "schemaVersion": 1,
  "backend": "unsupported",
  "state": "unsupported",
  "installedVersion": null,
  "availableVersion": null,
  "rebootRequired": null,
  "summary": "Unsupported OS",
  "errorCode": "unsupported_os",
  "logTail": []
}
''');

      expect(status.installedVersion, isNull);
      expect(status.availableVersion, isNull);
      expect(status.rebootRequired, isNull);
      expect(status.errorCode, PackageUpdateErrorCode.unsupportedOs);
    });
  });

  group('DbusPackageUpdateService', () {
    test('maps D-Bus method names and reply signatures', () async {
      final calls = <String>[];
      final service = DbusPackageUpdateService.test(
        methodCaller: (method, args, replySignature) async {
          calls.add('$method:${replySignature.value}:${args.length}');
          switch (method) {
            case 'GetPackageBackend':
              return [const DBusString('fedora-dnf5')];
            case 'CheckForUpdates':
              return [DBusString(_statusJson(state: 'available'))];
            case 'StartUpdate':
              return [const DBusString('update-1')];
            case 'GetUpdateStatus':
              expect((args.single as DBusString).value, 'update-1');
              return [DBusString(_statusJson(state: 'running'))];
            case 'CancelUpdate':
              expect((args.single as DBusString).value, 'update-1');
              return [const DBusBoolean(true)];
            default:
              throw StateError('unexpected method $method');
          }
        },
      );

      expect(
        await service.getPackageBackend(),
        PackageUpdateBackend.fedoraDnf5,
      );
      expect(
        (await service.checkForUpdates()).state,
        PackageUpdateRunState.available,
      );
      expect(await service.startUpdate(), 'update-1');
      expect(
        (await service.getUpdateStatus('update-1')).state,
        PackageUpdateRunState.running,
      );
      expect(await service.cancelUpdate('update-1'), isTrue);

      expect(calls, [
        'GetPackageBackend:s:0',
        'CheckForUpdates:s:0',
        'StartUpdate:s:0',
        'GetUpdateStatus:s:1',
        'CancelUpdate:b:1',
      ]);
    });

    test('daemon unavailable maps to typed service exception', () async {
      final service = DbusPackageUpdateService.test(
        methodCaller: (_, __, ___) async => throw Exception('daemon down'),
      );

      expect(
        service.checkForUpdates,
        throwsA(isA<PackageUpdateServiceException>()),
      );
    });
  });
}

String _statusJson({required String state}) =>
    '''
{
  "schemaVersion": 1,
  "backend": "fedora-dnf5",
  "state": "$state",
  "installedVersion": "0.0.1",
  "availableVersion": "0.0.2",
  "rebootRequired": false,
  "summary": "Update available",
  "errorCode": null,
  "logTail": []
}
''';
