import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import '../blocs/gpu/gpu_bloc.dart';
import '../blocs/gpu/gpu_event.dart';
import '../blocs/gpu/gpu_state.dart';
import '../services/gpu_service.dart';
import '../services/settings/app_settings_store.dart';
import '../widgets/copyable_alert.dart';
import '../widgets/gpu/gpu_status_card.dart';
import '../widgets/gpu/gpu_mode_selector.dart';

class GpuScreen extends StatelessWidget {
  final AppSettingsStore? settingsStore;

  const GpuScreen({super.key, this.settingsStore});

  @override
  Widget build(BuildContext context) {
    return BlocProvider(
      create: (context) => GpuBloc(
        gpuService: SupergfxGpuService(),
        settingsStore: settingsStore,
      )..add(LoadCapabilitiesEvent()),
      child: const GpuView(),
    );
  }
}

class GpuView extends StatelessWidget {
  const GpuView({super.key});

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: const Text('GPU Modes')),
      body: BlocConsumer<GpuBloc, GpuState>(
        listener: (context, state) {
          if (state is GpuRequiresReboot) {
            _showRebootDialog(context, state.message);
          } else if (state is GpuRequiresLogout) {
            _showLogoutDialog(context, state.message);
          } else if (state is GpuUnsupported) {
            CopyableAlert.showInfo(
              context,
              title: 'GPU Not Supported',
              message: 'Discrete GPU hardware not detected on this system.',
              details: state.message,
            );
          } else if (state is GpuError) {
            CopyableAlert.showError(
              context,
              title: 'GPU Error',
              message: 'An error occurred while performing a GPU operation.',
              errorDetails: state.message,
            );
          }
        },
        builder: (context, state) {
          if (state is GpuLoading) {
            return const Center(child: CircularProgressIndicator());
          } else if (state is GpuLoaded) {
            return SingleChildScrollView(
              padding: const EdgeInsets.all(16),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  GpuStatusCard(status: state.status),
                  const SizedBox(height: 24),
                  GpuModeSelector(
                    capabilities: state.capabilities,
                    currentMode: state.status.activeMode,
                    supportedModes: state.supportedModes,
                    onModeSelected: (mode) {
                      context.read<GpuBloc>().add(
                        SetModeRequestedEvent(mode: mode),
                      );
                    },
                  ),
                ],
              ),
            );
          } else if (state is GpuUnsupported) {
            return Center(
              child: Padding(
                padding: const EdgeInsets.all(24),
                child: Column(
                  mainAxisAlignment: MainAxisAlignment.center,
                  children: [
                    const Icon(Icons.warning, size: 64, color: Colors.orange),
                    const SizedBox(height: 16),
                    Text(
                      state.message,
                      textAlign: TextAlign.center,
                      style: Theme.of(context).textTheme.bodyLarge,
                    ),
                  ],
                ),
              ),
            );
          } else if (state is GpuError) {
            // Errors are shown via CopyableAlert in listener;
            // show retry button in body as fallback
            return Center(
              child: Column(
                mainAxisAlignment: MainAxisAlignment.center,
                children: [
                  const Icon(Icons.error_outline, size: 48, color: Colors.red),
                  const SizedBox(height: 16),
                  ElevatedButton(
                    onPressed: () {
                      context.read<GpuBloc>().add(LoadCapabilitiesEvent());
                    },
                    child: const Text('Retry'),
                  ),
                ],
              ),
            );
          }
          return const SizedBox.shrink();
        },
      ),
    );
  }

  void _showLogoutDialog(BuildContext context, String message) {
    showDialog(
      context: context,
      builder: (context) => AlertDialog(
        title: const Text('Logout Required'),
        content: Text(message),
        actions: [
          TextButton(
            onPressed: () {
              Navigator.of(context).pop();
            },
            child: const Text('OK'),
          ),
        ],
      ),
    );
  }

  void _showRebootDialog(BuildContext context, String message) {
    showDialog(
      context: context,
      builder: (context) => AlertDialog(
        title: const Text('Reboot Required'),
        content: Text(message),
        actions: [
          TextButton(
            onPressed: () {
              Navigator.of(context).pop();
            },
            child: const Text('OK'),
          ),
        ],
      ),
    );
  }
}
