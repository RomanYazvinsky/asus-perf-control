import 'package:bloc_test/bloc_test.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mocktail/mocktail.dart';

import 'package:flutter_application_1/blocs/fan/fan_bloc.dart';
import 'package:flutter_application_1/blocs/fan/fan_event.dart';
import 'package:flutter_application_1/blocs/fan/fan_state.dart';
import 'package:flutter_application_1/services/asusd_fan_client.dart';
import 'package:flutter_application_1/services/fan_service.dart';

class MockFanService extends Mock implements IFanService {}

FanEngineConfig _engineConfig() => FanEngineConfig(
  cpuCurve: [FanCurvePoint(tempC: 60, speedPercent: 50)],
  gpuCurve: [FanCurvePoint(tempC: 60, speedPercent: 50)],
);

List<FanCurvePoint> _eightPointCurve({double speedPercent = 50}) {
  return List.generate(
    8,
    (index) =>
        FanCurvePoint(tempC: 30.0 + index * 10.0, speedPercent: speedPercent),
  );
}

FanLoaded _runningFanState() => FanLoaded(
  capabilities: FanCapabilities(
    fans: [
      FanInfo(id: 'cpu', label: 'CPU Fan', hasRpm: true, hasCurve: true),
      FanInfo(id: 'gpu', label: 'GPU Fan', hasRpm: true, hasCurve: true),
    ],
  ),
  fanRpms: const {FanId.cpu: 2100, FanId.gpu: 2200},
  asusdCurves: const {FanId.cpu: {}, FanId.gpu: {}},
  selectedFan: FanId.cpu,
  selectedProfile: PerfProfile.balanced,
  controlModes: const {
    FanId.cpu: FanControlMode.dynamicEngine,
    FanId.gpu: FanControlMode.dynamicEngine,
  },
  dynamicEngineRunning: true,
  engineConfig: _engineConfig(),
);

