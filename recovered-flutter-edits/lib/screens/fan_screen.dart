import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import '../blocs/fan/fan_bloc.dart';
import '../blocs/fan/fan_event.dart';
import '../blocs/fan/fan_state.dart';
import '../services/fan_service.dart';
import '../services/asusd_fan_client.dart';
import '../services/settings/app_settings_store.dart';
import '../widgets/fan/fan_curve_editor.dart';
import '../widgets/copyable_alert.dart';
import '../widgets/profile_tab_bar.dart';

class FanScreen extends StatelessWidget {
  final AppSettingsStore? settingsStore;

  const FanScreen({super.key, this.settingsStore});

  @override
  Widget build(BuildContext context) {
    return BlocProvider(
      create: (context) =>
          FanBloc(FanServiceImpl(), settingsStore: settingsStore)
            ..add(LoadFanCapabilities()),
      child: const FanView(),
    );
  }
}

class FanView extends StatelessWidget {
  const FanView({super.key});

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: const Text('Fan Control')),
      body: BlocConsumer<FanBloc, FanState>(
        listener: (context, state) {
          if (state is FanError) {
            CopyableAlert.showError(
              context,
              title: 'Fan Control Error',
              message: 'An error occurred while controlling fans',
              errorDetails: state.message,
            );
          }
        },
        builder: (context, state) {
          if (state is FanLoading) {
            return const Center(child: CircularProgressIndicator());
          }

          if (state is FanUnsupported) {
            return Center(
              child: Column(
                mainAxisAlignment: MainAxisAlignment.center,
                children: [
                  const Icon(Icons.warning, size: 64, color: Colors.orange),
                  const SizedBox(height: 16),
                  Text(
                    state.message,
                    style: Theme.of(context).textTheme.titleMedium,
                    textAlign: TextAlign.center,
                  ),
                ],
              ),
            );
          }

          if (state is FanLoaded) {
            return FanContent(state: state);
          }

          if (state is FanMaxSpeedTesting) {
            return _buildMaxSpeedTestView(context, state);
          }

          if (state is FanMaxSpeedResult) {
            return FanMaxSpeedResultView(state: state);
          }

          return const SizedBox.shrink();
        },
      ),
    );
  }

  Widget _buildMaxSpeedTestView(
    BuildContext context,
    FanMaxSpeedTesting state,
  ) {
    return Center(
      child: Column(
        mainAxisAlignment: MainAxisAlignment.center,
        children: [
          const CircularProgressIndicator(),
          const SizedBox(height: 24),
          Text(
            'Max Speed Test in Progress',
            style: Theme.of(context).textTheme.titleLarge,
          ),
          const SizedBox(height: 16),
          Text('Current RPM: ${state.currentRpm}'),
          Text('Elapsed: ${state.elapsedSeconds}s'),
        ],
      ),
    );
  }
}

class FanContent extends StatelessWidget {
  final FanLoaded state;

  const FanContent({super.key, required this.state});

  Future<void> _toggleDynamicEngine(BuildContext context, bool enabled) async {
    if (enabled) {
      context.read<FanBloc>().add(StartDynamicEngine(_buildEngineConfig()));
    } else {
      context.read<FanBloc>().add(StopDynamicEngine());
    }
  }

  FanEngineConfig _buildEngineConfig() {
    return FanEngineConfig(
      profile: state.selectedProfile,
      cpuCurve: _curveFor(FanId.cpu),
      gpuCurve: _curveFor(FanId.gpu),
    );
  }

  List<FanCurvePoint> _curveFor(FanId fanId) {
    final curves =
        state.asusdCurves[fanId] ?? const <PerfProfile, FanCurveData>{};
    List<FanCurvePoint> points;
    final selectedCurve = curves[state.selectedProfile];
    if (selectedCurve != null) {
      points = selectedCurve.toCurvePoints();
    } else {
      points = List.generate(8, (i) {
        final tempC = 20.0 + i * 10.0;
        final speedPercent = 30.0 + (i / 7.0) * 70.0;
        return FanCurvePoint(tempC: tempC, speedPercent: speedPercent);
      });
    }
    // Safety clamp: temps >= 80C require at least 30% speed
    return points.map((p) {
      if (p.tempC >= 80.0 && p.speedPercent < 30.0) {
        return FanCurvePoint(tempC: p.tempC, speedPercent: 30.0);
      }
      return p;
    }).toList();
  }

