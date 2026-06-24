import 'package:flutter/material.dart';
import '../../models/gpu_models.dart';

class GpuModeSelector extends StatelessWidget {
  final GpuConfiguration capabilities;
  final GpuMode currentMode;
  final List<GpuMode> supportedModes;
  final ValueChanged<GpuMode> onModeSelected;

  const GpuModeSelector({
    super.key,
    required this.capabilities,
    required this.currentMode,
    required this.supportedModes,
    required this.onModeSelected,
  });

  bool _isModeSupported(GpuMode mode) {
    return supportedModes.contains(mode);
  }

  bool _isSelectable(GpuMode mode) {
    if (mode == GpuMode.asusEgpu) {
      return capabilities.externalGpuConnected ||
          currentMode == GpuMode.asusEgpu;
    }
    return true;
  }

  RadioListTile<GpuMode> _modeTile({
    required GpuMode mode,
    required Widget title,
    required Widget subtitle,
  }) {
    return RadioListTile<GpuMode>(
      title: title,
      subtitle: subtitle,
      value: mode,
      enabled: _isSelectable(mode),
    );
  }

  @override
  Widget build(BuildContext context) {
    return Card(
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Row(
              children: [
                const Icon(Icons.tune),
                const SizedBox(width: 8),
                Text('GPU Mode', style: Theme.of(context).textTheme.titleLarge),
              ],
            ),
            const Divider(),
            RadioGroup<GpuMode>(
              groupValue: currentMode,
              onChanged: (value) {
                if (value != null && _isSelectable(value)) {
                  onModeSelected(value);
                }
              },
              child: Column(
                children: [
                  if (_isModeSupported(GpuMode.integrated))
                    _modeTile(
                      mode: GpuMode.integrated,
                      title: const Text('Integrated'),
                      subtitle: const Text('Battery saving - dGPU disabled'),
                    ),
                  if (_isModeSupported(GpuMode.hybrid))
                    _modeTile(
                      mode: GpuMode.hybrid,
                      title: const Text('Hybrid'),
                      subtitle: const Text('Balanced - dGPU on demand'),
                    ),
                  if (_isModeSupported(GpuMode.ultimate))
                    _modeTile(
                      mode: GpuMode.ultimate,
                      title: const Text('Ultimate'),
                      subtitle: const Text(
                        'Maximum performance - dGPU primary',
                      ),
                    ),
                  if (_isModeSupported(GpuMode.asusMuxDgpu))
                    _modeTile(
                      mode: GpuMode.asusMuxDgpu,
                      title: const Text('MUX dGPU'),
                      subtitle: const Text(
                        'Direct dGPU output (requires reboot)',
                      ),
                    ),
                  if (_isModeSupported(GpuMode.asusEgpu))
                    _modeTile(
                      mode: GpuMode.asusEgpu,
                      title: const Text('eGPU'),
                      subtitle: Text(
                        capabilities.externalGpuConnected
                            ? 'External GPU (ASUS XG Mobile)'
                            : 'Connect XG Mobile to enable',
                        style: capabilities.externalGpuConnected
                            ? null
                            : const TextStyle(color: Colors.orange),
                      ),
                    ),
                  if (_isModeSupported(GpuMode.vfio))
                    _modeTile(
                      mode: GpuMode.vfio,
                      title: const Text('VFIO'),
                      subtitle: const Text('GPU passthrough for VMs'),
                    ),
                ],
              ),
            ),
          ],
        ),
      ),
    );
  }
}
