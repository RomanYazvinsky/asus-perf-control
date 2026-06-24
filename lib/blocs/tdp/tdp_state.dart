// TDP BLoC states

abstract class TdpState {}

/// Initial state before any load
class TdpInitial extends TdpState {}

/// Loading TDP config from hardware
class TdpLoading extends TdpState {}

/// TDP config successfully loaded
class TdpLoaded extends TdpState {
  final int sustainedW;
  final int boostW;
  final int tempLimitC;
  final int activePerformanceMode;
  final int minW;
  final int maxW;
  final int minBoostW;
  final int maxBoostW;
  final bool powerLimitsEnabled;
  final bool tempLimitEnabled;

  TdpLoaded({
    required this.sustainedW,
    required this.boostW,
    required this.tempLimitC,
    this.activePerformanceMode = 1,
    this.minW = 5,
    this.maxW = 54,
    this.minBoostW = 5,
    this.maxBoostW = 54,
    bool? powerLimitsEnabled,
    bool? tempLimitEnabled,
  }) : powerLimitsEnabled = powerLimitsEnabled ?? false,
       tempLimitEnabled = tempLimitEnabled ?? tempLimitC > 0;
}

/// Hardware does not support TDP control (kernel < 5.17 or non-ASUS)
class TdpUnsupported extends TdpState {}

/// Error applying or reading TDP config
class TdpError extends TdpState {
  final String message;

  TdpError(this.message);
}