  @override
  Widget build(BuildContext context) {
    final fanIds = _visibleFanIds();
    return Column(
      children: [
        // Temperature display
        Padding(
          padding: const EdgeInsets.all(16.0),
          child: Row(
            mainAxisAlignment: MainAxisAlignment.spaceAround,
            children: [
              Column(
                children: [
                  const Text('CPU', style: TextStyle(fontSize: 12)),
                  Text(
                    '${state.cpuTempC}°C',
                    style: const TextStyle(
                      fontSize: 20,
                      fontWeight: FontWeight.bold,
                    ),
                  ),
                ],
              ),
              Column(
                children: [
                  const Text('GPU', style: TextStyle(fontSize: 12)),
                  Text(
                    '${state.gpuTempC}°C',
                    style: const TextStyle(
                      fontSize: 20,
                      fontWeight: FontWeight.bold,
                    ),
                  ),
                ],
              ),
            ],
          ),
        ),
        // Dynamic engine toggle
        Padding(
          padding: const EdgeInsets.symmetric(horizontal: 16.0),
          child: Row(
            mainAxisAlignment: MainAxisAlignment.spaceBetween,
            children: [
              const Text('Dynamic Engine'),
              Switch(
                value: state.dynamicEngineRunning,
                onChanged: (enabled) => _toggleDynamicEngine(context, enabled),
              ),
            ],
          ),
        ),
        ProfileTabBar(
          selectedMode: _modeFromProfile(state.selectedProfile),
          onModeSelected: (mode) {
            context.read<FanBloc>().add(SelectProfile(_profileFromMode(mode)));
          },
        ),
        Expanded(
          child: SingleChildScrollView(
            child: Column(
              children: [
                for (final fanId in fanIds)
                  FanTabContent(fanId: fanId, state: state, scrollable: false),
              ],
            ),
          ),
        ),
      ],
    );
  }

  List<FanId> _visibleFanIds() {
    final detected = <FanId>{};
    for (final fan in state.capabilities.fans) {
      detected.add(fan.id == 'cpu' ? FanId.cpu : FanId.gpu);
    }
    return [
      if (detected.contains(FanId.cpu)) FanId.cpu,
      if (detected.contains(FanId.gpu)) FanId.gpu,
    ];
  }
}

int _modeFromProfile(PerfProfile profile) {
  return switch (profile) {
    PerfProfile.quiet || PerfProfile.lowPower => 0,
    PerfProfile.balanced => 1,
    PerfProfile.performance => 2,
  };
}

PerfProfile _profileFromMode(int mode) {
  return switch (mode) {
    0 => PerfProfile.quiet,
    1 => PerfProfile.balanced,
    2 => PerfProfile.performance,
    _ => PerfProfile.balanced,
  };
}

class FanTabContent extends StatelessWidget {
  final FanId fanId;
  final FanLoaded state;
  final bool scrollable;

  const FanTabContent({
    super.key,
    required this.fanId,
    required this.state,
    this.scrollable = true,
  });

  @override
  Widget build(BuildContext context) {
    final currentMode =
        state.controlModes[fanId] ?? FanControlMode.asusctlCurve;
    final rpm = state.fanRpms[fanId] ?? 0;

    final content = Padding(
      padding: const EdgeInsets.all(16.0),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          // Fan status card
          Card(
            child: Padding(
              padding: const EdgeInsets.all(16.0),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Text(
                    fanId == FanId.cpu ? 'CPU Fan' : 'GPU Fan',
                    style: Theme.of(context).textTheme.titleLarge,
                  ),
                  const SizedBox(height: 8),
                  Row(
                    children: [
                      const Icon(Icons.speed, size: 20),
                      const SizedBox(width: 8),
                      Text(
                        '$rpm RPM',
                        style: Theme.of(context).textTheme.headlineSmall,
                      ),
                    ],
                  ),
                  const SizedBox(height: 8),
                  Text(
                    'Mode: ${_modeToString(currentMode)}',
                    style: Theme.of(context).textTheme.bodyMedium,
                  ),
                ],
              ),
            ),
          ),

          const SizedBox(height: 16),

          // Mode-specific controls
          _buildModeControls(context, currentMode),

          const SizedBox(height: 16),

