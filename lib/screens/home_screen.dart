import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import '../blocs/cpu_boost/cpu_boost_bloc.dart';
import '../blocs/cpu_boost/cpu_boost_event.dart';
import '../blocs/cpu_boost/cpu_boost_state.dart';
import '../blocs/performance/performance_bloc.dart';
import '../blocs/performance/performance_event.dart';
import '../blocs/performance/performance_state.dart';
import '../widgets/mode_selector.dart';
import '../widgets/copyable_alert.dart';
import '../widgets/cpu_boost/boost_mode_selector.dart';

class HomeScreen extends StatelessWidget {
  final PerformanceBloc? testBloc;
  final CpuBoostBloc? testCpuBoostBloc;

  const HomeScreen({super.key, this.testBloc, this.testCpuBoostBloc});

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: const Text('ASUS Linux Control')),
      body: MultiBlocProvider(
        providers: [
          BlocProvider(
            create: (context) =>
                testBloc ?? (PerformanceBloc()..add(LoadCurrentModeEvent())),
          ),
          BlocProvider(
            create: (context) =>
                testCpuBoostBloc ?? (CpuBoostBloc()..add(LoadCpuBoostStatus())),
          ),
        ],
        child: MultiBlocListener(
          listeners: [
            BlocListener<PerformanceBloc, PerformanceState>(
              listener: (context, state) {
                if (state is PerformanceError) {
                  // Показываем алерт с копированием при ошибке
                  CopyableAlert.showError(
                    context,
                    title: 'Ошибка производительности',
                    message:
                        'Не удалось выполнить операцию с режимом производительности',
                    errorDetails: state.message,
                  );
                }
              },
            ),
            BlocListener<CpuBoostBloc, CpuBoostState>(
              listener: (context, state) {
                if (state is CpuBoostError) {
                  CopyableAlert.showError(
                    context,
                    title: 'Ошибка CPU Boost',
                    message: 'Не удалось выполнить операцию с CPU Boost',
                    errorDetails: state.message,
                  );
                }
              },
            ),
          ],
          child: BlocBuilder<PerformanceBloc, PerformanceState>(
            builder: (context, state) {
              return SingleChildScrollView(
                padding: const EdgeInsets.all(16.0),
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Text(
                      'ASUS Linux Control Tool',
                      style: Theme.of(context).textTheme.headlineMedium,
                    ),
                    const SizedBox(height: 8),
                    Text(
                      'MVP: Performance Mode Control',
                      style: Theme.of(context).textTheme.titleMedium,
                    ),
                    const SizedBox(height: 24),

                    // Current Status Card
                    Card(
                      child: Padding(
                        padding: const EdgeInsets.all(16.0),
                        child: Row(
                          children: [
                            const Icon(Icons.info_outline, size: 32),
                            const SizedBox(width: 16),
                            Expanded(
                              child: Column(
                                crossAxisAlignment: CrossAxisAlignment.start,
                                children: [
                                  Text(
                                    'Current Mode',
                                    style: Theme.of(
                                      context,
                                    ).textTheme.titleSmall,
                                  ),
                                  const SizedBox(height: 4),
                                  if (state is PerformanceLoading)
                                    const Text('Loading...')
                                  else if (state is PerformanceLoaded)
                                    Text(
                                      state.modeName,
                                      style: Theme.of(context)
                                          .textTheme
                                          .headlineSmall
                                          ?.copyWith(
                                            color: Theme.of(
                                              context,
                                            ).colorScheme.primary,
                                          ),
                                    )
                                  else if (state is PerformanceError)
                                    Text(
                                      'Error',
                                      style: TextStyle(
                                        color: Theme.of(
                                          context,
                                        ).colorScheme.error,
                                      ),
                                    )
                                  else
                                    const Text('Unknown'),
                                ],
                              ),
                            ),
                          ],
                        ),
                      ),
                    ),

                    const SizedBox(height: 24),

                    // Mode Selector
                    ModeSelector(
                      currentMode: state is PerformanceLoaded
                          ? state.currentMode
                          : 1,
                      onModeSelected: (mode) {
                        context.read<PerformanceBloc>().add(
                          SetModeRequestedEvent(mode),
                        );
                      },
                    ),
                    const SizedBox(height: 24),
                    BlocBuilder<CpuBoostBloc, CpuBoostState>(
                      builder: (context, boostState) {
                        return BoostModeSelector(
                          status: boostState is CpuBoostLoaded
                              ? boostState.status
                              : null,
                          isLoading: boostState is CpuBoostLoading,
                          isUnsupported: boostState is CpuBoostUnsupported,
                          onModeSelected: (mode) {
                            context.read<CpuBoostBloc>().add(
                              SetCpuBoostModeRequested(mode),
                            );
                          },
                        );
                      },
                    ),
                  ],
                ),
              );
            },
          ),
        ),
      ),
    );
  }
}
