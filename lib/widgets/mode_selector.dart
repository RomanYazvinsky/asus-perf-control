// Performance mode selector widget
import 'package:flutter/material.dart';

class ModeSelector extends StatelessWidget {
  final int? currentMode;
  final Function(int) onModeSelected;
  final bool isLoading;

  const ModeSelector({
    super.key,
    required this.currentMode,
    required this.onModeSelected,
    this.isLoading = false,
  });

  @override
  Widget build(BuildContext context) {
    return Card(
      child: Padding(
        padding: const EdgeInsets.all(16.0),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text(
              'Performance Mode',
              style: Theme.of(context).textTheme.headlineSmall,
            ),
            const SizedBox(height: 16),
            if (isLoading)
              const Center(child: CircularProgressIndicator())
            else
              Column(
                children: [
                  _buildModeButton(
                    context,
                    mode: 0,
                    title: 'Silent',
                    subtitle: 'Quiet operation, lower performance',
                    icon: Icons.volume_off,
                  ),
                  const SizedBox(height: 8),
                  _buildModeButton(
                    context,
                    mode: 1,
                    title: 'Balanced',
                    subtitle: 'Optimal balance of performance and noise',
                    icon: Icons.balance,
                  ),
                  const SizedBox(height: 8),
                  _buildModeButton(
                    context,
                    mode: 2,
                    title: 'Turbo',
                    subtitle: 'Maximum performance, higher noise',
                    icon: Icons.speed,
                  ),
                ],
              ),
          ],
        ),
      ),
    );
  }

  Widget _buildModeButton(
    BuildContext context, {
    required int mode,
    required String title,
    required String subtitle,
    required IconData icon,
  }) {
    final isSelected = currentMode == mode;

    return SizedBox(
      width: double.infinity,
      child: FilledButton.tonal(
        onPressed: isLoading ? null : () => onModeSelected(mode),
        style: FilledButton.styleFrom(
          backgroundColor: isSelected
              ? Theme.of(context).colorScheme.primaryContainer
              : null,
          padding: const EdgeInsets.all(16),
        ),
        child: Row(
          children: [
            Icon(icon, size: 32),
            const SizedBox(width: 16),
            Expanded(
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Text(
                    title,
                    style: Theme.of(context).textTheme.titleMedium?.copyWith(
                      fontWeight: isSelected
                          ? FontWeight.bold
                          : FontWeight.normal,
                    ),
                  ),
                  Text(subtitle, style: Theme.of(context).textTheme.bodySmall),
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
