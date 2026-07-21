// Fan BLoC implementation

import 'dart:async';
import 'package:flutter_bloc/flutter_bloc.dart';
import '../../services/backend_logger.dart';
import '../../services/settings/app_settings.dart';
import '../../services/settings/app_settings_store.dart';
import '../../services/fan_service.dart';
import '../../services/asusd_fan_client.dart';
import 'fan_event.dart';
import 'fan_state.dart';

class FanBloc extends Bloc<FanEvent, FanState> {
  final IFanService _fanService;
  final AppSettingsStore? _settingsStore;
  Timer? _rpmRefreshTimer;

  FanBloc(this._fanService, {AppSettingsStore? settingsStore})
    : _settingsStore = settingsStore,
      super(FanInitial()) {
    on<LoadFanCapabilities>(_onLoadFanCapabilities);
    on<RefreshFanStatus>(_onRefreshFanStatus);
    on<SelectFan>(_onSelectFan);
    on<SelectProfile>(_onSelectProfile);
    on<SetAsusctlCurve>(_onSetAsusctlCurve);
    on<SetConstantSpeed>(_onSetConstantSpeed);
    on<StartDynamicEngine>(_onStartDynamicEngine);
    on<StopDynamicEngine>(_onStopDynamicEngine);
    on<UpdateDynamicCurve>(_onUpdateDynamicCurve);
    on<ResetToDefaults>(_onResetToDefaults);
    on<StartMaxSpeedTest>(_onStartMaxSpeedTest);
  }

  Future<void> _onLoadFanCapabilities(
    LoadFanCapabilities event,
    Emitter<FanState> emit,
  ) async {
    emit(FanLoading());
    try {
      final capabilities = await _fanService.getCapabilities();
      if (capabilities.fans.isEmpty) {
        emit(FanUnsupported('No fans detected'));
        return;
      }

      final rpms = await _fanService.getFanRpms();
      final rpmMap = {for (var r in rpms) _fanIdFromString(r.id): r.rpm ?? 0};
      final temperatures = await _readTemperatures();
      var dynamicEngineRunning = await _readDynamicEngineRunning();
      final settings = await _settingsStore?.load();
      final savedFanSettings = settings?.hardware.fan;

      final asusdCurves = <FanId, Map<PerfProfile, FanCurveData>>{};
      for (var fan in capabilities.fans) {
        final fanId = _fanIdFromString(fan.id);
        asusdCurves[fanId] = <PerfProfile, FanCurveData>{};
      }
      for (var profile in PerfProfile.values) {
        try {
          final profileCurves = await _fanService.getAsusdCurves(profile);
          for (var curve in profileCurves) {
            final fanId = _fanIdFromCurveName(curve.name);
            if (fanId != null && asusdCurves.containsKey(fanId)) {
              asusdCurves[fanId]![profile] = curve;
            }
          }
        } catch (e) {
          // If asusd not available for this profile, keep other profile curves.
        }
      }

      final availableFanIds = capabilities.fans
          .map((fan) => _fanIdFromString(fan.id))
          .toSet();
      final selectedFan =
          _fanIdFromName(savedFanSettings?.selectedFan, availableFanIds) ??
          _fanIdFromString(capabilities.fans.first.id);
      final selectedProfile =
          _profileFromName(savedFanSettings?.selectedProfile) ??
          PerfProfile.balanced;
      final savedControlModes = _controlModesFromSettings(
        savedFanSettings,
        availableFanIds,
      );
      final savedDynamicConfig = _dynamicConfigFromSettings(savedFanSettings);
      if (!dynamicEngineRunning &&
          savedControlModes.values.contains(FanControlMode.dynamicEngine) &&
          savedDynamicConfig != null) {
        _logHardwareMismatch(
          'fan.dynamicEngine',
          expected: 'running',
          actual: 'stopped',
        );
        await _fanService.startDynamicEngine(savedDynamicConfig);
        dynamicEngineRunning = true;
      }
      if (selectedProfile != PerfProfile.balanced) {
        await _fanService.setActiveProfile(selectedProfile);
      }

      final loadedState = FanLoaded(
        capabilities: capabilities,
        fanRpms: rpmMap,
        cpuTempC: temperatures.cpuTempC,
        gpuTempC: temperatures.gpuTempC,
        asusdCurves: asusdCurves,
        selectedFan: selectedFan,
        selectedProfile: selectedProfile,
        controlModes: savedControlModes.isNotEmpty
            ? savedControlModes
            : {
                for (var f in capabilities.fans)
                  _fanIdFromString(f.id): dynamicEngineRunning
                      ? FanControlMode.dynamicEngine
                      : FanControlMode.asusctlCurve,
              },
        dynamicEngineRunning: dynamicEngineRunning,
        engineConfig: savedDynamicConfig,
      );
      if (savedFanSettings == null) {
        await _saveFanSettings(
          loadedState,
          dynamicEngineConfig: loadedState.engineConfig,
        );
      }
      emit(loadedState);

      _startRpmRefresh();
    } catch (e) {
      emit(FanError('Failed to load fan capabilities: $e'));
    }
  }

