import '../../services/cpu_boost_service.dart';

abstract class CpuBoostEvent {}

class LoadCpuBoostStatus extends CpuBoostEvent {}

class SetCpuBoostModeRequested extends CpuBoostEvent {
  final CpuBoostMode mode;

  SetCpuBoostModeRequested(this.mode);
}
