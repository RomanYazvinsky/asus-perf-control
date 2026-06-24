// Battery BLoC events

abstract class BatteryEvent {
  const BatteryEvent();
}

/// Load current battery charge limit
class LoadCurrentLimitEvent extends BatteryEvent {
  const LoadCurrentLimitEvent();
}

/// Request to set battery charge limit
class SetLimitRequestedEvent extends BatteryEvent {
  final int limit;

  const SetLimitRequestedEvent(this.limit);

  @override
  String toString() => 'SetLimitRequestedEvent(limit: $limit)';
}

/// Check if battery charge limit is supported on this hardware
class CheckBatterySupportEvent extends BatteryEvent {
  const CheckBatterySupportEvent();
}
