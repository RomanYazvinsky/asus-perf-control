// Battery BLoC
// Manages battery charge limit state

import 'package:flutter_bloc/flutter_bloc.dart';

import '../../services/battery_service.dart';
import '../../services/bridge_service.dart';
import 'battery_event.dart';
import 'battery_state.dart';

class BatteryBloc extends Bloc<BatteryEvent, BatteryState> {
  final IBatteryService _service;

  BatteryBloc({IBatteryService? service})
    : _service = service ?? BridgeService(),
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
      final status = await _service.getBatteryStatus();
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
        final status = await _service.getBatteryStatus();
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
}
