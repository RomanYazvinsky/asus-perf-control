import 'package:flutter_bloc/flutter_bloc.dart';

import '../../services/bridge_service.dart';
import '../../services/cpu_boost_service.dart';
import 'cpu_boost_event.dart';
import 'cpu_boost_state.dart';

class CpuBoostBloc extends Bloc<CpuBoostEvent, CpuBoostState> {
  final ICpuBoostService _service;

  CpuBoostBloc({ICpuBoostService? service})
    : _service = service ?? BridgeService(),
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

  Future<void> _onSetCpuBoostModeRequested(
    SetCpuBoostModeRequested event,
    Emitter<CpuBoostState> emit,
  ) async {
    emit(CpuBoostLoading());
    try {
      await _service.setCpuBoostMode(event.mode);
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
}
