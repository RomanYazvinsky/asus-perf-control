// GPU model classes (manually defined - not generated)
// These types are used with supergfxctl D-Bus interface

enum GpuMode { integrated, hybrid, ultimate, asusMuxDgpu, asusEgpu, vfio }

enum GpuVendor { nvidia, amd }

enum GpuPowerState { active, idle, suspended, unknown }

class GpuConfiguration {
  final bool hasDgpu;
  final GpuVendor? dgpuVendor;
  final bool hasMuxSwitch;
  final bool supportsEgpu;
  final bool externalGpuConnected;
  final bool vfioAvailable;

  const GpuConfiguration({
    required this.hasDgpu,
    this.dgpuVendor,
    required this.hasMuxSwitch,
    required this.supportsEgpu,
    required this.externalGpuConnected,
    required this.vfioAvailable,
  });

  @override
  int get hashCode =>
      hasDgpu.hashCode ^
      dgpuVendor.hashCode ^
      hasMuxSwitch.hashCode ^
      supportsEgpu.hashCode ^
      externalGpuConnected.hashCode ^
      vfioAvailable.hashCode;

  @override
  bool operator ==(Object other) =>
      identical(this, other) ||
      other is GpuConfiguration &&
          runtimeType == other.runtimeType &&
          hasDgpu == other.hasDgpu &&
          dgpuVendor == other.dgpuVendor &&
          hasMuxSwitch == other.hasMuxSwitch &&
          supportsEgpu == other.supportsEgpu &&
          externalGpuConnected == other.externalGpuConnected &&
          vfioAvailable == other.vfioAvailable;
}

class GpuStatus {
  final GpuMode activeMode;
  final bool dgpuEnabled;
  final double? temperature;
  final double? utilization;
  final GpuPowerState powerState;

  const GpuStatus({
    required this.activeMode,
    required this.dgpuEnabled,
    this.temperature,
    this.utilization,
    required this.powerState,
  });

  @override
  int get hashCode =>
      activeMode.hashCode ^
      dgpuEnabled.hashCode ^
      temperature.hashCode ^
      utilization.hashCode ^
      powerState.hashCode;

  @override
  bool operator ==(Object other) =>
      identical(this, other) ||
      other is GpuStatus &&
          runtimeType == other.runtimeType &&
          activeMode == other.activeMode &&
          dgpuEnabled == other.dgpuEnabled &&
          temperature == other.temperature &&
          utilization == other.utilization &&
          powerState == other.powerState;
}