  Future<void> _onRefreshFanStatus(
    RefreshFanStatus event,
    Emitter<FanState> emit,
  ) async {
    if (state is! FanLoaded) return;

    final currentState = state as FanLoaded;
    try {
      final rpms = await _fanService.getFanRpms();
      final rpmMap = {for (var r in rpms) _fanIdFromString(r.id): r.rpm ?? 0};
      final temperatures = await _readTemperatures();
      final dynamicEngineRunning = await _readDynamicEngineRunning();
      final controlModes = dynamicEngineRunning
          ? currentState.controlModes
          : {
              for (final entry in currentState.controlModes.entries)
                entry.key: entry.value == FanControlMode.dynamicEngine
                    ? FanControlMode.asusctlCurve
                    : entry.value,
            };

      emit(
        currentState.copyWith(
          fanRpms: rpmMap,
          cpuTempC: temperatures.cpuTempC,
          gpuTempC: temperatures.gpuTempC,
          controlModes: controlModes,
          dynamicEngineRunning: dynamicEngineRunning,
          clearEngineConfig: !dynamicEngineRunning,
        ),
      );
    } catch (e) {
      // Don't emit error on refresh failure, just log
    }
  }

  Future<void> _onSelectFan(SelectFan event, Emitter<FanState> emit) async {
    if (state is! FanLoaded) return;

    final currentState = state as FanLoaded;
    await _saveFanSettings(
      currentState.copyWith(selectedFan: event.fanId),
      dynamicEngineConfig: currentState.engineConfig,
    );
    emit(currentState.copyWith(selectedFan: event.fanId));
  }

  Future<void> _onSelectProfile(
    SelectProfile event,
    Emitter<FanState> emit,
  ) async {
    if (state is! FanLoaded) return;

    final currentState = state as FanLoaded;
    try {
      await _fanService.setActiveProfile(event.profile);
      final nextState = currentState.copyWith(selectedProfile: event.profile);
      await _saveFanSettings(
        nextState,
        dynamicEngineConfig: currentState.engineConfig,
      );
      emit(nextState);
    } catch (e) {
      emit(FanError('Failed to switch performance profile: $e'));
    }
  }

  Future<void> _onSetAsusctlCurve(
    SetAsusctlCurve event,
    Emitter<FanState> emit,
  ) async {
    if (state is! FanLoaded) return;

    final currentState = state as FanLoaded;
    try {
      await _fanService.setAsusdCurve(event.profile, event.fanId, event.curve);

      final newControlModes = Map<FanId, FanControlMode>.from(
        currentState.controlModes,
      );
      newControlModes[event.fanId] = FanControlMode.asusctlCurve;

      final nextState = currentState.copyWith(
        controlModes: newControlModes,
        asusdCurves: _asusdCurvesWithCurve(
          currentState,
          event.profile,
          event.fanId,
          event.curve,
        ),
      );
      await _saveFanSettings(
        nextState,
        dynamicEngineConfig: nextState.engineConfig,
      );
      emit(nextState);
    } catch (e) {
      emit(FanError('Failed to set asusd curve: $e'));
    }
  }

