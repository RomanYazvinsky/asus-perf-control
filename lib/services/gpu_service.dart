import '../models/gpu_models.dart';
import 'supergfx_client.dart';

abstract class IGpuService {
  Future<GpuConfiguration> getCapabilities();
  Future<GpuStatus> getStatus();
  Future<void> setGpuMode(GpuMode mode);
  Future<List<GpuMode>> getSupportedModes();
  Future<void> confirmMuxAndReboot();
  Future<void> rebindDgpuToGraphics();
}

/// GPU service using supergfxctl D-Bus interface
class SupergfxGpuService implements IGpuService {
  final SupergfxClient _client = SupergfxClient();

  /// Map supergfxctl mode to our GpuMode enum
  GpuMode _mapMode(SupergfxMode mode) {
    switch (mode) {
      case SupergfxMode.integrated:
        return GpuMode.integrated;
      case SupergfxMode.hybrid:
        return GpuMode.hybrid;
      case SupergfxMode.vfio:
        return GpuMode.vfio;
      case SupergfxMode.asusMuxDgpu:
        return GpuMode.asusMuxDgpu;
      case SupergfxMode.asusEgpu:
        return GpuMode.asusEgpu;
      default:
        return GpuMode.hybrid;
    }
  }

  /// Map our GpuMode to supergfxctl mode
  SupergfxMode _toSupergfxMode(GpuMode mode) {
    switch (mode) {
      case GpuMode.integrated:
        return SupergfxMode.integrated;
      case GpuMode.hybrid:
        return SupergfxMode.hybrid;
      case GpuMode.ultimate:
        return SupergfxMode.hybrid; // Ultimate = Hybrid in supergfxctl
      case GpuMode.asusMuxDgpu:
        return SupergfxMode.asusMuxDgpu;
      case GpuMode.asusEgpu:
        return SupergfxMode.asusEgpu;
      case GpuMode.vfio:
        return SupergfxMode.vfio;
    }
  }

  @override
  Future<GpuConfiguration> getCapabilities() async {
    final supported = await _client.getSupportedModes();
    final vendor = await _client.getVendor();

    return GpuConfiguration(
      hasDgpu: vendor.toLowerCase() != 'unknown',
      dgpuVendor: vendor.toLowerCase() == 'nvidia'
          ? GpuVendor.nvidia
          : vendor.toLowerCase() == 'amd'
          ? GpuVendor.amd
          : null,
      hasMuxSwitch: supported.contains(SupergfxMode.asusMuxDgpu),
      supportsEgpu: supported.contains(SupergfxMode.asusEgpu),
      externalGpuConnected: supported.contains(SupergfxMode.asusEgpu),
      vfioAvailable: supported.contains(SupergfxMode.vfio),
    );
  }

  @override
  Future<GpuStatus> getStatus() async {
    final mode = await _client.getMode();

    return GpuStatus(
      activeMode: _mapMode(mode),
      dgpuEnabled: mode != SupergfxMode.integrated,
      temperature: null, // supergfxctl doesn't provide temp
      utilization: null, // supergfxctl doesn't provide utilization
      powerState: mode == SupergfxMode.integrated
          ? GpuPowerState.suspended
          : GpuPowerState.active,
    );
  }

  @override
  Future<void> setGpuMode(GpuMode mode) async {
    try {
      final supergfxMode = _toSupergfxMode(mode);
      final action = await _client.setMode(supergfxMode);

      switch (action) {
        case SupergfxUserAction.reboot:
          throw Exception('Reboot required. Please save your work and reboot.');
        case SupergfxUserAction.logout:
          throw Exception('Logout required. Please save your work and logout.');
        case SupergfxUserAction.nothing:
          // Success
          break;
        default:
          throw Exception('Action required: ${action.name}');
      }
    } catch (e) {
      final message = e.toString().replaceFirst('Exception: ', '');
      if (message.startsWith('Failed to set GPU mode:')) {
        throw Exception(message);
      }
      throw Exception('Failed to set GPU mode: $message');
    }
  }

  @override
  Future<List<GpuMode>> getSupportedModes() async {
    final modes = await _client.getSupportedModes();
    return modes.map(_mapMode).toList();
  }

  @override
  Future<void> confirmMuxAndReboot() async {
    throw UnsupportedError(
      'MUX reboot is handled by supergfxctl automatically. Use setGpuMode instead.',
    );
  }

  @override
  Future<void> rebindDgpuToGraphics() async {
    throw UnsupportedError(
      'GPU rebinding is handled by supergfxctl automatically.',
    );
  }
}
