// TDP BLoC events

abstract class TdpEvent {}

/// Load current TDP configuration from hardware
class LoadTdpConfig extends TdpEvent {}

/// Switch active performance profile and reload restored profile settings
class SelectTdpProfile extends TdpEvent {
  final int mode;

  SelectTdpProfile(this.mode);
}

/// Apply selected CPU power and/or temperature limits to hardware
class SetTdpRequested extends TdpEvent {
  final bool applyPowerLimits;
  final bool applyTempLimit;
  final int sustainedW;
  final int boostW;
  final int tempLimitC;

  SetTdpRequested({
    required this.applyPowerLimits,
    required this.applyTempLimit,
    required this.sustainedW,
    required this.boostW,
    required this.tempLimitC,
  });
}

/// Reset TDP limits to firmware/mode defaults (skip all params)
class ResetTdpToDefaults extends TdpEvent {}

/// Clear custom CPU power limits and return to profile defaults
class ClearPowerLimitsRequested extends TdpEvent {}
