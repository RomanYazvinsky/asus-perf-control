import 'package:flutter_bloc/flutter_bloc.dart';

import '../../services/bridge_service.dart';
import '../../services/dbus_client.dart';
import '../../services/tdp_service.dart';
import 'tdp_event.dart';
import 'tdp_state.dart';

class TdpBloc extends Bloc<TdpEvent, TdpState> {
  final ITdpService _service;

  TdpBloc({ITdpService? service})
    : _service = service ?? BridgeService(),
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

  Future<void> _onSelectTdpProfile(
    SelectTdpProfile event,
    Emitter<TdpState> emit,
  ) async {
    emit(TdpLoading());
    try {
      await _service.setPerformanceMode(event.mode);
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
}
