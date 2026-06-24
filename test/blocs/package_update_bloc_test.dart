import 'package:flutter_test/flutter_test.dart';
import 'package:mocktail/mocktail.dart';

import 'package:flutter_application_1/blocs/package_updates/package_update_bloc.dart';
import 'package:flutter_application_1/blocs/package_updates/package_update_event.dart';
import 'package:flutter_application_1/blocs/package_updates/package_update_state.dart';
import 'package:flutter_application_1/services/package_update_service.dart';

class _MockPackageUpdateService extends Mock implements PackageUpdateService {}

void main() {
  group('PackageUpdateBloc', () {
    late _MockPackageUpdateService service;

    setUp(() {
      service = _MockPackageUpdateService();
    });

    test('initial load requests backend and checks for updates', () async {
      when(
        () => service.getPackageBackend(),
      ).thenAnswer((_) async => PackageUpdateBackend.fedoraDnf5);
      when(
        () => service.checkForUpdates(),
      ).thenAnswer((_) async => status(state: PackageUpdateRunState.upToDate));

      final bloc = PackageUpdateBloc(service: service)
        ..add(LoadPackageUpdateStatus());

      await expectLater(
        bloc.stream,
        emitsInOrder([
          isA<PackageUpdateState>().having(
            (s) => s.isChecking,
            'checking',
            isTrue,
          ),
          isA<PackageUpdateState>()
              .having(
                (s) => s.status?.state,
                'state',
                PackageUpdateRunState.upToDate,
              )
              .having((s) => s.canStartUpdate, 'canStartUpdate', isFalse),
        ]),
      );
      verify(() => service.getPackageBackend()).called(1);
      verify(() => service.checkForUpdates()).called(1);
      await bloc.close();
    });

    test('unsupported backend emits disabled update state', () async {
      when(
        () => service.getPackageBackend(),
      ).thenAnswer((_) async => PackageUpdateBackend.unsupported);
      when(() => service.checkForUpdates()).thenAnswer(
        (_) async => status(
          backend: PackageUpdateBackend.unsupported,
          state: PackageUpdateRunState.unsupported,
          errorCode: PackageUpdateErrorCode.unsupportedOs,
        ),
      );

      final bloc = PackageUpdateBloc(service: service)
        ..add(LoadPackageUpdateStatus());

      await expectLater(
        bloc.stream,
        emitsThrough(
          isA<PackageUpdateState>()
              .having(
                (s) => s.status?.state,
                'state',
                PackageUpdateRunState.unsupported,
              )
              .having((s) => s.canStartUpdate, 'canStartUpdate', isFalse),
        ),
      );
      await bloc.close();
    });

    test('available update enables update button', () async {
      when(
        () => service.getPackageBackend(),
      ).thenAnswer((_) async => PackageUpdateBackend.fedoraDnf5);
      when(
        () => service.checkForUpdates(),
      ).thenAnswer((_) async => status(state: PackageUpdateRunState.available));

      final bloc = PackageUpdateBloc(service: service)
        ..add(LoadPackageUpdateStatus());

      await expectLater(
        bloc.stream,
        emitsThrough(
          isA<PackageUpdateState>().having(
            (s) => s.canStartUpdate,
            'canStartUpdate',
            isTrue,
          ),
        ),
      );
      await bloc.close();
    });

    test('running update disables update button and polls status', () async {
      when(() => service.startUpdate()).thenAnswer((_) async => 'update-1');
      when(
        () => service.getUpdateStatus('update-1'),
      ).thenAnswer((_) async => status(state: PackageUpdateRunState.running));

      final bloc = PackageUpdateBloc(service: service)
        ..add(StartPackageUpdateRequested());

      await expectLater(
        bloc.stream,
        emitsThrough(
          isA<PackageUpdateState>()
              .having((s) => s.isUpdateRunning, 'running', isTrue)
              .having((s) => s.canStartUpdate, 'canStartUpdate', isFalse),
        ),
      );
      verify(() => service.getUpdateStatus('update-1')).called(1);
      await bloc.close();
    });

    test(
      'succeeded with reboot required shows reboot-required state',
      () async {
        when(() => service.getUpdateStatus('update-1')).thenAnswer(
          (_) async => status(
            state: PackageUpdateRunState.succeeded,
            rebootRequired: true,
          ),
        );

        final bloc = PackageUpdateBloc(
          service: service,
          initialUpdateId: 'update-1',
        )..add(PollPackageUpdateStatusRequested());

        await expectLater(
          bloc.stream,
          emitsThrough(
            isA<PackageUpdateState>()
                .having((s) => s.status?.rebootRequired, 'rebootRequired', true)
                .having((s) => s.summary, 'summary', contains('Restart')),
          ),
        );
        await bloc.close();
      },
    );

    test('daemon unavailable emits actionable unavailable state', () async {
      when(
        () => service.getPackageBackend(),
      ).thenThrow(const PackageUpdateServiceException('daemon unavailable'));

      final bloc = PackageUpdateBloc(service: service)
        ..add(LoadPackageUpdateStatus());

      await expectLater(
        bloc.stream,
        emitsThrough(
          isA<PackageUpdateState>()
              .having((s) => s.daemonUnavailable, 'daemonUnavailable', isTrue)
              .having((s) => s.canStartUpdate, 'canStartUpdate', isFalse),
        ),
      );
      await bloc.close();
    });

    test('authorization denied maps to failed state', () async {
      when(
        () => service.startUpdate(),
      ).thenThrow(const PackageUpdateServiceException('authorization_denied'));

      final bloc = PackageUpdateBloc(service: service)
        ..add(StartPackageUpdateRequested());

      await expectLater(
        bloc.stream,
        emitsThrough(
          isA<PackageUpdateState>()
              .having(
                (s) => s.status?.state,
                'state',
                PackageUpdateRunState.failed,
              )
              .having(
                (s) => s.status?.errorCode,
                'errorCode',
                PackageUpdateErrorCode.authorizationDenied,
              ),
        ),
      );
      await bloc.close();
    });
  });
}

PackageUpdateStatus status({
  PackageUpdateBackend backend = PackageUpdateBackend.fedoraDnf5,
  PackageUpdateRunState state = PackageUpdateRunState.available,
  String? installedVersion = '1.0.0',
  String? availableVersion = '1.0.1',
  bool? rebootRequired = false,
  PackageUpdateErrorCode? errorCode,
}) => PackageUpdateStatus(
  schemaVersion: 1,
  backend: backend,
  state: state,
  installedVersion: installedVersion,
  availableVersion: availableVersion,
  rebootRequired: rebootRequired,
  summary: 'Update status',
  errorCode: errorCode,
  logTail: const [],
);