  Future<void> _onSetConstantSpeed(
    SetConstantSpeed event,
    Emitter<FanState> emit,
  ) async {
    if (state is! FanLoaded) return;

    final currentState = state as FanLoaded;
    try {
      await _fanService.setConstantSpeed(
        event.profile,
        event.fanId,
        event.speedPercent,
      );

      final newControlModes = Map<FanId, FanControlMode>.from(
        currentState.controlModes,
      );
      newControlModes[event.fanId] = FanControlMode.constantSpeed;

      final nextState = currentState.copyWith(
        controlModes: newControlModes,
        asusdCurves: _asusdCurvesWithCurve(
          currentState,
          event.profile,
          event.fanId,
          List.generate(
            8,
            (index) => FanCurvePoint(
              tempC: 30.0 + index * 10.0,
              speedPercent: event.speedPercent.toDouble(),
            ),
          ),
        ),
      );
      await _saveFanSettings(
        nextState,
        dynamicEngineConfig: nextState.engineConfig,
      );
      emit(nextState);
    } catch (e) {
      emit(FanError('Failed to set constant speed: $e'));
    }
  }

  Future<void> _onStartDynamicEngine(
    StartDynamicEngine event,
    Emitter<FanState> emit,
  ) async {
    if (state is! FanLoaded) return;

    final currentState = state as FanLoaded;
    try {
      await _fanService.startDynamicEngine(event.config);

      final newControlModes = Map<FanId, FanControlMode>.from(
        currentState.controlModes,
      );
      for (var fanId in currentState.capabilities.fans.map(
        (f) => _fanIdFromString(f.id),
      )) {
        newControlModes[fanId] = FanControlMode.dynamicEngine;
      }

      final nextState = currentState.copyWith(
        controlModes: newControlModes,
        dynamicEngineRunning: true,
        engineConfig: event.config,
      );
      await _saveFanSettings(nextState, dynamicEngineConfig: event.config);
      emit(nextState);
    } catch (e) {
      emit(FanError('Failed to start dynamic engine: $e'));
    }
  }

  Future<void> _onStopDynamicEngine(
    StopDynamicEngine event,
    Emitter<FanState> emit,
  ) async {
    if (state is! FanLoaded) return;

    final currentState = state as FanLoaded;
    try {
      await _fanService.stopDynamicEngine();

      final newControlModes = Map<FanId, FanControlMode>.from(
        currentState.controlModes,
      );
      for (var fanId in currentState.capabilities.fans.map(
        (f) => _fanIdFromString(f.id),
      )) {
        newControlModes[fanId] = FanControlMode.asusctlCurve;
      }

      final nextState = currentState.copyWith(
        controlModes: newControlModes,
        dynamicEngineRunning: false,
        clearEngineConfig: true,
      );
      await _saveFanSettings(nextState, dynamicEngineConfig: null);
      emit(nextState);
    } catch (e) {
      emit(FanError('Failed to stop dynamic engine: $e'));
    }
  }

  Future<void> _onUpdateDynamicCurve(
    UpdateDynamicCurve event,
    Emitter<FanState> emit,
  ) async {
    if (state is! FanLoaded) return;

    final currentState = state as FanLoaded;
    try {
      await _fanService.updateDynamicCurve(event.fanId, event.curve);
      final currentConfig =
          currentState.engineConfig ?? _engineConfigFromState(currentState);

      final nextConfig = _engineConfigWithCurve(
        currentConfig,
        event.fanId,
        event.curve,
      );
      final nextState = currentState.copyWith(engineConfig: nextConfig);
      await _saveFanSettings(nextState, dynamicEngineConfig: nextConfig);
      emit(nextState);
    } catch (e) {
      emit(FanError('Failed to update dynamic curve: $e'));
    }
  }

  FanEngineConfig _engineConfigWithCurve(
    FanEngineConfig config,
    FanId fanId,
    List<FanCurvePoint> curve,
  ) {
    return FanEngineConfig(
      profile: config.profile,
      tickMs: config.tickMs,
      hysteresisC: config.hysteresisC,
      rampDownDelayMs: config.rampDownDelayMs,
      emergencyTempC: config.emergencyTempC,
      cpuCurve: fanId == FanId.cpu ? curve : config.cpuCurve,
      gpuCurve: fanId == FanId.gpu ? curve : config.gpuCurve,
      crossFan: config.crossFan,
      crossFanCurve: config.crossFanCurve,
    );
  }

