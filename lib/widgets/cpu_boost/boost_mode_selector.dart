import 'package:flutter/material.dart';

import '../../services/cpu_boost_service.dart';

class BoostModeSelector extends StatelessWidget {
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
                  for (final mode
                      in status?.availableModes ?? CpuBoostMode.values)
                    if ((status?.availableModes ?? CpuBoostMode.values)
                        .contains(mode))
                      Padding(
                        padding: const EdgeInsets.only(bottom: 8),
                        child: _BoostModeButton(
                          mode: mode,
                          isSelected: status?.currentMode == mode,
                          onModeSelected: onModeSelected,
                        ),
                      ),
                ],
              ),
          ],
        ),
      ),
    );
  }
}

class _BoostModeButton extends StatelessWidget {
  final CpuBoostMode mode;
  final bool isSelected;
  final ValueChanged<CpuBoostMode> onModeSelected;

  const _BoostModeButton({
    required this.mode,
    required this.isSelected,
    required this.onModeSelected,
  });

  @override
  Widget build(BuildContext context) {
    return SizedBox(
      width: double.infinity,
      child: FilledButton.tonal(
        onPressed: () => onModeSelected(mode),
        style: FilledButton.styleFrom(
          backgroundColor: isSelected
              ? Theme.of(context).colorScheme.primaryContainer
              : null,
          padding: const EdgeInsets.all(16),
        ),
        child: Row(
          children: [
            Icon(_iconForMode(mode), size: 32),
            const SizedBox(width: 16),
            Expanded(
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Text(
                    mode.label,
                    style: Theme.of(context).textTheme.titleMedium?.copyWith(
                      fontWeight: isSelected
                          ? FontWeight.bold
                          : FontWeight.normal,
                    ),
                  ),
                  Text(
                    _descriptionForMode(mode),
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

  IconData _iconForMode(CpuBoostMode mode) {
    return switch (mode) {
      CpuBoostMode.disabled => Icons.block,
      CpuBoostMode.efficient => Icons.eco,
      CpuBoostMode.efficientEnabled => Icons.energy_savings_leaf,
      CpuBoostMode.enabled => Icons.trending_up,
      CpuBoostMode.efficientAggressive => Icons.bolt,
      CpuBoostMode.aggressive => Icons.rocket_launch,
    };
  }

  String _descriptionForMode(CpuBoostMode mode) {
    return switch (mode) {
      CpuBoostMode.disabled => 'No CPU boost',
      CpuBoostMode.efficient => 'Lowest boost bias',
      CpuBoostMode.efficientEnabled => 'Efficient normal boost',
      CpuBoostMode.enabled => 'Balanced boost',
      CpuBoostMode.efficientAggressive => 'Responsive efficient boost',
      CpuBoostMode.aggressive => 'Maximum boost response',
    };
  }
}
