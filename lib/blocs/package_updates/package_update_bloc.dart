import 'package:flutter_bloc/flutter_bloc.dart';

import '../../services/bridge_service.dart';
import '../../services/package_update_service.dart';
import 'package_update_event.dart';
import 'package_update_state.dart';

class PackageUpdateBloc extends Bloc<PackageUpdateEvent, PackageUpdateState> {
  PackageUpdateBloc({PackageUpdateService? service, String? initialUpdateId})
    : _service = service ?? BridgeService().packageUpdates,
      super(PackageUpdateState(updateId: initialUpdateId)) {
    on<LoadPackageUpdateStatus>(_onLoad);
    on<CheckPackageUpdatesRequested>(_onCheck);
    on<StartPackageUpdateRequested>(_onStart);
    on<PollPackageUpdateStatusRequested>(_onPoll);
    on<CancelPackageUpdateRequested>(_onCancel);
  }

  PackageUpdateBloc.seeded(PackageUpdateState initialState)
    : _service = BridgeService().packageUpdates,
      super(initialState) {
    on<LoadPackageUpdateStatus>(_onLoad);
    on<CheckPackageUpdatesRequested>(_onCheck);
    on<StartPackageUpdateRequested>(_onStart);
    on<PollPackageUpdateStatusRequested>(_onPoll);
    on<CancelPackageUpdateRequested>(_onCancel);
  }

  final PackageUpdateService _service;

  Future<void> _onLoad(
    LoadPackageUpdateStatus event,
    Emitter<PackageUpdateState> emit,
  ) async {
    emit(state.copyWith(isChecking: true, daemonUnavailable: false));
    try {
      final backend = await _service.getPackageBackend();
      final status = await _service.checkForUpdates();
      emit(PackageUpdateState(status: status, backend: backend));
    } catch (e) {
      emit(_unavailableState(e));
    }
  }

  Future<void> _onCheck(
    CheckPackageUpdatesRequested event,
    Emitter<PackageUpdateState> emit,
  ) async {
    emit(state.copyWith(isChecking: true, daemonUnavailable: false));
    try {
      final status = await _service.checkForUpdates();
      emit(state.copyWith(status: status, isChecking: false));
    } catch (e) {
      emit(_unavailableState(e));
    }
  }

  Future<void> _onStart(
    StartPackageUpdateRequested event,
    Emitter<PackageUpdateState> emit,
  ) async {
    try {
      final updateId = await _service.startUpdate();
      final status = await _service.getUpdateStatus(updateId);
      emit(PackageUpdateState(status: status, updateId: updateId));
    } catch (e) {
      emit(state.copyWith(status: _failedStatusForException(e)));
    }
  }

  Future<void> _onPoll(
    PollPackageUpdateStatusRequested event,
    Emitter<PackageUpdateState> emit,
  ) async {
    final updateId = state.updateId;
    if (updateId == null) {
      return;
    }
    try {
      final status = await _service.getUpdateStatus(updateId);
      emit(state.copyWith(status: status));
    } catch (e) {
      emit(_unavailableState(e));
    }
  }

  Future<void> _onCancel(
    CancelPackageUpdateRequested event,
    Emitter<PackageUpdateState> emit,
  ) async {
    final updateId = state.updateId;
    if (updateId == null) {
      return;
    }
    try {
      await _service.cancelUpdate(updateId);
      final status = await _service.getUpdateStatus(updateId);
      emit(state.copyWith(status: status, clearUpdateId: true));
    } catch (e) {
      emit(state.copyWith(status: _failedStatusForException(e)));
    }
  }

  PackageUpdateState _unavailableState(Object error) =>
      PackageUpdateState(daemonUnavailable: true, lastError: error.toString());

  PackageUpdateStatus _failedStatusForException(Object error) {
    final message = error.toString();
    return PackageUpdateStatus(
      schemaVersion: 1,
      backend: state.backend ?? PackageUpdateBackend.unsupported,
      state: PackageUpdateRunState.failed,
      installedVersion: null,
      availableVersion: null,
      rebootRequired: null,
      summary: 'Package update failed',
      errorCode: message.contains('authorization_denied')
          ? PackageUpdateErrorCode.authorizationDenied
          : PackageUpdateErrorCode.updateFailed,
      logTail: [message],
    );
  }
}