void main() {
  group('FanBloc dynamic engine state', () {
    late MockFanService mockFanService;

    setUpAll(() {
      registerFallbackValue(_engineConfig());
      registerFallbackValue(PerfProfile.balanced);
      registerFallbackValue(FanId.cpu);
      registerFallbackValue(<FanCurvePoint>[]);
    });

    setUp(() {
      mockFanService = MockFanService();
      when(
        () => mockFanService.startDynamicEngine(any()),
      ).thenAnswer((_) async {});
      when(() => mockFanService.stopDynamicEngine()).thenAnswer((_) async {});
      when(() => mockFanService.getTemperatures()).thenAnswer(
        (_) async => const FanTemperatures(cpuTempC: 45, gpuTempC: 50),
      );
      when(
        () => mockFanService.isDynamicEngineRunning(),
      ).thenAnswer((_) async => false);
      when(
        () => mockFanService.setActiveProfile(any()),
      ).thenAnswer((_) async {});
    });

    blocTest<FanBloc, FanState>(
      'SelectProfile switches active system profile and updates selected profile',
      build: () => FanBloc(mockFanService),
      seed: _runningFanState,
      act: (bloc) => bloc.add(SelectProfile(PerfProfile.performance)),
      expect: () => [
        isA<FanLoaded>().having(
          (state) => state.selectedProfile,
          'selectedProfile',
          PerfProfile.performance,
        ),
      ],
      verify: (_) {
        verify(
          () => mockFanService.setActiveProfile(PerfProfile.performance),
        ).called(1);
      },
    );

    blocTest<FanBloc, FanState>(
      'StartDynamicEngine sets running state and keeps selected profile in config',
      build: () => FanBloc(mockFanService),
      seed: () {
        final base = _runningFanState();
        return base.copyWith(
          selectedProfile: PerfProfile.performance,
          dynamicEngineRunning: false,
          clearEngineConfig: true,
          controlModes: const {
            FanId.cpu: FanControlMode.asusctlCurve,
            FanId.gpu: FanControlMode.asusctlCurve,
          },
        );
      },
      act: (bloc) => bloc.add(
        StartDynamicEngine(
          FanEngineConfig(
            profile: PerfProfile.performance,
            cpuCurve: [FanCurvePoint(tempC: 60, speedPercent: 50)],
            gpuCurve: [FanCurvePoint(tempC: 60, speedPercent: 50)],
          ),
        ),
      ),
      expect: () => [
        isA<FanLoaded>()
            .having(
              (state) => state.dynamicEngineRunning,
              'dynamicEngineRunning',
              true,
            )
            .having(
              (state) => state.engineConfig?.profile,
              'engine profile',
              PerfProfile.performance,
            )
            .having(
              (state) => state.controlModes[FanId.cpu],
              'cpu mode',
              FanControlMode.dynamicEngine,
            )
            .having(
              (state) => state.controlModes[FanId.gpu],
              'gpu mode',
              FanControlMode.dynamicEngine,
            ),
      ],
      verify: (_) {
        final captured =
            verify(
                  () => mockFanService.startDynamicEngine(captureAny()),
                ).captured.single
                as FanEngineConfig;
        expect(captured.profile, PerfProfile.performance);
      },
    );

    blocTest<FanBloc, FanState>(
      'StopDynamicEngine clears engineConfig and sets dynamicEngineRunning=false',
      build: () => FanBloc(mockFanService),
      seed: _runningFanState,
      act: (bloc) => bloc.add(StopDynamicEngine()),
      expect: () => [
        isA<FanLoaded>()
            .having(
              (state) => state.dynamicEngineRunning,
              'dynamicEngineRunning',
              false,
            )
            .having((state) => state.engineConfig, 'engineConfig', isNull)
            .having(
              (state) => state.controlModes[FanId.cpu],
              'cpu mode',
              FanControlMode.asusctlCurve,
            )
            .having(
              (state) => state.controlModes[FanId.gpu],
              'gpu mode',
              FanControlMode.asusctlCurve,
            ),
      ],
      verify: (_) {
        verify(() => mockFanService.stopDynamicEngine()).called(1);
      },
    );

    blocTest<FanBloc, FanState>(
      'UpdateDynamicCurve updates daemon and current engine config',
      build: () {
        when(
          () => mockFanService.updateDynamicCurve(FanId.cpu, any()),
        ).thenAnswer((_) async {});
        return FanBloc(mockFanService);
      },
      seed: _runningFanState,
      act: (bloc) => bloc.add(
        UpdateDynamicCurve(
          fanId: FanId.cpu,
          curve: [FanCurvePoint(tempC: 65, speedPercent: 55)],
        ),
      ),
      expect: () => [
        isA<FanLoaded>().having(
          (state) => state.engineConfig?.cpuCurve.single.speedPercent,
          'updated cpu dynamic curve speed',
          55,
        ),
      ],
      verify: (_) {
        verify(
          () => mockFanService.updateDynamicCurve(FanId.cpu, any()),
        ).called(1);
      },
    );

    blocTest<FanBloc, FanState>(
      'SetAsusctlCurve stores applied curve in state for dynamic engine start',
      build: () {
        when(
          () => mockFanService.setAsusdCurve(any(), any(), any()),
        ).thenAnswer((_) async {});
        return FanBloc(mockFanService);
      },
      seed: () {
        final base = _runningFanState();
        return base.copyWith(
          controlModes: const {
            FanId.cpu: FanControlMode.asusctlCurve,
            FanId.gpu: FanControlMode.asusctlCurve,
          },
          dynamicEngineRunning: false,
          clearEngineConfig: true,
        );
      },
      act: (bloc) => bloc.add(
        SetAsusctlCurve(
          fanId: FanId.cpu,
          profile: PerfProfile.balanced,
          curve: _eightPointCurve(speedPercent: 62),
        ),
      ),
      expect: () => [
        isA<FanLoaded>().having(
          (state) => state.asusdCurves[FanId.cpu]?[PerfProfile.balanced]
              ?.toCurvePoints()
              .first
              .speedPercent,
          'stored cpu curve speed',
          62,
        ),
      ],
    );

    blocTest<FanBloc, FanState>(
      'UpdateDynamicCurve creates visible engine config when dynamic was restored without config',
      build: () {
        when(
          () => mockFanService.updateDynamicCurve(FanId.cpu, any()),
        ).thenAnswer((_) async {});
        return FanBloc(mockFanService);
      },
      seed: () => _runningFanState().copyWith(clearEngineConfig: true),
      act: (bloc) => bloc.add(
        UpdateDynamicCurve(
          fanId: FanId.cpu,
          curve: _eightPointCurve(speedPercent: 64),
        ),
      ),
      expect: () => [
        isA<FanLoaded>().having(
          (state) => state.engineConfig?.cpuCurve.first.speedPercent,
          'visible cpu dynamic curve speed',
          64,
        ),
      ],
      verify: (_) {
        verify(
          () => mockFanService.updateDynamicCurve(FanId.cpu, any()),
        ).called(1);
      },
    );

    blocTest<FanBloc, FanState>(
      'RefreshFanStatus syncs dynamic engine running state from daemon',
      build: () {
        when(() => mockFanService.getFanRpms()).thenAnswer(
          (_) async => [
            FanReading(id: 'cpu', rpm: 2000),
            FanReading(id: 'gpu', rpm: 6500),
          ],
        );
        when(
          () => mockFanService.isDynamicEngineRunning(),
        ).thenAnswer((_) async => false);
        return FanBloc(mockFanService);
      },
      seed: _runningFanState,
      act: (bloc) => bloc.add(RefreshFanStatus()),
      expect: () => [
        isA<FanLoaded>()
            .having(
              (state) => state.dynamicEngineRunning,
              'dynamicEngineRunning',
              false,
            )
            .having(
              (state) => state.controlModes[FanId.gpu],
              'gpu mode',
              FanControlMode.asusctlCurve,
            )
            .having((state) => state.fanRpms[FanId.gpu], 'gpu rpm', 6500),
      ],
    );

    test('close does not stop dynamic engine when running', () async {
      final bloc = FanBloc(mockFanService);
      when(() => mockFanService.getCapabilities()).thenAnswer(
        (_) async => FanCapabilities(
          fans: [
            FanInfo(id: 'cpu', label: 'CPU Fan', hasRpm: true, hasCurve: true),
            FanInfo(id: 'gpu', label: 'GPU Fan', hasRpm: true, hasCurve: true),
          ],
        ),
      );
      when(() => mockFanService.getFanRpms()).thenAnswer(
        (_) async => [
          FanReading(id: 'cpu', rpm: 2000),
          FanReading(id: 'gpu', rpm: 2100),
        ],
      );
      when(
        () => mockFanService.getAsusdCurves(any()),
      ).thenAnswer((_) async => []);

      bloc.add(LoadFanCapabilities());
      await Future<void>.delayed(const Duration(milliseconds: 10));

      bloc.add(
        StartDynamicEngine(
          FanEngineConfig(
            cpuCurve: [FanCurvePoint(tempC: 60, speedPercent: 50)],
            gpuCurve: [FanCurvePoint(tempC: 60, speedPercent: 50)],
          ),
        ),
      );
      await Future<void>.delayed(const Duration(milliseconds: 10));

      await bloc.close();

      verifyNever(() => mockFanService.stopDynamicEngine());
    });

    blocTest<FanBloc, FanState>(
      'LoadFanCapabilities restores dynamic engine state from daemon',
      build: () {
        when(() => mockFanService.getCapabilities()).thenAnswer(
          (_) async => FanCapabilities(
            fans: [
              FanInfo(
                id: 'cpu',
                label: 'CPU Fan',
                hasRpm: true,
                hasCurve: true,
              ),
              FanInfo(
                id: 'gpu',
                label: 'GPU Fan',
                hasRpm: true,
                hasCurve: true,
              ),
            ],
          ),
        );
        when(() => mockFanService.getFanRpms()).thenAnswer(
          (_) async => [
            FanReading(id: 'cpu', rpm: 2000),
            FanReading(id: 'gpu', rpm: 2100),
          ],
        );
        when(
          () => mockFanService.getAsusdCurves(any()),
        ).thenAnswer((_) async => []);
        when(
          () => mockFanService.isDynamicEngineRunning(),
        ).thenAnswer((_) async => true);
        return FanBloc(mockFanService);
      },
      act: (bloc) => bloc.add(LoadFanCapabilities()),
      expect: () => [
        isA<FanLoading>(),
        isA<FanLoaded>()
            .having(
              (state) => state.dynamicEngineRunning,
              'dynamicEngineRunning',
              true,
            )
            .having(
              (state) => state.controlModes[FanId.cpu],
              'cpu mode',
              FanControlMode.dynamicEngine,
            )
            .having(
              (state) => state.controlModes[FanId.gpu],
              'gpu mode',
              FanControlMode.dynamicEngine,
            ),
      ],
    );
  });
}
