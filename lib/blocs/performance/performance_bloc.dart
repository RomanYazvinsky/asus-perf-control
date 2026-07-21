import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:bloc/bloc.dart';

import '../../services/bridge_service.dart';
import '../../services/backend_logger.dart';
import '../../services/performance_service.dart';
import '../../services/settings/app_settings_store.dart';
import 'performance_event.dart';
import 'performance_state.dart';

class PerformanceBloc extends Bloc<PerformanceEvent, PerformanceState> {
  final IPerformanceService _service;
  final AppSettingsStore? _settingsStore;

  PerformanceBloc({
    IPerformanceService? service,
    AppSettingsStore? settingsStore,
  }) : _service = service ?? BridgeService(),
       _settingsStore = settingsStore,
       super(PerformanceInitial()) {
    on<LoadCurrentModeEvent>(_onLoadCurrentMode);
    on<SetModeRequestedEvent>(_onSetModeRequested);
  }

  Future<void> _onLoadCurrentMode(
    LoadCurrentModeEvent event,
    Emitter<PerformanceState> emit,
  ) async {
    emit(PerformanceLoading());

    try {
      var currentMode = await _service.getPerformanceMode();
      final settings = await _settingsStore?.load();
      final desiredMode = settings?.hardware.performanceMode;
      if (desiredMode == null) {
        await _settingsStore?.update((settings) {
          return settings.copyWith(
            hardware: settings.hardware.copyWith(performanceMode: currentMode),
          );
        });
      } else if (desiredMode != currentMode) {
        _logHardwareMismatch(
          'performanceMode',
          expected: desiredMode,
          actual: currentMode,
        );
        await _service.setPerformanceMode(desiredMode);
        currentMode = desiredMode;
      }
      emit(PerformanceLoaded(currentMode));
    } catch (e) {
      emit(PerformanceError(e.toString()));
    }
  }

  Future<void> _onSetModeRequested(
    SetModeRequestedEvent event,
    Emitter<PerformanceState> emit,
  ) async {
    emit(PerformanceLoading());

    try {
      await _service.setPerformanceMode(event.mode);
      await _settingsStore?.update((settings) {
        return settings.copyWith(
          hardware: settings.hardware.copyWith(performanceMode: event.mode),
        );
      });
      emit(PerformanceLoaded(event.mode));
    } catch (e) {
      emit(PerformanceError(e.toString()));
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
}
