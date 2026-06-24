// Battery BLoC states

abstract class BatteryState {
  const BatteryState();
}

/// Initial state before any data loaded
class BatteryInitial extends BatteryState {
  const BatteryInitial();
}

/// Loading data from service
class BatteryLoading extends BatteryState {
  const BatteryLoading();
}

/// Battery charge limit loaded successfully
class BatteryLoaded extends BatteryState {
  final int chargeLimit;
  final double? percentage;
  final String? state;

  const BatteryLoaded({required this.chargeLimit, this.percentage, this.state});

  @override
  String toString() =>
      'BatteryLoaded(chargeLimit: $chargeLimit, percentage: $percentage, state: $state)';
}

/// Battery charge limit not supported on this hardware
class BatteryUnsupported extends BatteryState {
  final String message;

  const BatteryUnsupported([
    this.message = 'Battery charge limit not supported',
  ]);
}

/// Error occurred while loading or setting charge limit
class BatteryError extends BatteryState {
  final String message;

  const BatteryError(this.message);

  @override
  String toString() => 'BatteryError(message: $message)';
}
