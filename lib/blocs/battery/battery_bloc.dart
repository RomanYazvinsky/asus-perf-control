// Battery BLoC
// Manages battery charge limit state

import 'package:flutter_bloc/flutter_bloc.dart';

import '../../services/backend_logger.dart';
import '../../services/battery_service.dart';
import '../../services/bridge_service.dart';
import '../../services/settings/app_settings_store.dart';
import 'battery_event.dart';
import 'battery_state.dart';

class BatteryBloc extends Bloc<BatteryEvent, BatteryState> {
  final IBatteryService _service;
  final AppSettingsStore? _settingsStore;

  BatteryBloc({IBatteryService? service, AppSettingsStore? settingsStore})
    : _service = service ?? BridgeService(),
      _settingsStore = settingsStore,
      super(const BatteryInitial()) {
    on<LoadCurrentLimitEvent>(_onLoadCurrentLimit);
    on<SetLimitRequestedEvent>(_onSetLimitRequested);
    on<CheckBatterySupportEvent>(_onCheckBatterySupport);
  }

  Future<void> _onLoadCurrentLimit(
    LoadCurrentLimitEvent event,
    Emitter<BatteryState> emit,
  ) async {
    emit(const BatteryLoading());

    try {
      final status = await _restoreSavedLimitIfNeeded(
        await _service.getBatteryStatus(),
      );
      emit(
        BatteryLoaded(
          chargeLimit: status.chargeLimit,
          percentage: status.percentage,
          state: status.state,
        ),
      );
    } catch (e) {
      emit(BatteryError(e.toString()));
    }
  }

  Future<void> _onSetLimitRequested(
    SetLimitRequestedEvent event,
    Emitter<BatteryState> emit,
  ) async {
    emit(const BatteryLoading());

    try {
      await _service.setChargeLimit(event.limit);
      await _settingsStore?.update((settings) {
        return settings.copyWith(
          hardware: settings.hardware.copyWith(batteryChargeLimit: event.limit),
        );
      });
      emit(BatteryLoaded(chargeLimit: event.limit));
    } catch (e) {
      emit(BatteryError(e.toString()));
    }
  }

  Future<void> _onCheckBatterySupport(
    CheckBatterySupportEvent event,
    Emitter<BatteryState> emit,
  ) async {
    emit(const BatteryLoading());

    try {
      final isSupported = await _service.isChargeLimitSupported();
      if (isSupported) {
        // Directly load current limit instead of adding event to queue
        final status = await _restoreSavedLimitIfNeeded(
          await _service.getBatteryStatus(),
        );
        emit(
          BatteryLoaded(
            chargeLimit: status.chargeLimit,
            percentage: status.percentage,
            state: status.state,
          ),
        );
      } else {
        emit(const BatteryUnsupported());
      }
    } catch (e) {
      emit(BatteryError(e.toString()));
    }
  }

  Future<BatteryStatus> _restoreSavedLimitIfNeeded(BatteryStatus status) async {
    final settings = await _settingsStore?.load();
    final desiredLimit = settings?.hardware.batteryChargeLimit;
    if (desiredLimit == null) {
      await _settingsStore?.update((settings) {
        return settings.copyWith(
          hardware: settings.hardware.copyWith(
            batteryChargeLimit: status.chargeLimit,
          ),
        );
      });
      return status;
    }
    if (desiredLimit == status.chargeLimit) {
      return status;
    }
    _logHardwareMismatch(
      'batteryChargeLimit',
      expected: desiredLimit,
      actual: status.chargeLimit,
    );
    await _service.setChargeLimit(desiredLimit);
    return BatteryStatus(
      chargeLimit: desiredLimit,
      percentage: status.percentage,
      state: status.state,
    );
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
