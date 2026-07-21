// Battery Charge Limit Screen
// Allows users to set maximum battery charge percentage

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../blocs/battery/battery_bloc.dart';
import '../blocs/battery/battery_event.dart';
import '../blocs/battery/battery_state.dart';
import '../services/settings/app_settings_store.dart';
import '../widgets/copyable_alert.dart';

class BatteryScreen extends StatefulWidget {
  final BatteryBloc? testBloc;
  final AppSettingsStore? settingsStore;

  const BatteryScreen({super.key, this.testBloc, this.settingsStore});

  @override
  State<BatteryScreen> createState() => _BatteryScreenState();
}

class _BatteryScreenState extends State<BatteryScreen> {
  late final BatteryBloc _batteryBloc;
  double _chargeLimit = 80;
  bool _hasLoadedData = false;
  bool _hasShownUnsupported = false;

  @override
  void initState() {
    super.initState();
    _batteryBloc =
        widget.testBloc ??
        (BatteryBloc(settingsStore: widget.settingsStore)
          ..add(const CheckBatterySupportEvent()));
  }

  @override
  void dispose() {
    _batteryBloc.close();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: const Text('Battery Charge Limit')),
      body: BlocProvider<BatteryBloc>.value(
        value: _batteryBloc,
        child: BlocListener<BatteryBloc, BatteryState>(
          listener: (context, state) {
            if (state is BatteryError) {
              CopyableAlert.showError(
                context,
                title: 'Battery Error',
                message: _hasLoadedData
                    ? 'Failed to set charge limit'
                    : 'Failed to load battery status',
                errorDetails: state.message,
              );
            } else if (state is BatteryUnsupported && !_hasShownUnsupported) {
              _hasShownUnsupported = true;
              CopyableAlert.showInfo(
                context,
                title: 'Battery Charge Limit Not Supported',
                message:
                    'Your hardware or kernel does not support charge limit control.',
                details:
                    'Requires Linux kernel 5.4+ with charge_control_end_threshold sysfs attribute.',
              );
            } else if (state is BatteryLoaded) {
              setState(() {
                _chargeLimit = state.chargeLimit.toDouble();
                _hasLoadedData = true;
              });
            }
          },
          child: BlocBuilder<BatteryBloc, BatteryState>(
            builder: (context, state) {
              if (state is BatteryUnsupported) {
                return const Center(child: CircularProgressIndicator());
              }
              if (state is BatteryLoading ||
                  state is BatteryInitial ||
                  state is BatteryError ||
                  !_hasLoadedData) {
                return const Center(child: CircularProgressIndicator());
              }
              return _buildBatteryControls(state);
            },
          ),
        ),
      ),
    );
  }

  Widget _buildBatteryControls(BatteryState state) {
    final theme = Theme.of(context);
    final colorScheme = theme.colorScheme;

    return SingleChildScrollView(
      padding: const EdgeInsets.all(16),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          // Info card
          Card(
            child: Padding(
              padding: const EdgeInsets.all(16),
              child: Row(
                children: [
                  Icon(Icons.info_outline, color: colorScheme.primary),
                  const SizedBox(width: 16),
                  Expanded(
                    child: Text(
                      'Setting a charge limit helps extend battery lifespan by preventing '
                      'overcharging. Recommended: 80% for daily use, 100% when you need full capacity.',
                      style: theme.textTheme.bodyMedium,
                    ),
                  ),
                ],
              ),
            ),
          ),
          const SizedBox(height: 24),

          // Charge limit slider card
          Card(
            child: Padding(
              padding: const EdgeInsets.all(20),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Row(
                    mainAxisAlignment: MainAxisAlignment.spaceBetween,
                    children: [
                      Text('Charge Limit', style: theme.textTheme.titleMedium),
                      Container(
                        padding: const EdgeInsets.symmetric(
                          horizontal: 12,
                          vertical: 6,
                        ),
                        decoration: BoxDecoration(
                          color: colorScheme.primaryContainer,
                          borderRadius: BorderRadius.circular(16),
                        ),
                        child: Text(
                          '${_chargeLimit.round()}%',
                          style: theme.textTheme.titleSmall?.copyWith(
                            color: colorScheme.onPrimaryContainer,
                            fontWeight: FontWeight.bold,
                          ),
                        ),
                      ),
                    ],
                  ),
                  const SizedBox(height: 8),
                  Text(
                    'Battery will stop charging at this percentage',
                    style: theme.textTheme.bodySmall?.copyWith(
                      color: colorScheme.onSurfaceVariant,
                    ),
                  ),
                  const SizedBox(height: 16),
                  Slider(
                    value: _chargeLimit,
                    min: 50,
                    max: 100,
                    divisions: 50,
                    label: '${_chargeLimit.round()}%',
                    onChanged: (value) {
                      setState(() {
                        _chargeLimit = value;
                      });
                    },
                  ),
                  Row(
                    mainAxisAlignment: MainAxisAlignment.spaceBetween,
                    children: [
                      Text('50%', style: theme.textTheme.bodySmall),
                      Text('100%', style: theme.textTheme.bodySmall),
                    ],
                  ),
                  const SizedBox(height: 16),

                  // Quick preset buttons
                  Row(
                    children: [
                      _buildPresetButton('80%', 80, theme),
                      const SizedBox(width: 8),
                      _buildPresetButton('90%', 90, theme),
                      const SizedBox(width: 8),
                      _buildPresetButton('100%', 100, theme),
                    ],
                  ),
                ],
              ),
            ),
          ),
          const SizedBox(height: 24),

          // Apply button
          SizedBox(
            width: double.infinity,
            child: FilledButton.icon(
              onPressed: state is BatteryLoading
                  ? null
                  : () => _batteryBloc.add(
                      SetLimitRequestedEvent(_chargeLimit.round()),
                    ),
              icon: state is BatteryLoading
                  ? const SizedBox(
                      width: 18,
                      height: 18,
                      child: CircularProgressIndicator(strokeWidth: 2),
                    )
                  : const Icon(Icons.save),
              label: Text(state is BatteryLoading ? 'Applying...' : 'Apply'),
            ),
          ),

          // Current status display
          if (state is BatteryLoaded) ...[
            const SizedBox(height: 16),
            Center(
              child: Text(
                'Current limit: ${state.chargeLimit}%',
                style: theme.textTheme.bodyMedium?.copyWith(
                  color: colorScheme.onSurfaceVariant,
                ),
              ),
            ),
          ],
        ],
      ),
    );
  }

  Widget _buildPresetButton(String label, double value, ThemeData theme) {
    final isSelected = _chargeLimit.round() == value.round();
    return OutlinedButton(
      onPressed: () {
        setState(() {
          _chargeLimit = value;
        });
      },
      style: OutlinedButton.styleFrom(
        backgroundColor: isSelected ? theme.colorScheme.primaryContainer : null,
      ),
      child: Text(label),
    );
  }
}
