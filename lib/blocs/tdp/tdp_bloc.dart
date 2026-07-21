import 'package:flutter_bloc/flutter_bloc.dart';

import '../../services/backend_logger.dart';
import '../../services/bridge_service.dart';
import '../../services/dbus_client.dart';
import '../../services/settings/app_settings.dart';
import '../../services/settings/app_settings_store.dart';
import '../../services/tdp_service.dart';
import 'tdp_event.dart';
import 'tdp_state.dart';

class TdpBloc extends Bloc<TdpEvent, TdpState> {
  final ITdpService _service;
  final AppSettingsStore? _settingsStore;

  TdpBloc({ITdpService? service, AppSettingsStore? settingsStore})
    : _service = service ?? BridgeService(),
      _settingsStore = settingsStore,
      super(TdpInitial()) {
    on<LoadTdpConfig>(_onLoadTdpConfig);
    on<SelectTdpProfile>(_onSelectTdpProfile);
    on<SetTdpRequested>(_onSetTdpRequested);
    on<ResetTdpToDefaults>(_onResetTdpToDefaults);
    on<ClearPowerLimitsRequested>(_onClearPowerLimitsRequested);
  }

  Future<void> _onLoadTdpConfig(
    LoadTdpConfig event,
    Emitter<TdpState> emit,
  ) async {
    emit(TdpLoading());
    try {
      final TdpConfigDbus config = await _restoreSavedTdpIfNeeded(
        await _service.getTdpConfig(),
      );
      if (!config.isSupported) {
        emit(TdpUnsupported());
        return;
      }
      emit(
        TdpLoaded(
          sustainedW: config.sustainedW,
          boostW: config.boostW,
          tempLimitC: config.tempLimitC,
          activePerformanceMode: config.activePerformanceMode,
          minW: config.minSustainedW,
          maxW: config.maxSustainedW,
          minBoostW: config.minBoostW,
          maxBoostW: config.maxBoostW,
          powerLimitsEnabled: config.powerLimitsEnabled,
        ),
      );
    } catch (e) {
      emit(TdpError(e.toString()));
    }
  }

  Future<void> _onSelectTdpProfile(
    SelectTdpProfile event,
    Emitter<TdpState> emit,
  ) async {
    emit(TdpLoading());
    try {
      await _service.setPerformanceMode(event.mode);
      await _settingsStore?.update((settings) {
        return settings.copyWith(
          hardware: settings.hardware.copyWith(performanceMode: event.mode),
        );
      });
      final TdpConfigDbus config = await _service.getTdpConfig();
      if (!config.isSupported) {
        emit(TdpUnsupported());
        return;
      }
      emit(
        TdpLoaded(
          sustainedW: config.sustainedW,
          boostW: config.boostW,
          tempLimitC: config.tempLimitC,
          activePerformanceMode: config.activePerformanceMode,
          minW: config.minSustainedW,
          maxW: config.maxSustainedW,
          minBoostW: config.minBoostW,
          maxBoostW: config.maxBoostW,
          powerLimitsEnabled: config.powerLimitsEnabled,
        ),
      );
    } catch (e) {
      emit(TdpError(e.toString()));
    }
  }

  Future<void> _onSetTdpRequested(
    SetTdpRequested event,
    Emitter<TdpState> emit,
  ) async {
    final prevState = state;
    emit(TdpLoading());
    try {
      if (event.applyPowerLimits) {
        await _service.setPowerLimits(
          sustainedW: event.sustainedW,
          boostW: event.boostW,
        );
      }
      if (event.applyTempLimit) {
        await _service.setCpuTempLimit(event.tempLimitC);
      }
      await _settingsStore?.update((settings) {
        return settings.copyWith(
          hardware: settings.hardware.copyWith(
            tdp: TdpDesiredSettings(
              powerLimitsEnabled: event.applyPowerLimits,
              sustainedW: event.sustainedW,
              boostW: event.boostW,
              tempLimitEnabled: event.applyTempLimit,
              tempLimitC: event.applyTempLimit ? event.tempLimitC : 0,
            ),
          ),
        );
      });

      final previousLoaded = prevState is TdpLoaded ? prevState : null;
      emit(
        TdpLoaded(
          sustainedW: event.applyPowerLimits
              ? event.sustainedW
              : previousLoaded?.sustainedW ?? event.sustainedW,
          boostW: event.applyPowerLimits
              ? event.boostW
              : previousLoaded?.boostW ?? event.boostW,
          tempLimitC: event.applyTempLimit
              ? event.tempLimitC
              : previousLoaded?.tempLimitC ?? event.tempLimitC,
          activePerformanceMode: previousLoaded?.activePerformanceMode ?? 1,
          minW: previousLoaded?.minW ?? 5,
          maxW: previousLoaded?.maxW ?? 54,
          minBoostW: previousLoaded?.minBoostW ?? 5,
          maxBoostW: previousLoaded?.maxBoostW ?? 54,
          powerLimitsEnabled: event.applyPowerLimits,
          tempLimitEnabled: event.applyTempLimit,
        ),
      );
    } catch (e) {
      emit(TdpError(e.toString()));
    }
  }