  FanProfileCurves _asusdCurvesWithCurve(
    FanLoaded state,
    PerfProfile profile,
    FanId fanId,
    List<FanCurvePoint> curve,
  ) {
    final nextCurves = <FanId, Map<PerfProfile, FanCurveData>>{
      for (final entry in state.asusdCurves.entries)
        entry.key: Map<PerfProfile, FanCurveData>.from(entry.value),
    };

    final fanName = fanId == FanId.cpu ? 'CPU' : 'GPU';
    nextCurves.putIfAbsent(fanId, () => <PerfProfile, FanCurveData>{});
    nextCurves[fanId]![profile] = FanCurveData.fromCurvePoints(
      name: fanName,
      points: curve,
      enabled: true,
    );
    return nextCurves;
  }

  FanEngineConfig _engineConfigFromState(FanLoaded state) {
    return FanEngineConfig(
      profile: state.selectedProfile,
      cpuCurve: _curveFromState(state, FanId.cpu),
      gpuCurve: _curveFromState(state, FanId.gpu),
    );
  }

  List<FanCurvePoint> _curveFromState(FanLoaded state, FanId fanId) {
    final selectedCurve = state.asusdCurves[fanId]?[state.selectedProfile]
        ?.toCurvePoints();
    final points = selectedCurve ?? _defaultCurve();
    return points.map(_withSafetyClamp).toList();
  }

  List<FanCurvePoint> _defaultCurve() {
    return List.generate(8, (index) {
      return FanCurvePoint(
        tempC: 20.0 + index * 10.0,
        speedPercent: 30.0 + (index / 7.0) * 70.0,
      );
    });
  }

  FanCurvePoint _withSafetyClamp(FanCurvePoint point) {
    if (point.tempC >= 80.0 && point.speedPercent < 30.0) {
      return FanCurvePoint(tempC: point.tempC, speedPercent: 30.0);
    }
    return point;
  }

  Future<void> _onResetToDefaults(
    ResetToDefaults event,
    Emitter<FanState> emit,
  ) async {
    if (state is! FanLoaded) return;

    final currentState = state as FanLoaded;
    try {
      await _fanService.resetToDefaults(event.profile);

      // Reload curves for the reset profile
      final newAsusdCurves = <FanId, Map<PerfProfile, FanCurveData>>{
        for (final entry in currentState.asusdCurves.entries)
          entry.key: Map<PerfProfile, FanCurveData>.from(entry.value),
      };
      try {
        final profileCurves = await _fanService.getAsusdCurves(event.profile);
        for (var curve in profileCurves) {
          final fanId = _fanIdFromCurveName(curve.name);
          if (fanId != null) {
            newAsusdCurves.putIfAbsent(
              fanId,
              () => <PerfProfile, FanCurveData>{},
            );
            newAsusdCurves[fanId]![event.profile] = curve;
          }
        }
      } catch (e) {
        // If reload fails, keep existing curves.
      }

      final newControlModes = Map<FanId, FanControlMode>.from(
        currentState.controlModes,
      );
      for (var fanId in currentState.capabilities.fans.map(
        (f) => _fanIdFromString(f.id),
      )) {
        newControlModes[fanId] = FanControlMode.asusctlCurve;
      }

      final nextState = currentState.copyWith(
        controlModes: newControlModes,
        asusdCurves: newAsusdCurves,
      );
      await _saveFanSettings(
        nextState,
        dynamicEngineConfig: nextState.engineConfig,
      );
      emit(nextState);
    } catch (e) {
      emit(FanError('Failed to reset to defaults: $e'));
    }
  }

  Future<void> _onStartMaxSpeedTest(
    StartMaxSpeedTest event,
    Emitter<FanState> emit,
  ) async {
    if (state is! FanLoaded) return;

    final currentState = state as FanLoaded;
    try {
      emit(
        FanMaxSpeedTesting(
          fanId: event.fanId,
          currentRpm: 0,
          elapsedSeconds: 0,
        ),
      );

      final maxRpm = await _fanService.runMaxSpeedTest(
        currentState.selectedProfile,
        event.fanId,
      );

      emit(
        FanMaxSpeedResult(
          previous: currentState,
          fanId: event.fanId,
          maxRpm: maxRpm,
        ),
      );

      // Auto-restore to previous state after 3 seconds
      Future.delayed(const Duration(seconds: 3), () {
        if (state is FanMaxSpeedResult) {
          emit((state as FanMaxSpeedResult).previous);
        }
      });
    } catch (e) {
      emit(FanError('Max speed test failed: $e'));
      // Restore to previous state
      emit(currentState);
    }
  }

