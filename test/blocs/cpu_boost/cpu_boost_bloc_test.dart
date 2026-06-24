import 'package:flutter_test/flutter_test.dart';
import 'package:mocktail/mocktail.dart';
import 'package:flutter_application_1/blocs/cpu_boost/cpu_boost_bloc.dart';
import 'package:flutter_application_1/blocs/cpu_boost/cpu_boost_event.dart';
import 'package:flutter_application_1/blocs/cpu_boost/cpu_boost_state.dart';
import 'package:flutter_application_1/services/cpu_boost_service.dart';

class MockCpuBoostService extends Mock implements ICpuBoostService {}

void main() {
  test('CpuBoostMode exposes stable daemon ids', () {
    expect(CpuBoostMode.disabled.daemonId, 'disabled');
    expect(CpuBoostMode.efficient.daemonId, 'efficient');
    expect(CpuBoostMode.efficientEnabled.daemonId, 'efficient_enabled');
    expect(CpuBoostMode.enabled.daemonId, 'enabled');
    expect(CpuBoostMode.efficientAggressive.daemonId, 'efficient_aggressive');
    expect(CpuBoostMode.aggressive.daemonId, 'aggressive');
  });

  group('CpuBoostBloc', () {
    late MockCpuBoostService service;

    setUp(() {
      service = MockCpuBoostService();
    });

    test('loads current boost mode', () async {
      when(() => service.getCpuBoostStatus()).thenAnswer(
        (_) async => const CpuBoostStatus(
          isSupported: true,
          currentMode: CpuBoostMode.efficientAggressive,
          boostEnabled: true,
          epp: 'balance_performance',
          availableModes: CpuBoostMode.values,
        ),
      );

      final bloc = CpuBoostBloc(service: service)..add(LoadCpuBoostStatus());

      await expectLater(
        bloc.stream,
        emitsInOrder([
          isA<CpuBoostLoading>(),
          isA<CpuBoostLoaded>().having(
            (s) => s.status.currentMode,
            'mode',
            CpuBoostMode.efficientAggressive,
          ),
        ]),
      );
      await bloc.close();
    });

    test('sets boost disabled and reloads status', () async {
      when(
        () => service.setCpuBoostMode(CpuBoostMode.disabled),
      ).thenAnswer((_) async {});
      when(() => service.getCpuBoostStatus()).thenAnswer(
        (_) async => const CpuBoostStatus(
          isSupported: true,
          currentMode: CpuBoostMode.disabled,
          boostEnabled: false,
          availableModes: CpuBoostMode.values,
        ),
      );

      final bloc = CpuBoostBloc(service: service)
        ..add(SetCpuBoostModeRequested(CpuBoostMode.disabled));

      await expectLater(
        bloc.stream,
        emitsInOrder([
          isA<CpuBoostLoading>(),
          isA<CpuBoostLoaded>().having(
            (s) => s.status.boostEnabled,
            'boostEnabled',
            false,
          ),
        ]),
      );
      await bloc.close();
    });
  });
}
