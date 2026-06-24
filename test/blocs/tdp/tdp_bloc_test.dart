import 'package:bloc_test/bloc_test.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mocktail/mocktail.dart';

import 'package:flutter_application_1/blocs/tdp/tdp_bloc.dart';
import 'package:flutter_application_1/blocs/tdp/tdp_event.dart';
import 'package:flutter_application_1/blocs/tdp/tdp_state.dart';

import '../../mocks.dart';

void main() {
  group('TdpBloc independent limit writes', () {
    late MockTdpService service;

    test('fakeTdpConfig carries explicit power limit enabled flag', () {
      final config = fakeTdpConfig(
        sustainedW: 22,
        boostW: 33,
        powerLimitsEnabled: false,
      );

      expect(config.sustainedW, 22);
      expect(config.boostW, 33);
      expect(config.powerLimitsEnabled, isFalse);
    });

    setUp(() {
      service = MockTdpService();
      when(() => service.getTdpConfig()).thenAnswer(
        (_) async => fakeTdpConfig(
          sustainedW: 22,
          boostW: 33,
          tempLimitC: 0,
          minSustainedW: 15,
          maxSustainedW: 45,
          minBoostW: 20,
          maxBoostW: 60,
        ),
      );
      when(
        () => service.setPowerLimits(
          sustainedW: any(named: 'sustainedW'),
          boostW: any(named: 'boostW'),
        ),
      ).thenAnswer((_) async {});
      when(() => service.setCpuTempLimit(any())).thenAnswer((_) async {});
      when(() => service.clearPowerLimits()).thenAnswer((_) async {});
      when(() => service.setPerformanceMode(any())).thenAnswer((_) async {});
    });

    blocTest<TdpBloc, TdpState>(
      'loads explicit power flag independently from non-zero power values',
      build: () {
        when(() => service.getTdpConfig()).thenAnswer(
          (_) async => fakeTdpConfig(
            sustainedW: 22,
            boostW: 33,
            powerLimitsEnabled: false,
          ),
        );
        return TdpBloc(service: service);
      },
      act: (bloc) => bloc.add(LoadTdpConfig()),
      expect: () => [
        isA<TdpLoading>(),
        isA<TdpLoaded>()
            .having((s) => s.sustainedW, 'sustainedW', 22)
            .having((s) => s.boostW, 'boostW', 33)
            .having((s) => s.powerLimitsEnabled, 'powerLimitsEnabled', false)
            .having((s) => s.tempLimitEnabled, 'tempLimitEnabled', false),
      ],
    );

    blocTest<TdpBloc, TdpState>(
      'loads enabled power flag from config',
      build: () {
        when(() => service.getTdpConfig()).thenAnswer(
          (_) async => fakeTdpConfig(
            sustainedW: 22,
            boostW: 33,
            powerLimitsEnabled: true,
          ),
        );
        return TdpBloc(service: service);
      },
      act: (bloc) => bloc.add(LoadTdpConfig()),
      expect: () => [
        isA<TdpLoading>(),
        isA<TdpLoaded>()
            .having((s) => s.sustainedW, 'sustainedW', 22)
            .having((s) => s.boostW, 'boostW', 33)
            .having((s) => s.powerLimitsEnabled, 'powerLimitsEnabled', true),
      ],
    );

    blocTest<TdpBloc, TdpState>(
      'loads active performance mode from config',
      build: () {
        when(() => service.getTdpConfig()).thenAnswer(
          (_) async => fakeTdpConfig(
            sustainedW: 22,
            boostW: 33,
            tempLimitC: 75,
            activePerformanceMode: 0,
          ),
        );
        return TdpBloc(service: service);
      },
      act: (bloc) => bloc.add(LoadTdpConfig()),
      expect: () => [
        isA<TdpLoading>(),
        isA<TdpLoaded>()
            .having((s) => s.activePerformanceMode, 'activePerformanceMode', 0)
            .having((s) => s.tempLimitC, 'tempLimitC', 75),
      ],
    );

    blocTest<TdpBloc, TdpState>(
      'selecting profile switches system mode and reloads restored settings',
      build: () {
        when(() => service.getTdpConfig()).thenAnswer(
          (_) async => fakeTdpConfig(
            sustainedW: 18,
            boostW: 28,
            tempLimitC: 78,
            activePerformanceMode: 0,
            powerLimitsEnabled: true,
          ),
        );
        return TdpBloc(service: service);
      },
      seed: () => TdpLoaded(
        sustainedW: 30,
        boostW: 45,
        tempLimitC: 90,
        activePerformanceMode: 2,
        powerLimitsEnabled: true,
        tempLimitEnabled: true,
      ),
      act: (bloc) => bloc.add(SelectTdpProfile(0)),
      expect: () => [
        isA<TdpLoading>(),
        isA<TdpLoaded>()
            .having((s) => s.activePerformanceMode, 'activePerformanceMode', 0)
            .having((s) => s.sustainedW, 'sustainedW', 18)
            .having((s) => s.boostW, 'boostW', 28)
            .having((s) => s.tempLimitC, 'tempLimitC', 78)
            .having((s) => s.powerLimitsEnabled, 'powerLimitsEnabled', true),
      ],
      verify: (_) {
        verify(() => service.setPerformanceMode(0)).called(1);
        verify(() => service.getTdpConfig()).called(1);
      },
    );

    blocTest<TdpBloc, TdpState>(
      'clear power limits calls clear path and preserves slider values',
      build: () => TdpBloc(service: service),
      seed: () => TdpLoaded(
        sustainedW: 22,
        boostW: 33,
        tempLimitC: 85,
        minW: 15,
        maxW: 45,
        minBoostW: 20,
        maxBoostW: 60,
        powerLimitsEnabled: true,
        tempLimitEnabled: true,
      ),
      act: (bloc) => bloc.add(ClearPowerLimitsRequested()),
      expect: () => [
        isA<TdpLoading>(),
        isA<TdpLoaded>()
            .having((s) => s.sustainedW, 'sustainedW', 22)
            .having((s) => s.boostW, 'boostW', 33)
            .having((s) => s.tempLimitC, 'tempLimitC', 85)
            .having((s) => s.powerLimitsEnabled, 'powerLimitsEnabled', false)
            .having((s) => s.tempLimitEnabled, 'tempLimitEnabled', true),
      ],
      verify: (_) {
        verify(() => service.clearPowerLimits()).called(1);
      },
    );

    blocTest<TdpBloc, TdpState>(
      'temperature-only apply calls only setCpuTempLimit',
      build: () => TdpBloc(service: service),
      seed: () => TdpLoaded(
        sustainedW: 22,
        boostW: 33,
        tempLimitC: 80,
        minW: 15,
        maxW: 45,
        minBoostW: 20,
        maxBoostW: 60,
      ),
      act: (bloc) => bloc.add(
        SetTdpRequested(
          applyPowerLimits: false,
          applyTempLimit: true,
          sustainedW: 30,
          boostW: 40,
          tempLimitC: 88,
        ),
      ),
      expect: () => [
        isA<TdpLoading>(),
        isA<TdpLoaded>()
            .having((s) => s.sustainedW, 'sustainedW', 22)
            .having((s) => s.boostW, 'boostW', 33)
            .having((s) => s.tempLimitC, 'tempLimitC', 88)
            .having((s) => s.powerLimitsEnabled, 'powerLimitsEnabled', false)
            .having((s) => s.tempLimitEnabled, 'tempLimitEnabled', true)
            .having((s) => s.minW, 'minW', 15)
            .having((s) => s.maxW, 'maxW', 45)
            .having((s) => s.minBoostW, 'minBoostW', 20)
            .having((s) => s.maxBoostW, 'maxBoostW', 60),
      ],
      verify: (_) {
        verify(() => service.setCpuTempLimit(88)).called(1);
        verifyNever(
          () => service.setPowerLimits(
            sustainedW: any(named: 'sustainedW'),
            boostW: any(named: 'boostW'),
          ),
        );
      },
    );

    blocTest<TdpBloc, TdpState>(
      'power-only apply calls only setPowerLimits',
      build: () => TdpBloc(service: service),
      seed: () => TdpLoaded(
        sustainedW: 22,
        boostW: 33,
        tempLimitC: 80,
        minW: 15,
        maxW: 45,
        minBoostW: 20,
        maxBoostW: 60,
      ),
      act: (bloc) => bloc.add(
        SetTdpRequested(
          applyPowerLimits: true,
          applyTempLimit: false,
          sustainedW: 30,
          boostW: 40,
          tempLimitC: 88,
        ),
      ),
      expect: () => [
        isA<TdpLoading>(),
        isA<TdpLoaded>()
            .having((s) => s.sustainedW, 'sustainedW', 30)
            .having((s) => s.boostW, 'boostW', 40)
            .having((s) => s.tempLimitC, 'tempLimitC', 80)
            .having((s) => s.powerLimitsEnabled, 'powerLimitsEnabled', true)
            .having((s) => s.tempLimitEnabled, 'tempLimitEnabled', false),
      ],
      verify: (_) {
        verify(
          () => service.setPowerLimits(sustainedW: 30, boostW: 40),
        ).called(1);
        verifyNever(() => service.setCpuTempLimit(any()));
      },
    );

    blocTest<TdpBloc, TdpState>(
      'combined apply calls both write paths',
      build: () => TdpBloc(service: service),
      seed: () => TdpLoaded(
        sustainedW: 22,
        boostW: 33,
        tempLimitC: 80,
        minW: 15,
        maxW: 45,
        minBoostW: 20,
        maxBoostW: 60,
      ),
      act: (bloc) => bloc.add(
        SetTdpRequested(
          applyPowerLimits: true,
          applyTempLimit: true,
          sustainedW: 31,
          boostW: 41,
          tempLimitC: 89,
        ),
      ),
      expect: () => [
        isA<TdpLoading>(),
        isA<TdpLoaded>()
            .having((s) => s.sustainedW, 'sustainedW', 31)
            .having((s) => s.boostW, 'boostW', 41)
            .having((s) => s.tempLimitC, 'tempLimitC', 89)
            .having((s) => s.powerLimitsEnabled, 'powerLimitsEnabled', true)
            .having((s) => s.tempLimitEnabled, 'tempLimitEnabled', true),
      ],
      verify: (_) {
        verify(
          () => service.setPowerLimits(sustainedW: 31, boostW: 41),
        ).called(1);
        verify(() => service.setCpuTempLimit(89)).called(1);
      },
    );
  });
}
