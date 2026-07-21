import 'package:flutter_bloc/flutter_bloc.dart';
import '../../models/gpu_models.dart';
import '../../services/gpu_service.dart';
import '../../services/backend_logger.dart';
import '../../services/settings/app_settings_store.dart';
import 'gpu_event.dart';
import 'gpu_state.dart';

class GpuBloc extends Bloc<GpuEvent, GpuState> {
  final IGpuService _gpuService;
  final AppSettingsStore? _settingsStore;

  GpuBloc({required IGpuService gpuService, AppSettingsStore? settingsStore})
    : _gpuService = gpuService,
      _settingsStore = settingsStore,
      super(GpuInitial()) {
    on<LoadCapabilitiesEvent>(_onLoadCapabilities);
    on<LoadStatusEvent>(_onLoadStatus);
    on<SetModeRequestedEvent>(_onSetModeRequested);
    on<ConfirmRebootEvent>(_onConfirmReboot);
  }

  Future<void> _onLoadCapabilities(
    LoadCapabilitiesEvent event,
    Emitter<GpuState> emit,
  ) async {
    emit(GpuLoading());
    try {
      final capabilities = await _gpuService.getCapabilities();

      if (!capabilities.hasDgpu) {
        emit(
          GpuUnsupported(
            message:
                'GPU hardware not detected. Check that discrete GPU is present.',
          ),
        );
        return;
      }

      final status = await _gpuService.getStatus();
      final supportedModes = await _gpuService.getSupportedModes();
      await _logSavedModeMismatch(status);

      emit(
        GpuLoaded(
          capabilities: capabilities,
          status: status,
          supportedModes: supportedModes,
        ),
      );
    } catch (e) {
      emit(GpuError(message: 'Failed to load GPU capabilities: $e'));
    }
  }

  Future<void> _onLoadStatus(
    LoadStatusEvent event,
    Emitter<GpuState> emit,
  ) async {
    if (state is GpuLoaded) {
      try {
        final status = await _gpuService.getStatus();
        final currentState = state as GpuLoaded;
        emit(
          GpuLoaded(
            capabilities: currentState.capabilities,
            status: status,
            supportedModes: currentState.supportedModes,
          ),
        );
      } catch (e) {
        emit(GpuError(message: 'Failed to load GPU status: $e'));
      }
    }
  }

  Future<void> _onSetModeRequested(
    SetModeRequestedEvent event,
    Emitter<GpuState> emit,
  ) async {
    if (state is! GpuLoaded) {
      emit(GpuError(message: 'GPU not loaded. Please try again.'));
      return;
    }

    final currentState = state as GpuLoaded;

    // Validate that requested mode is supported
    if (!currentState.supportedModes.contains(event.mode)) {
      emit(
        GpuError(
          message:
              'GPU mode "${event.mode.name}" is not supported on this laptop model. '
              'Supported modes: ${currentState.supportedModes.map((m) => m.name).join(", ")}',
        ),
      );
      // Restore previous state
      emit(currentState);
      return;
    }

    try {
      await _gpuService.setGpuMode(event.mode);
      await _settingsStore?.update((settings) {
        return settings.copyWith(
          hardware: settings.hardware.copyWith(gpuMode: event.mode.name),
        );
      });

      // Reload status after mode change
      final status = await _gpuService.getStatus();
      emit(
        GpuLoaded(
          capabilities: currentState.capabilities,
          status: status,
          supportedModes: currentState.supportedModes,
        ),
      );
    } catch (e) {
      final error = e.toString();

      // Check for reboot requirement (MUX switch)
      if (error.contains('reboot') || error.contains('RebootRequired')) {
        emit(
          GpuRequiresReboot(
            message:
                'Switching to MUX dGPU mode requires a system reboot. The MUX switch will physically route display output directly to the dGPU for maximum performance.',
          ),
        );
      } else if (error.contains('logout') || error.contains('LogoutRequired')) {
        emit(
          GpuRequiresLogout(
            message:
                'GPU mode changed. Please save your work and log out, then log back in to finish applying the change.',
          ),
        );
      } else if (error.contains('eGPU not connected') ||
          error.contains('EgpuNotConnected')) {
        emit(
          GpuError(
            message:
                'External GPU not connected. Connect XG Mobile before enabling eGPU mode.',
          ),
        );
      } else if (error.contains('VFIO') || error.contains('VfioNotAvailable')) {
        emit(
          GpuError(
            message:
                'VFIO kernel modules not available. Install vfio-pci module for GPU passthrough.',
          ),
        );
      } else {
        emit(GpuError(message: 'Failed to set GPU mode: $e'));
      }
    }
  }

  Future<void> _logSavedModeMismatch(GpuStatus status) async {
    final settings = await _settingsStore?.load();
    final desiredName = settings?.hardware.gpuMode;
    if (desiredName == null) {
      await _settingsStore?.update((settings) {
        return settings.copyWith(
          hardware: settings.hardware.copyWith(gpuMode: status.activeMode.name),
        );
      });
      return;
    }
    if (desiredName == status.activeMode.name) return;
    BackendLogger.logEvent(
      'Settings.hardwareMismatch',
      fields: {
        'key': 'gpuMode',
        'expected': desiredName,
        'actual': status.activeMode.name,
      },
    );
  }

  Future<void> _onConfirmReboot(
    ConfirmRebootEvent event,
    Emitter<GpuState> emit,
  ) async {
    try {
      await _gpuService.confirmMuxAndReboot();
    } catch (e) {
      emit(GpuError(message: 'Failed to confirm MUX reboot: $e'));
    }
  }
}
