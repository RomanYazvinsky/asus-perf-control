import '../../services/cpu_boost_service.dart';

abstract class CpuBoostState {}

class CpuBoostInitial extends CpuBoostState {}

class CpuBoostLoading extends CpuBoostState {}

class CpuBoostLoaded extends CpuBoostState {
  final CpuBoostStatus status;

  CpuBoostLoaded(this.status);
}

class CpuBoostUnsupported extends CpuBoostState {}

class CpuBoostError extends CpuBoostState {
  final String message;

  CpuBoostError(this.message);
}
