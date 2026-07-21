import 'package:flutter_bloc/flutter_bloc.dart';

import '../../services/backend_logger.dart';
import '../../services/bridge_service.dart';
import '../../services/cpu_boost_service.dart';
import '../../services/settings/app_settings_store.dart';
import 'cpu_boost_event.dart';
import 'cpu_boost_state.dart';

class CpuBoostBloc extends Bloc<CpuBoostEvent, CpuBoostState> {
  final ICpuBoostService _service;
  final AppSettingsStore? _settingsStore;

  CpuBoostBloc({ICpuBoostService? service, AppSettingsStore? settingsStore})
    : _service = service ?? BridgeService(),
      _settingsStore = settingsStore,
      super(CpuBoostInitial()) {
    on<LoadCpuBoostStatus>(_onLoadCpuBoostStatus);
    on<SetCpuBoostModeRequested>(_onSetCpuBoostModeRequested);
  }

  Future<void> _onLoadCpuBoostStatus(
    LoadCpuBoostStatus event,
    Emitter<CpuBoostState> emit,
  ) async {
    emit(CpuBoostLoading());
    try {
      var status = await _service.getCpuBoostStatus();
      if (!status.isSupported) {
        emit(CpuBoostUnsupported());
        return;
      }
      final settings = await _settingsStore?.load();
      final desiredId = settings?.hardware.cpuBoostMode;
      final desiredMode = _cpuBoostModeFromSavedId(desiredId);
      if (desiredId == null) {
        await _settingsStore?.update((settings) {
          return settings.copyWith(
            hardware: settings.hardware.copyWith(
              cpuBoostMode: status.currentMode.daemonId,
            ),
          );
        });
      } else if (desiredMode != null && desiredMode != status.currentMode) {
        _logHardwareMismatch(
          'cpuBoostMode',
          expected: desiredMode.daemonId,
          actual: status.currentMode.daemonId,
        );
        if (status.availableModes.contains(desiredMode)) {
          await _service.setCpuBoostMode(desiredMode);
          status = await _service.getCpuBoostStatus();
        }
      }
      emit(CpuBoostLoaded(status));
    } catch (e) {
      emit(CpuBoostError(e.toString()));
    }
  }

  Future<void> _onSetCpuBoostModeRequested(
    SetCpuBoostModeRequested event,
    Emitter<CpuBoostState> emit,
  ) async {
    emit(CpuBoostLoading());
    try {
      await _service.setCpuBoostMode(event.mode);
      await _settingsStore?.update((settings) {
        return settings.copyWith(
          hardware: settings.hardware.copyWith(
            cpuBoostMode: event.mode.daemonId,
          ),
        );
      });
      final status = await _service.getCpuBoostStatus();
      if (!status.isSupported) {
        emit(CpuBoostUnsupported());
        return;
      }
      emit(CpuBoostLoaded(status));
    } catch (e) {
      emit(CpuBoostError(e.toString()));
    }
  }

  void _logHardwareMismatch(
    String key, {
    required Object expected,
    required Object actual,
  }) {
    BackendLogger.logEvent(
      'Settings.hardwareMismatch',
      fields: {'key': key, 'expected': expected, 'actual': actual},
    );
  }

  CpuBoostMode? _cpuBoostModeFromSavedId(String? id) {
    if (id == null) return null;
    for (final mode in CpuBoostMode.values) {
      if (mode.daemonId == id) return mode;
    }
    BackendLogger.logEvent(
      'Settings.readMismatch',
      fields: {'key': 'hardware.cpuBoostMode', 'actual': id},
    );
    return null;
  }
}