          // Action buttons
          Wrap(
            spacing: 8,
            children: [
              ElevatedButton.icon(
                onPressed: () {
                  context.read<FanBloc>().add(
                    ResetToDefaults(state.selectedProfile),
                  );
                },
                icon: const Icon(Icons.restore),
                label: const Text('Reset to Default'),
              ),
              ElevatedButton.icon(
                onPressed: () {
                  context.read<FanBloc>().add(StartMaxSpeedTest(fanId));
                },
                icon: const Icon(Icons.speed),
                label: const Text('Test Max Speed'),
              ),
            ],
          ),
        ],
      ),
    );
    if (!scrollable) return content;
    return SingleChildScrollView(child: content);
  }

  List<FanCurvePoint> _getDefaultCurve() {
    // Default 8-point curve: linear ramp from 20°C/30% to 90°C/100%
    return List.generate(8, (i) {
      final tempC = 20.0 + i * 10.0; // 20, 30, 40, 50, 60, 70, 80, 90
      final speedPercent = 30.0 + (i / 7.0) * 70.0; // 30% to 100%
      return FanCurvePoint(tempC: tempC, speedPercent: speedPercent);
    });
  }

  Widget _buildModeControls(BuildContext context, FanControlMode mode) {
    final rpm = state.fanRpms[fanId] ?? 0;
    final fanCurves =
        state.asusdCurves[fanId] ?? const <PerfProfile, FanCurveData>{};
    final currentCurve =
        _dynamicCurveFor(fanId, mode) ??
        fanCurves[state.selectedProfile]?.toCurvePoints() ??
        _getDefaultCurve();

    Widget curveEditor({
      required bool readOnly,
      ValueChanged<List<FanCurvePoint>>? onCurveChanged,
    }) {
      return FanCurveEditor(
        curve: currentCurve,
        currentTemp: null,
        currentSpeed: rpm > 0 ? rpm.toDouble() / 50 * 100 : null,
        readOnly: readOnly,
        onCurveChanged: onCurveChanged,
      );
    }

    switch (mode) {
      case FanControlMode.asusctlCurve:
        return curveEditor(
          readOnly: false,
          onCurveChanged: (curve) {
            context.read<FanBloc>().add(
              SetAsusctlCurve(
                fanId: fanId,
                profile: state.selectedProfile,
                curve: curve,
              ),
            );
          },
        );
      case FanControlMode.constantSpeed:
        return Card(
          child: Padding(
            padding: const EdgeInsets.all(16.0),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text(
                  'Constant Speed',
                  style: Theme.of(context).textTheme.titleMedium,
                ),
                const SizedBox(height: 8),
                const Text(
                  'Constant speed control is not available in this build.',
                ),
              ],
            ),
          ),
        );
      case FanControlMode.dynamicEngine:
        return Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            curveEditor(
              readOnly: false,
              onCurveChanged: (curve) {
                context.read<FanBloc>().add(
                  UpdateDynamicCurve(fanId: fanId, curve: curve),
                );
              },
            ),
            const SizedBox(height: 8),
            Text(
              'Apply sends this curve to the running Dynamic Engine.',
              style: Theme.of(context).textTheme.bodySmall,
            ),
          ],
        );
    }
  }

  List<FanCurvePoint>? _dynamicCurveFor(FanId fanId, FanControlMode mode) {
    final engineConfig = state.engineConfig;
    if (mode != FanControlMode.dynamicEngine || engineConfig == null) {
      return null;
    }
    return fanId == FanId.cpu ? engineConfig.cpuCurve : engineConfig.gpuCurve;
  }

  String _modeToString(FanControlMode mode) {
    switch (mode) {
      case FanControlMode.asusctlCurve:
        return 'Asusctl Curve';
      case FanControlMode.constantSpeed:
        return 'Constant Speed';
      case FanControlMode.dynamicEngine:
        return 'Dynamic Engine';
    }
  }
}

class FanMaxSpeedTestView extends StatelessWidget {
  final FanMaxSpeedTesting state;

  const FanMaxSpeedTestView({super.key, required this.state});

  @override
  Widget build(BuildContext context) {
    return Center(
      child: Column(
        mainAxisAlignment: MainAxisAlignment.center,
        children: [
          const CircularProgressIndicator(),
          const SizedBox(height: 24),
          Text(
            'Testing Max Speed...',
            style: Theme.of(context).textTheme.titleLarge,
          ),
          const SizedBox(height: 16),
          Text(
            '${state.elapsedSeconds}s elapsed',
            style: Theme.of(context).textTheme.bodyLarge,
          ),
          const SizedBox(height: 8),
          Text(
            '${state.currentRpm} RPM',
            style: Theme.of(context).textTheme.headlineSmall,
          ),
        ],
      ),
    );
  }
}

class FanMaxSpeedResultView extends StatelessWidget {
  final FanMaxSpeedResult state;

  const FanMaxSpeedResultView({super.key, required this.state});

  @override
  Widget build(BuildContext context) {
    return Center(
      child: Card(
        child: Padding(
          padding: const EdgeInsets.all(24.0),
          child: Column(
            mainAxisSize: MainAxisSize.min,
            children: [
              const Icon(Icons.check_circle, size: 64, color: Colors.green),
              const SizedBox(height: 16),
              Text(
                'Max Speed Test Complete',
                style: Theme.of(context).textTheme.titleLarge,
              ),
              const SizedBox(height: 16),
              Text(
                '${state.fanId == FanId.cpu ? 'CPU' : 'GPU'} Fan Max RPM',
                style: Theme.of(context).textTheme.bodyLarge,
              ),
              const SizedBox(height: 8),
              Text(
                '${state.maxRpm} RPM',
                style: Theme.of(context).textTheme.headlineMedium,
              ),
              const SizedBox(height: 24),
              ElevatedButton(
                onPressed: () {
                  context.read<FanBloc>().add(LoadFanCapabilities());
                },
                child: const Text('OK'),
              ),
            ],
          ),
        ),
      ),
    );
  }
}
