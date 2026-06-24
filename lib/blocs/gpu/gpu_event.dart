import '../../models/gpu_models.dart';

abstract class GpuEvent {}

class LoadCapabilitiesEvent extends GpuEvent {}

class LoadStatusEvent extends GpuEvent {}

class SetModeRequestedEvent extends GpuEvent {
  final GpuMode mode;

  SetModeRequestedEvent({required this.mode});
}

class ConfirmRebootEvent extends GpuEvent {}
