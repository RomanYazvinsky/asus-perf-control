// Fan BLoC events

import '../../services/fan_service.dart';
import '../../services/asusd_fan_client.dart';

abstract class FanEvent {}

// Initial load
class LoadFanCapabilities extends FanEvent {}

class RefreshFanStatus extends FanEvent {} // periodic RPM poll (every 2s)

class SelectFan extends FanEvent {
  final FanId fanId;
  SelectFan(this.fanId);
}

class SelectProfile extends FanEvent {
  final PerfProfile profile;
  SelectProfile(this.profile);
}

// Mode: AsusctlCurve — edit 8-point curve, push to asusd
class SetAsusctlCurve extends FanEvent {
  final FanId fanId;
  final PerfProfile profile;
  final List<FanCurvePoint> curve; // exactly 8 points
  SetAsusctlCurve({
    required this.fanId,
    required this.profile,
    required this.curve,
  });
}

// Mode: ConstantSpeed — flat curve via asusd (no daemon)
class SetConstantSpeed extends FanEvent {
  final FanId fanId;
  final PerfProfile profile;
  final int speedPercent; // 0–100
  SetConstantSpeed({
    required this.fanId,
    required this.profile,
    required this.speedPercent,
  });
}

// Mode: DynamicEngine — daemon tick loop via hwmon7 sysfs
class StartDynamicEngine extends FanEvent {
  final FanEngineConfig config; // includes cross-fan toggle
  StartDynamicEngine(this.config);
}

class StopDynamicEngine extends FanEvent {}

class UpdateDynamicCurve extends FanEvent {
  final FanId fanId;
  final List<FanCurvePoint> curve;
  UpdateDynamicCurve({required this.fanId, required this.curve});
}

// Reset — uses asusd SetCurvesToDefaults
class ResetToDefaults extends FanEvent {
  final PerfProfile profile;
  ResetToDefaults(this.profile);
}

// Max speed test — flat 100% via asusd, poll RPM 10s, then restore
class StartMaxSpeedTest extends FanEvent {
  final FanId fanId;
  StartMaxSpeedTest(this.fanId);
}
