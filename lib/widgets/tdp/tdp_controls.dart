import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../blocs/tdp/tdp_bloc.dart';
import '../../blocs/tdp/tdp_event.dart';
import '../../blocs/tdp/tdp_state.dart';
import '../profile_tab_bar.dart';
import 'tdp_slider_card.dart';

class TdpControls extends StatelessWidget {
  final TdpState state;
  final double sustainedW;
  final double boostW;
  final double tempLimitC;
  final double minW;
  final double maxW;
  final double minBoostW;
  final double maxBoostW;
  final bool powerLimitsEnabled;
  final bool tempLimitEnabled;
  final ValueChanged<bool> onPowerLimitsEnabledChanged;
  final ValueChanged<bool> onTempLimitEnabledChanged;
  final ValueChanged<double> onSustainedChanged;
  final ValueChanged<double> onBoostChanged;
  final ValueChanged<double> onTempChanged;

  const TdpControls({
    super.key,
    required this.state,
    required this.sustainedW,
    required this.boostW,
    required this.tempLimitC,
    required this.minW,
    required this.maxW,
    required this.minBoostW,
    required this.maxBoostW,
    required this.powerLimitsEnabled,
    required this.tempLimitEnabled,
    required this.onPowerLimitsEnabledChanged,
    required this.onTempLimitEnabledChanged,
    required this.onSustainedChanged,
    required this.onBoostChanged,
    required this.onTempChanged,
  });