  void _startRpmRefresh() {
    _rpmRefreshTimer?.cancel();
    _rpmRefreshTimer = Timer.periodic(const Duration(seconds: 2), (timer) {
      add(RefreshFanStatus());
    });
  }

  @override
  Future<void> close() async {
    _rpmRefreshTimer?.cancel();
    if (_fanService is FanServiceImpl) {
      await _fanService.close();
    }
    return super.close();
  }

  FanId _fanIdFromString(String id) {
    return id == 'cpu' ? FanId.cpu : FanId.gpu;
  }

  FanId? _fanIdFromName(String? name, Set<FanId> availableFanIds) {
    if (name == null) return null;
    for (final fanId in availableFanIds) {
      if (fanId.name == name) return fanId;
    }
    _logReadMismatch('hardware.fan.selectedFan', actual: name);
    return null;
  }

  PerfProfile? _profileFromName(String? name) {
    if (name == null) return null;
    for (final profile in PerfProfile.values) {
      if (profile.name == name) return profile;
    }
    _logReadMismatch('hardware.fan.selectedProfile', actual: name);
    return null;
  }

  Map<FanId, FanControlMode> _controlModesFromSettings(
    FanDesiredSettings? settings,
    Set<FanId> availableFanIds,
  ) {
    if (settings == null || settings.controlModes.isEmpty) return const {};

    final modes = <FanId, FanControlMode>{};
    for (final entry in settings.controlModes.entries) {
      final fanId = _fanIdFromName(entry.key, availableFanIds);
      final mode = _controlModeFromName(entry.value);
      if (fanId != null && mode != null) {
        modes[fanId] = mode;
      }
    }
    return modes;
  }

  FanControlMode? _controlModeFromName(String name) {
    for (final mode in FanControlMode.values) {
      if (mode.name == name) return mode;
    }
    _logReadMismatch('hardware.fan.controlModes', actual: name);
    return null;
  }

  FanEngineConfig? _dynamicConfigFromSettings(FanDesiredSettings? settings) {
    final json = settings?.dynamicEngineConfig;
    if (json == null) return null;
    try {
      return FanEngineConfig.fromJson(json);
    } catch (e) {
      _logReadMismatch('hardware.fan.dynamicEngineConfig', actual: e);
      return null;
    }
  }

  Future<void> _saveFanSettings(
    FanLoaded state, {
    required FanEngineConfig? dynamicEngineConfig,
  }) async {
    await _settingsStore?.update((settings) {
      return settings.copyWith(
        hardware: settings.hardware.copyWith(
          fan: FanDesiredSettings(
            selectedProfile: state.selectedProfile.name,
            selectedFan: state.selectedFan.name,
            controlModes: {
              for (final entry in state.controlModes.entries)
                entry.key.name: entry.value.name,
            },
            dynamicEngineConfig: dynamicEngineConfig?.toJson(),
          ),
        ),
      );
    });
  }

  void _logReadMismatch(String key, {required Object actual}) {
    BackendLogger.logEvent(
      'Settings.readMismatch',
      fields: {'key': key, 'actual': actual},
    );
  }

  void _logHardwareMismatch(
    String key, {
    required Object expected,
    required Object actual,
  }) {
    BackendLogger.logEvent(
      'Settings.hardwareMismatch',
      fields: {'key': key, 'expected': expected, 'actual': actual},
    );
  }

  Future<FanTemperatures> _readTemperatures() async {
    try {
      return await _fanService.getTemperatures();
    } catch (_) {
      return const FanTemperatures(cpuTempC: 0, gpuTempC: 0);
    }
  }

  Future<bool> _readDynamicEngineRunning() async {
    try {
      return await _fanService.isDynamicEngineRunning();
    } catch (_) {
      return false;
    }
  }

  FanId? _fanIdFromCurveName(String name) {
    final normalized = name.toLowerCase();
    if (normalized == 'cpu') return FanId.cpu;
    if (normalized == 'gpu') return FanId.gpu;
    return null;
  }
}
