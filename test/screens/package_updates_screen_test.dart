import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import 'package:flutter_application_1/blocs/package_updates/package_update_bloc.dart';
import 'package:flutter_application_1/blocs/package_updates/package_update_state.dart';
import 'package:flutter_application_1/screens/package_updates_screen.dart';
import 'package:flutter_application_1/services/package_update_service.dart';

Widget _buildScreen(
  PackageUpdateStatus status, {
  bool daemonUnavailable = false,
}) {
  final bloc = PackageUpdateBloc.seeded(
    PackageUpdateState(status: status, daemonUnavailable: daemonUnavailable),
  );
  return MaterialApp(
    home: PackageUpdatesScreen(key: UniqueKey(), testBloc: bloc),
  );
}

void main() {
  testWidgets('shows backend and versions for available update', (
    tester,
  ) async {
    await tester.pumpWidget(_buildScreen(status()));

    expect(find.text('Package Updates'), findsOneWidget);
    expect(find.text('fedora-dnf5'), findsOneWidget);
    expect(find.text('1.0.0'), findsOneWidget);
    expect(find.text('1.0.1'), findsOneWidget);
    expect(find.text('Install Update'), findsOneWidget);
  });

  testWidgets('shows up to date with disabled update button', (tester) async {
    await tester.pumpWidget(
      _buildScreen(status(state: PackageUpdateRunState.upToDate)),
    );

    expect(find.text('ASUS Linux Control is up to date'), findsOneWidget);
    final button = tester.widget<ElevatedButton>(
      find.widgetWithText(ElevatedButton, 'Install Update'),
    );
    expect(button.onPressed, isNull);
  });

  testWidgets('shows unsupported OS with disabled update button', (
    tester,
  ) async {
    await tester.pumpWidget(
      _buildScreen(
        status(
          backend: PackageUpdateBackend.unsupported,
          state: PackageUpdateRunState.unsupported,
          errorCode: PackageUpdateErrorCode.unsupportedOs,
        ),
      ),
    );

    expect(
      find.text('Package updates are not supported on this system'),
      findsOneWidget,
    );
    final button = tester.widget<ElevatedButton>(
      find.widgetWithText(ElevatedButton, 'Install Update'),
    );
    expect(button.onPressed, isNull);
  });

  testWidgets('shows specific package update errors', (tester) async {
    for (final entry in {
      PackageUpdateErrorCode.missingRepository:
          'Package repository is not configured',
      PackageUpdateErrorCode.signatureError:
          'Package signature verification failed',
      PackageUpdateErrorCode.packageDatabaseLocked:
          'Package database is locked',
    }.entries) {
      await tester.pumpWidget(
        _buildScreen(
          status(state: PackageUpdateRunState.failed, errorCode: entry.key),
        ),
      );

      expect(find.text(entry.value), findsOneWidget);
    }
  });

  testWidgets('shows running progress and cancel button', (tester) async {
    await tester.pumpWidget(
      _buildScreen(status(state: PackageUpdateRunState.running)),
    );

    expect(find.text('Package update is running'), findsOneWidget);
    expect(find.text('Cancel'), findsOneWidget);
    final button = tester.widget<ElevatedButton>(
      find.widgetWithText(ElevatedButton, 'Install Update'),
    );
    expect(button.onPressed, isNull);
  });

  testWidgets('shows reboot required state', (tester) async {
    await tester.pumpWidget(
      _buildScreen(
        status(state: PackageUpdateRunState.succeeded, rebootRequired: true),
      ),
    );

    expect(find.text('Restart required to finish updating'), findsOneWidget);
  });

  testWidgets('shows daemon unavailable with disabled update button', (
    tester,
  ) async {
    await tester.pumpWidget(_buildScreen(status(), daemonUnavailable: true));

    expect(find.text('Package update daemon is unavailable'), findsOneWidget);
    final button = tester.widget<ElevatedButton>(
      find.widgetWithText(ElevatedButton, 'Install Update'),
    );
    expect(button.onPressed, isNull);
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