  Future<void> _onResetTdpToDefaults(
    ResetTdpToDefaults event,
    Emitter<TdpState> emit,
  ) async {
    emit(TdpLoading());
    try {
      // TDP values are volatile (reset on reboot). There's no API to "reset to firmware defaults"
      // without changing performance mode. This action simply reloads current values from hardware,
      // which is useful if user wants to discard UI changes before applying them.
      // For true reset, user should reboot or change performance mode.
      final TdpConfigDbus config = await _service.getTdpConfig();
      emit(
        TdpLoaded(
          sustainedW: config.sustainedW,
          boostW: config.boostW,
          tempLimitC: config.tempLimitC,
          activePerformanceMode: config.activePerformanceMode,
          minW: config.minSustainedW,
          maxW: config.maxSustainedW,
          minBoostW: config.minBoostW,
          maxBoostW: config.maxBoostW,
          powerLimitsEnabled: config.powerLimitsEnabled,
        ),
      );
    } catch (e) {
      emit(TdpError(e.toString()));
    }
  }

  Future<void> _onClearPowerLimitsRequested(
    ClearPowerLimitsRequested event,
    Emitter<TdpState> emit,
  ) async {
    final prevState = state;
    emit(TdpLoading());
    try {
      await _service.clearPowerLimits();

      final previousLoaded = prevState is TdpLoaded ? prevState : null;
      await _settingsStore?.update((settings) {
        final previousTdp = settings.hardware.tdp;
        return settings.copyWith(
          hardware: settings.hardware.copyWith(
            tdp: TdpDesiredSettings(
              powerLimitsEnabled: false,
              sustainedW:
                  previousLoaded?.sustainedW ?? previousTdp?.sustainedW ?? 0,
              boostW: previousLoaded?.boostW ?? previousTdp?.boostW ?? 0,
              tempLimitEnabled:
                  previousLoaded?.tempLimitEnabled ??
                  previousTdp?.tempLimitEnabled ??
                  false,
              tempLimitC:
                  previousLoaded?.tempLimitC ?? previousTdp?.tempLimitC ?? 0,
            ),
          ),
        );
      });
      emit(
        TdpLoaded(
          sustainedW: previousLoaded?.sustainedW ?? 0,
          boostW: previousLoaded?.boostW ?? 0,
          tempLimitC: previousLoaded?.tempLimitC ?? 0,
          activePerformanceMode: previousLoaded?.activePerformanceMode ?? 1,
          minW: previousLoaded?.minW ?? 5,
          maxW: previousLoaded?.maxW ?? 54,
          minBoostW: previousLoaded?.minBoostW ?? 5,
          maxBoostW: previousLoaded?.maxBoostW ?? 54,
          powerLimitsEnabled: false,
          tempLimitEnabled: previousLoaded?.tempLimitEnabled,
        ),
      );
    } catch (e) {
      emit(TdpError(e.toString()));
    }
  }

  Future<TdpConfigDbus> _restoreSavedTdpIfNeeded(TdpConfigDbus config) async {
    final settings = await _settingsStore?.load();
    final desired = settings?.hardware.tdp;
    if (!config.isSupported) return config;
    if (desired == null) {
      await _settingsStore?.update((settings) {
        return settings.copyWith(
          hardware: settings.hardware.copyWith(
            tdp: TdpDesiredSettings(
              powerLimitsEnabled: config.powerLimitsEnabled,
              sustainedW: config.sustainedW,
              boostW: config.boostW,
              tempLimitEnabled: config.tempLimitC > 0,
              tempLimitC: config.tempLimitC,
            ),
          ),
        );
      });
      return config;
    }

    var changed = false;
    final powerMismatch =
        desired.powerLimitsEnabled != config.powerLimitsEnabled ||
        (desired.powerLimitsEnabled &&
            (desired.sustainedW != config.sustainedW ||
                desired.boostW != config.boostW));
    if (powerMismatch) {
      _logHardwareMismatch(
        'tdp.powerLimits',
        expected:
            '${desired.powerLimitsEnabled}:${desired.sustainedW}/${desired.boostW}',
        actual:
            '${config.powerLimitsEnabled}:${config.sustainedW}/${config.boostW}',
      );
      if (desired.powerLimitsEnabled) {
        await _service.setPowerLimits(
          sustainedW: desired.sustainedW,
          boostW: desired.boostW,
        );
      } else {
        await _service.clearPowerLimits();
      }
      changed = true;
    }

    final desiredTemp = desired.tempLimitEnabled ? desired.tempLimitC : 0;
    if (desiredTemp != config.tempLimitC) {
      _logHardwareMismatch(
        'tdp.tempLimitC',
        expected: desiredTemp,
        actual: config.tempLimitC,
      );
      await _service.setCpuTempLimit(desiredTemp);
      changed = true;
    }

    return changed ? await _service.getTdpConfig() : config;
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