  @override
  Widget build(BuildContext context) {
    final canApply = powerLimitsEnabled || tempLimitEnabled;
    final activeProfileLabel = state is TdpLoaded
        ? _profileLabel((state as TdpLoaded).activePerformanceMode)
        : 'Balanced';

    return SingleChildScrollView(
      padding: const EdgeInsets.all(16.0),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          ProfileTabBar(
            selectedMode: state is TdpLoaded
                ? (state as TdpLoaded).activePerformanceMode
                : 1,
            onModeSelected: (mode) {
              context.read<TdpBloc>().add(SelectTdpProfile(mode));
            },
          ),
          const SizedBox(height: 16),
          Text(
            'CPU Power Limits',
            style: Theme.of(context).textTheme.headlineMedium,
          ),
          const SizedBox(height: 8),
          Text(
            'Configure sustained and boost TDP limits. '
            'Values are persistent — asusd saves them to its profile.',
            style: Theme.of(context).textTheme.bodyMedium,
          ),
          const SizedBox(height: 16),

          Card(
            child: SwitchListTile(
              title: const Text('Enable Power Limits'),
              subtitle: Text(
                powerLimitsEnabled
                    ? 'Custom sustained and boost limits are active'
                    : 'Power limits will not be changed',
                style: Theme.of(context).textTheme.bodySmall,
              ),
              value: powerLimitsEnabled,
              onChanged: onPowerLimitsEnabledChanged,
            ),
          ),
          const SizedBox(height: 16),

          TdpSliderCard(
            label: 'Sustained TDP (PL1/SPL)',
            value: sustainedW,
            min: minW,
            max: maxW,
            unit: 'W',
            description: 'Long-term power limit under sustained load',
            enabled: powerLimitsEnabled,
            onChanged: onSustainedChanged,
          ),
          const SizedBox(height: 12),

          // Boost TDP only shown if supported (maxBoostW > 0)
          if (maxBoostW > 0) ...[
            TdpSliderCard(
              label: 'Boost TDP (PL2/SPPT)',
              value: boostW.clamp(minBoostW, maxBoostW),
              min: minBoostW,
              max: maxBoostW,
              unit: 'W',
              description: 'Short-burst power limit (PL2/SPPT)',
              enabled: powerLimitsEnabled,
              onChanged: onBoostChanged,
            ),
            const SizedBox(height: 12),
          ],

          const SizedBox(height: 12),
          Text(
            'CPU Temperature Limit',
            style: Theme.of(context).textTheme.headlineMedium,
          ),
          const SizedBox(height: 8),
          Wrap(
            spacing: 8,
            crossAxisAlignment: WrapCrossAlignment.center,
            children: [
              Text(
                'Active profile',
                style: Theme.of(context).textTheme.bodyMedium,
              ),
              Chip(
                avatar: const Icon(Icons.speed, size: 18),
                label: Text(activeProfileLabel),
              ),
            ],
          ),
          const SizedBox(height: 8),
          Text(
            'Configure the CPU temperature ceiling independently from power limits.',
            style: Theme.of(context).textTheme.bodyMedium,
          ),
          const SizedBox(height: 16),

          Card(
            child: SwitchListTile(
              title: const Text('Enable CPU Temperature Limit'),
              subtitle: Text(
                tempLimitEnabled
                    ? 'Custom CPU temperature ceiling is active'
                    : 'Temperature limit will not be changed',
                style: Theme.of(context).textTheme.bodySmall,
              ),
              value: tempLimitEnabled,
              onChanged: onTempLimitEnabledChanged,
            ),
          ),
          const SizedBox(height: 16),

          TdpSliderCard(
            label: 'CPU Temperature Limit',
            value: tempLimitC,
            min: 60,
            max: 100,
            unit: '\u00b0C',
            description: 'CPU throttles when this temperature is reached',
            enabled: tempLimitEnabled,
            onChanged: onTempChanged,
          ),
          Padding(
            padding: const EdgeInsets.only(left: 12, top: 2),
            child: Text(
              'Persists across reboots (saved to disk)',
              style: Theme.of(context).textTheme.bodySmall?.copyWith(
                color: Theme.of(context).colorScheme.onSurfaceVariant,
              ),
            ),
          ),
          const SizedBox(height: 24),

          Row(
            children: [
              Expanded(
                child: FilledButton.icon(
                  key: const Key('apply_button'),
                  onPressed: (state is TdpLoading || !canApply)
                      ? null
                      : () {
                          context.read<TdpBloc>().add(
                            SetTdpRequested(
                              applyPowerLimits: powerLimitsEnabled,
                              applyTempLimit: tempLimitEnabled,
                              sustainedW: sustainedW.round(),
                              boostW: boostW.round(),
                              tempLimitC: tempLimitC.round(),
                            ),
                          );
                        },
                  icon: const Icon(Icons.check),
                  label: const Text('Apply'),
                ),
              ),
              const SizedBox(width: 12),
              Expanded(
                child: OutlinedButton.icon(
                  key: const Key('reset_button'),
                  onPressed: state is TdpLoading
                      ? null
                      : () => context.read<TdpBloc>().add(ResetTdpToDefaults()),
                  icon: const Icon(Icons.refresh),
                  label: const Text('Reload from Hardware'),
                ),
              ),
            ],
          ),

          if (!canApply) ...[
            const SizedBox(height: 16),
            Card(
              color: Theme.of(context).colorScheme.secondaryContainer,
              child: Padding(
                padding: const EdgeInsets.all(12.0),
                child: Row(
                  children: [
                    Icon(
                      Icons.info_outline,
                      color: Theme.of(context).colorScheme.onSecondaryContainer,
                    ),
                    const SizedBox(width: 8),
                    Expanded(
                      child: Text(
                        'Enable at least one limit to apply changes',
                        style: TextStyle(
                          color: Theme.of(
                            context,
                          ).colorScheme.onSecondaryContainer,
                        ),
                      ),
                    ),
                  ],
                ),
              ),
            ),
          ],

          if (state is TdpError) ...[
            const SizedBox(height: 16),
            Card(
              color: Theme.of(context).colorScheme.errorContainer,
              child: Padding(
                padding: const EdgeInsets.all(12.0),
                child: Row(
                  children: [
                    Icon(
                      Icons.error_outline,
                      color: Theme.of(context).colorScheme.onErrorContainer,
                    ),
                    const SizedBox(width: 8),
                    Expanded(
                      child: Text(
                        (state as TdpError).message,
                        style: TextStyle(
                          color: Theme.of(context).colorScheme.onErrorContainer,
                        ),
                      ),
                    ),
                  ],
                ),
              ),
            ),
          ],
        ],
      ),
    );
  }

  String _profileLabel(int mode) {
    return switch (mode) {
      0 => 'Silent',
      1 => 'Balanced',
      2 => 'Turbo',
      _ => 'Balanced',
    };
  }
}
