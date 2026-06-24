// Fan BLoC states

import '../../services/fan_service.dart';
import '../../services/asusd_fan_client.dart';

abstract class FanState {}

class FanInitial extends FanState {}

class FanLoading extends FanState {}

class FanLoaded extends FanState {
  final FanCapabilities capabilities; // detected fans
  final Map<FanId, int> fanRpms; // live RPM per fan
  final int cpuTempC;
  final int gpuTempC;
  final FanProfileCurves asusdCurves; // fan -> profile -> curve
  final FanId selectedFan;
  final PerfProfile selectedProfile;
  final Map<FanId, FanControlMode> controlModes; // current mode per fan
  final bool dynamicEngineRunning;
  final FanEngineConfig? engineConfig; // null if engine not running

  FanLoaded({
    required this.capabilities,
    required this.fanRpms,
    this.cpuTempC = 0,
    this.gpuTempC = 0,
    required this.asusdCurves,
    required this.selectedFan,
    required this.selectedProfile,
    required this.controlModes,
    this.dynamicEngineRunning = false,
    this.engineConfig,
  });

  FanLoaded copyWith({
    FanCapabilities? capabilities,
    Map<FanId, int>? fanRpms,
    int? cpuTempC,
    int? gpuTempC,
    FanProfileCurves? asusdCurves,
    FanId? selectedFan,
    PerfProfile? selectedProfile,
    Map<FanId, FanControlMode>? controlModes,
    bool? dynamicEngineRunning,
    FanEngineConfig? engineConfig,
    bool clearEngineConfig = false,
  }) {
    return FanLoaded(
      capabilities: capabilities ?? this.capabilities,
      fanRpms: fanRpms ?? this.fanRpms,
      cpuTempC: cpuTempC ?? this.cpuTempC,
      gpuTempC: gpuTempC ?? this.gpuTempC,
      asusdCurves: asusdCurves ?? this.asusdCurves,
      selectedFan: selectedFan ?? this.selectedFan,
      selectedProfile: selectedProfile ?? this.selectedProfile,
      controlModes: controlModes ?? this.controlModes,
      dynamicEngineRunning: dynamicEngineRunning ?? this.dynamicEngineRunning,
      engineConfig: clearEngineConfig
          ? null
          : (engineConfig ?? this.engineConfig),
    );
  }
}

class FanUnsupported extends FanState {
  final String message;
  FanUnsupported(this.message);
}

class FanError extends FanState {
  final String message;
  FanError(this.message);
}

class FanMaxSpeedTesting extends FanState {
  final FanId fanId;
  final int currentRpm;
  final int elapsedSeconds;
  FanMaxSpeedTesting({
    required this.fanId,
    required this.currentRpm,
    required this.elapsedSeconds,
  });
}

class FanMaxSpeedResult extends FanState {
  final FanLoaded previous; // restore to this after test
  final FanId fanId;
  final int maxRpm;
  FanMaxSpeedResult({
    required this.previous,
    required this.fanId,
    required this.maxRpm,
  });
}
