import 'package:bloc_test/bloc_test.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mocktail/mocktail.dart';
import 'package:flutter_application_1/blocs/performance/performance_bloc.dart';
import 'package:flutter_application_1/blocs/performance/performance_event.dart';
import 'package:flutter_application_1/blocs/performance/performance_state.dart';

import '../mocks.dart';

void main() {
  group('PerformanceBloc', () {
    late MockPerformanceService mockService;

    setUp(() {
      mockService = MockPerformanceService();
    });

    test('initial state is PerformanceInitial', () {
      when(() => mockService.getPerformanceMode()).thenAnswer((_) async => 1);
      final bloc = PerformanceBloc(service: mockService);
      expect(bloc.state, isA<PerformanceInitial>());
      bloc.close();
    });

    group('LoadCurrentModeEvent', () {
      blocTest<PerformanceBloc, PerformanceState>(
        'emits [Loading, Loaded(1)] when service returns Balanced',
        build: () {
          when(
            () => mockService.getPerformanceMode(),
          ).thenAnswer((_) async => 1);
          return PerformanceBloc(service: mockService);
        },
        act: (bloc) => bloc.add(LoadCurrentModeEvent()),
        expect: () => [
          isA<PerformanceLoading>(),
          isA<PerformanceLoaded>()
              .having((s) => s.currentMode, 'currentMode', 1)
              .having((s) => s.modeName, 'modeName', 'Balanced'),
        ],
      );

      blocTest<PerformanceBloc, PerformanceState>(
        'emits [Loading, Loaded(0)] when service returns Silent',
        build: () {
          when(
            () => mockService.getPerformanceMode(),
          ).thenAnswer((_) async => 0);
          return PerformanceBloc(service: mockService);
        },
        act: (bloc) => bloc.add(LoadCurrentModeEvent()),
        expect: () => [
          isA<PerformanceLoading>(),
          isA<PerformanceLoaded>()
              .having((s) => s.currentMode, 'currentMode', 0)
              .having((s) => s.modeName, 'modeName', 'Silent'),
        ],
      );

      blocTest<PerformanceBloc, PerformanceState>(
        'emits [Loading, Error] when service throws',
        build: () {
          when(
            () => mockService.getPerformanceMode(),
          ).thenThrow(Exception('sysfs read failed'));
          return PerformanceBloc(service: mockService);
        },
        act: (bloc) => bloc.add(LoadCurrentModeEvent()),
        expect: () => [isA<PerformanceLoading>(), isA<PerformanceError>()],
      );
    });

    group('SetModeRequestedEvent', () {
      blocTest<PerformanceBloc, PerformanceState>(
        'calls service.setPerformanceMode(0) and emits Loaded(0)',
        build: () {
          when(
            () => mockService.setPerformanceMode(any()),
          ).thenAnswer((_) async {});
          return PerformanceBloc(service: mockService);
        },
        act: (bloc) => bloc.add(SetModeRequestedEvent(0)),
        expect: () => [
          isA<PerformanceLoading>(),
          isA<PerformanceLoaded>()
              .having((s) => s.currentMode, 'currentMode', 0)
              .having((s) => s.modeName, 'modeName', 'Silent'),
        ],
        verify: (bloc) {
          verify(() => mockService.setPerformanceMode(0)).called(1);
        },
      );

      blocTest<PerformanceBloc, PerformanceState>(
        'calls service.setPerformanceMode(2) and emits Loaded(2)',
        build: () {
          when(
            () => mockService.setPerformanceMode(any()),
          ).thenAnswer((_) async {});
          return PerformanceBloc(service: mockService);
        },
        act: (bloc) => bloc.add(SetModeRequestedEvent(2)),
        expect: () => [
          isA<PerformanceLoading>(),
          isA<PerformanceLoaded>()
              .having((s) => s.currentMode, 'currentMode', 2)
              .having((s) => s.modeName, 'modeName', 'Turbo'),
        ],
        verify: (bloc) {
          verify(() => mockService.setPerformanceMode(2)).called(1);
        },
      );

      blocTest<PerformanceBloc, PerformanceState>(
        'emits [Loading, Error] when service throws on set',
        build: () {
          when(
            () => mockService.setPerformanceMode(any()),
          ).thenThrow(Exception('permission denied'));
          return PerformanceBloc(service: mockService);
        },
        act: (bloc) => bloc.add(SetModeRequestedEvent(0)),
        expect: () => [isA<PerformanceLoading>(), isA<PerformanceError>()],
      );
    });

    group('State transitions', () {
      blocTest<PerformanceBloc, PerformanceState>(
        'can load then set mode',
        build: () {
          when(
            () => mockService.getPerformanceMode(),
          ).thenAnswer((_) async => 1);
          when(
            () => mockService.setPerformanceMode(any()),
          ).thenAnswer((_) async {});
          return PerformanceBloc(service: mockService);
        },
        act: (bloc) async {
          bloc.add(LoadCurrentModeEvent());
          await Future.delayed(const Duration(milliseconds: 50));
          bloc.add(SetModeRequestedEvent(2));
        },
        expect: () => [
          isA<PerformanceLoading>(),
          isA<PerformanceLoaded>().having((s) => s.currentMode, 'mode', 1),
          isA<PerformanceLoading>(),
          isA<PerformanceLoaded>().having((s) => s.currentMode, 'mode', 2),
        ],
      );
    });
  });

  group('PerformanceLoaded', () {
    test('returns correct mode name for mode 0', () {
      expect(PerformanceLoaded(0).modeName, 'Silent');
    });

    test('returns correct mode name for mode 1', () {
      expect(PerformanceLoaded(1).modeName, 'Balanced');
    });

    test('returns correct mode name for mode 2', () {
      expect(PerformanceLoaded(2).modeName, 'Turbo');
    });

    test('returns Unknown for invalid mode', () {
      expect(PerformanceLoaded(99).modeName, 'Unknown');
    });
  });
}
