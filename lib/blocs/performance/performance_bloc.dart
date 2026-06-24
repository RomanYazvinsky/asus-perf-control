import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:bloc/bloc.dart';

import '../../services/bridge_service.dart';
import '../../services/performance_service.dart';
import 'performance_event.dart';
import 'performance_state.dart';

class PerformanceBloc extends Bloc<PerformanceEvent, PerformanceState> {
  final IPerformanceService _service;

  PerformanceBloc({IPerformanceService? service})
    : _service = service ?? BridgeService(),
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
      final currentMode = await _service.getPerformanceMode();
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
      emit(PerformanceLoaded(event.mode));
    } catch (e) {
      emit(PerformanceError(e.toString()));
    }
  }
}
