import 'package:flutter/material.dart';

import '../../services/cpu_boost_service.dart';

class BoostModeSelector extends StatelessWidget {
  static const List<_BoostPreset> _presets = [
    _BoostPreset(
      id: 'off',
      label: 'Off',
      description: 'Disable CPU boost',
      icon: Icons.block,
      modes: [CpuBoostMode.disabled],
    ),
    _BoostPreset(
      id: 'efficient',
      label: 'Efficient',
      description: 'Boost with lower power bias',
      icon: Icons.energy_savings_leaf,
      modes: [CpuBoostMode.efficientEnabled, CpuBoostMode.efficient],
    ),
    _BoostPreset(
      id: 'balanced',
      label: 'Balanced',
      description: 'Responsive boost for normal use',
      icon: Icons.trending_up,
      modes: [CpuBoostMode.efficientAggressive, CpuBoostMode.enabled],
    ),
    _BoostPreset(
      id: 'aggressive',
      label: 'Aggressive',
      description: 'Maximum boost response',
      icon: Icons.rocket_launch,
      modes: [CpuBoostMode.aggressive],
    ),
  ];

  final CpuBoostStatus? status;
  final ValueChanged<CpuBoostMode> onModeSelected;
  final bool isLoading;
  final bool isUnsupported;

  const BoostModeSelector({
    super.key,
    required this.status,
    required this.onModeSelected,
    this.isLoading = false,
    this.isUnsupported = false,
  });

  @override
  Widget build(BuildContext context) {
    return Card(
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text(
              'CPU Boost Mode',
              style: Theme.of(context).textTheme.headlineSmall,
            ),
            const SizedBox(height: 16),
            if (isLoading)
              const Center(child: CircularProgressIndicator())
            else if (isUnsupported || status?.isSupported == false)
              const Text('CPU boost controls are unavailable')
            else
              Column(
                children: [
                  for (final preset in _availablePresets(status))
                    if (preset.resolveMode(status?.availableModes) != null)
                      Padding(
                        padding: const EdgeInsets.only(bottom: 8),
                        child: _BoostModeButton(
                          key: Key('cpu_boost_${preset.id}_button'),
                          preset: preset,
                          isSelected: preset.includes(status?.currentMode),
                          onModeSelected: () {
                            final mode = preset.resolveMode(
                              status?.availableModes,
                            );
                            if (mode != null) {
                              onModeSelected(mode);
                            }
                          },
                        ),
                      ),
                ],
              ),
          ],
        ),
      ),
    );
  }

  List<_BoostPreset> _availablePresets(CpuBoostStatus? status) {
    final availableModes = status?.availableModes ?? CpuBoostMode.values;
    return _presets
        .where((preset) => preset.resolveMode(availableModes) != null)
        .toList(growable: false);
  }
}

class _BoostModeButton extends StatelessWidget {
  final _BoostPreset preset;
  final bool isSelected;
  final VoidCallback onModeSelected;

  const _BoostModeButton({
    super.key,
    required this.preset,
    required this.isSelected,
    required this.onModeSelected,
  });

  @override
  Widget build(BuildContext context) {
    return SizedBox(
      width: double.infinity,
      child: FilledButton.tonal(
        onPressed: onModeSelected,
        style: FilledButton.styleFrom(
          backgroundColor: isSelected
              ? Theme.of(context).colorScheme.primaryContainer
              : null,
          padding: const EdgeInsets.all(16),
        ),
        child: Row(
          children: [
            Icon(preset.icon, size: 32),
            const SizedBox(width: 16),
            Expanded(
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Text(
                    preset.label,
                    style: Theme.of(context).textTheme.titleMedium?.copyWith(
                      fontWeight: isSelected
                          ? FontWeight.bold
                          : FontWeight.normal,
                    ),
                  ),
                  Text(
                    preset.description,
                    style: Theme.of(context).textTheme.bodySmall,
                  ),
                ],
              ),
            ),
            if (isSelected) const Icon(Icons.check_circle, color: Colors.green),
          ],
        ),
      ),
    );
  }
}

class _BoostPreset {
  final String id;
  final String label;
  final String description;
  final IconData icon;
  final List<CpuBoostMode> modes;

  const _BoostPreset({
    required this.id,
    required this.label,
    required this.description,
    required this.icon,
    required this.modes,
  });

  bool includes(CpuBoostMode? mode) {
    return mode != null && modes.contains(mode);
  }

  CpuBoostMode? resolveMode(List<CpuBoostMode>? availableModes) {
    final available = availableModes ?? CpuBoostMode.values;
    for (final mode in modes) {
      if (available.contains(mode)) {
        return mode;
      }
    }
    return null;
  }
}
