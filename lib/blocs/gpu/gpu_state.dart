import '../../models/gpu_models.dart';

abstract class GpuState {}

class GpuInitial extends GpuState {}

class GpuLoading extends GpuState {}

class GpuLoaded extends GpuState {
  final GpuConfiguration capabilities;
  final GpuStatus status;
  final List<GpuMode> supportedModes;

  GpuLoaded({
    required this.capabilities,
    required this.status,
    required this.supportedModes,
  });
}

class GpuUnsupported extends GpuState {
  final String message;

  GpuUnsupported({required this.message});
}

class GpuError extends GpuState {
  final String message;

  GpuError({required this.message});
}

class GpuRequiresReboot extends GpuState {
  final String message;

  GpuRequiresReboot({required this.message});
}

class GpuRequiresLogout extends GpuState {
  final String message;

  GpuRequiresLogout({required this.message});
}
