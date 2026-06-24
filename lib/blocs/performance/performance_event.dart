// Performance events for BLoC pattern

abstract class PerformanceEvent {}

/// Request to load the current performance mode from hardware
class LoadCurrentModeEvent extends PerformanceEvent {}

/// Request to set a new performance mode
class SetModeRequestedEvent extends PerformanceEvent {
  final int mode;

  SetModeRequestedEvent(this.mode);
}
