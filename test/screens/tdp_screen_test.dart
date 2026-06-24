import 'package:flutter/material.dart';
import 'package:bloc_test/bloc_test.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mocktail/mocktail.dart';

import 'package:flutter_application_1/blocs/tdp/tdp_bloc.dart';
import 'package:flutter_application_1/blocs/tdp/tdp_event.dart';
import 'package:flutter_application_1/blocs/tdp/tdp_state.dart';
import 'package:flutter_application_1/screens/tdp_screen.dart';

class _MockTdpBloc extends MockBloc<TdpEvent, TdpState> implements TdpBloc {}

Widget _buildScreen(TdpBloc bloc) =>
    MaterialApp(home: TdpScreen(testBloc: bloc));

Future<void> _pumpUi(WidgetTester tester) async {
  await tester.pump();
  await tester.pump(const Duration(milliseconds: 300));
}

void main() {
  setUpAll(() {
    registerFallbackValue(LoadTdpConfig());
    registerFallbackValue(ClearPowerLimitsRequested());
    registerFallbackValue(SelectTdpProfile(1));
  });

  group('TdpScreen widget tests', () {
    testWidgets('shows loading indicator while data not yet loaded', (
      tester,
    ) async {
      final bloc = _MockTdpBloc();
      when(() => bloc.state).thenReturn(TdpLoading());
      whenListen<TdpState>(
        bloc,
        const Stream<TdpState>.empty(),
        initialState: TdpLoading(),
      );

      await tester.pumpWidget(_buildScreen(bloc));
      await _pumpUi(tester);

      expect(find.byType(CircularProgressIndicator), findsWidgets);
    });

    testWidgets('shows unsupported card when TdpUnsupported', (tester) async {
      final bloc = _MockTdpBloc();
      when(() => bloc.state).thenReturn(TdpLoading());
      whenListen<TdpState>(
        bloc,
        Stream<TdpState>.fromIterable([TdpUnsupported()]),
        initialState: TdpLoading(),
      );

      await tester.pumpWidget(_buildScreen(bloc));
      await _pumpUi(tester);

      expect(find.textContaining('Not Available'), findsOneWidget);
    });

    testWidgets('shows TdpControls with correct labels when TdpLoaded', (
      tester,
    ) async {
      final bloc = _MockTdpBloc();
      when(() => bloc.state).thenReturn(TdpLoading());
      whenListen<TdpState>(
        bloc,
        Stream<TdpState>.fromIterable([
          TdpLoaded(
            sustainedW: 20,
            boostW: 35,
            tempLimitC: 85,
            minW: 15,
            maxW: 45,
          ),
        ]),
        initialState: TdpLoading(),
      );

      await tester.pumpWidget(_buildScreen(bloc));
      await _pumpUi(tester);

      expect(find.text('Sustained TDP (PL1/SPL)'), findsOneWidget);
      expect(find.text('Boost TDP (PL2/SPPT)'), findsOneWidget);
      expect(find.text('Enable Power Limits'), findsOneWidget);
      expect(find.text('Enable CPU Temperature Limit'), findsOneWidget);
    });

    testWidgets('shows active profile for temperature limit editing', (
      tester,
    ) async {
      final bloc = _MockTdpBloc();
      when(() => bloc.state).thenReturn(TdpLoading());
      whenListen<TdpState>(
        bloc,
        Stream<TdpState>.fromIterable([
          TdpLoaded(
            sustainedW: 20,
            boostW: 35,
            tempLimitC: 75,
            minW: 15,
            maxW: 45,
            activePerformanceMode: 0,
          ),
        ]),
        initialState: TdpLoading(),
      );

      await tester.pumpWidget(_buildScreen(bloc));
      await _pumpUi(tester);

      expect(find.text('Silent'), findsWidgets);
    });

    testWidgets('profile tab dispatches profile switch for restored settings', (
      tester,
    ) async {
      final bloc = _MockTdpBloc();
      when(() => bloc.state).thenReturn(TdpLoading());
      whenListen<TdpState>(
        bloc,
        Stream<TdpState>.fromIterable([
          TdpLoaded(
            sustainedW: 20,
            boostW: 35,
            tempLimitC: 75,
            minW: 15,
            maxW: 45,
            activePerformanceMode: 1,
            powerLimitsEnabled: true,
          ),
        ]),
        initialState: TdpLoading(),
      );

      await tester.pumpWidget(_buildScreen(bloc));
      await _pumpUi(tester);

      expect(find.byType(TabBar), findsOneWidget);
      await tester.tap(find.text('Turbo'));
      await tester.pump();

      final captured = verify(() => bloc.add(captureAny())).captured;
      final event = captured.whereType<SelectTdpProfile>().single;
      expect(event.mode, 2);
    });

    testWidgets('Apply button is active when only power limits are enabled', (
      tester,
    ) async {
      final bloc = _MockTdpBloc();
      when(() => bloc.state).thenReturn(TdpLoading());
      whenListen<TdpState>(
        bloc,
        Stream<TdpState>.fromIterable([
          TdpLoaded(
            sustainedW: 20,
            boostW: 35,
            tempLimitC: 0,
            minW: 15,
            maxW: 45,
            powerLimitsEnabled: true,
          ),
        ]),
        initialState: TdpLoading(),
      );

      await tester.pumpWidget(_buildScreen(bloc));
      await _pumpUi(tester);

      final button = tester.widget<FilledButton>(
        find.byKey(const Key('apply_button')),
      );
      expect(button.onPressed, isNotNull);

      await tester.ensureVisible(find.byKey(const Key('apply_button')));
      await tester.tap(find.byKey(const Key('apply_button')));
      await tester.pump();

      final captured = verify(() => bloc.add(captureAny())).captured;
      final event = captured.whereType<SetTdpRequested>().single;
      expect(event.applyPowerLimits, isTrue);
      expect(event.applyTempLimit, isFalse);
      expect(event.sustainedW, 20);
      expect(event.boostW, 35);
      expect(event.tempLimitC, 90);
    });

    testWidgets(
      'Apply button is active when only temperature limit is enabled',
      (tester) async {
        final bloc = _MockTdpBloc();
        when(() => bloc.state).thenReturn(TdpLoading());
        whenListen<TdpState>(
          bloc,
          Stream<TdpState>.fromIterable([
            TdpLoaded(
              sustainedW: 0,
              boostW: 0,
              tempLimitC: 85,
              minW: 15,
              maxW: 45,
            ),
          ]),
          initialState: TdpLoading(),
        );

        await tester.pumpWidget(_buildScreen(bloc));
        await _pumpUi(tester);

        final button = tester.widget<FilledButton>(
          find.byKey(const Key('apply_button')),
        );
        expect(button.onPressed, isNotNull);

        await tester.ensureVisible(find.byKey(const Key('apply_button')));
        await tester.tap(find.byKey(const Key('apply_button')));
        await tester.pump();

        final captured = verify(() => bloc.add(captureAny())).captured;
        final event = captured.whereType<SetTdpRequested>().single;
        expect(event.applyPowerLimits, isFalse);
        expect(event.applyTempLimit, isTrue);
        expect(event.sustainedW, 15);
        expect(event.boostW, 5);
        expect(event.tempLimitC, 85);
      },
    );

    testWidgets(
      'Apply button is disabled when both power and temperature limits are disabled',
      (tester) async {
        final bloc = _MockTdpBloc();
        when(() => bloc.state).thenReturn(TdpLoading());
        whenListen<TdpState>(
          bloc,
          Stream<TdpState>.fromIterable([
            TdpLoaded(
              sustainedW: 0,
              boostW: 0,
              tempLimitC: 0,
              minW: 15,
              maxW: 45,
            ),
          ]),
          initialState: TdpLoading(),
        );

        await tester.pumpWidget(_buildScreen(bloc));
        await _pumpUi(tester);

        final button = tester.widget<FilledButton>(
          find.byKey(const Key('apply_button')),
        );
        expect(button.onPressed, isNull);
        expect(
          find.text('Enable at least one limit to apply changes'),
          findsOneWidget,
        );
      },
    );

    testWidgets(
      'power sliders are disabled when only temperature limit is enabled',
      (tester) async {
        final bloc = _MockTdpBloc();
        when(() => bloc.state).thenReturn(TdpLoading());
        whenListen<TdpState>(
          bloc,
          Stream<TdpState>.fromIterable([
            TdpLoaded(
              sustainedW: 0,
              boostW: 0,
              tempLimitC: 85,
              minW: 15,
              maxW: 45,
            ),
          ]),
          initialState: TdpLoading(),
        );

        await tester.pumpWidget(_buildScreen(bloc));
        await _pumpUi(tester);

        final sliders = tester.widgetList<Slider>(find.byType(Slider)).toList();
        expect(sliders, hasLength(3));
        expect(sliders[0].onChanged, isNull);
        expect(sliders[1].onChanged, isNull);
        expect(sliders[2].onChanged, isNotNull);
      },
    );

    testWidgets(
      'power stays disabled when loaded state preserves non-zero power values',
      (tester) async {
        final bloc = _MockTdpBloc();
        when(() => bloc.state).thenReturn(TdpLoading());
        whenListen<TdpState>(
          bloc,
          Stream<TdpState>.fromIterable([
            TdpLoaded(
              sustainedW: 20,
              boostW: 35,
              tempLimitC: 85,
              minW: 15,
              maxW: 45,
              powerLimitsEnabled: false,
              tempLimitEnabled: true,
            ),
          ]),
          initialState: TdpLoading(),
        );

        await tester.pumpWidget(_buildScreen(bloc));
        await _pumpUi(tester);

        final sliders = tester.widgetList<Slider>(find.byType(Slider)).toList();
        expect(sliders, hasLength(3));
        expect(sliders[0].onChanged, isNull);
        expect(sliders[1].onChanged, isNull);
        expect(sliders[2].onChanged, isNotNull);

        await tester.ensureVisible(find.byKey(const Key('apply_button')));
        await tester.tap(find.byKey(const Key('apply_button')));
        await tester.pump();

        final captured = verify(() => bloc.add(captureAny())).captured;
        final event = captured.whereType<SetTdpRequested>().single;
        expect(event.applyPowerLimits, isFalse);
        expect(event.applyTempLimit, isTrue);
      },
    );

    testWidgets(
      'temperature slider is disabled when only power limits are enabled',
      (tester) async {
        final bloc = _MockTdpBloc();
        when(() => bloc.state).thenReturn(TdpLoading());
        whenListen<TdpState>(
          bloc,
          Stream<TdpState>.fromIterable([
            TdpLoaded(
              sustainedW: 20,
              boostW: 35,
              tempLimitC: 0,
              minW: 15,
              maxW: 45,
              powerLimitsEnabled: true,
            ),
          ]),
          initialState: TdpLoading(),
        );

        await tester.pumpWidget(_buildScreen(bloc));
        await _pumpUi(tester);

        final sliders = tester.widgetList<Slider>(find.byType(Slider)).toList();
        expect(sliders, hasLength(3));
        expect(sliders[0].onChanged, isNotNull);
        expect(sliders[1].onChanged, isNotNull);
        expect(sliders[2].onChanged, isNull);
      },
    );

    testWidgets(
      'non-zero power values do not enable power switch without explicit flag',
      (tester) async {
        final bloc = _MockTdpBloc();
        when(() => bloc.state).thenReturn(TdpLoading());
        whenListen<TdpState>(
          bloc,
          Stream<TdpState>.fromIterable([
            TdpLoaded(
              sustainedW: 20,
              boostW: 35,
              tempLimitC: 0,
              minW: 15,
              maxW: 45,
              powerLimitsEnabled: false,
            ),
          ]),
          initialState: TdpLoading(),
        );

        await tester.pumpWidget(_buildScreen(bloc));
        await _pumpUi(tester);

        final switches = tester
            .widgetList<Switch>(find.byType(Switch))
            .toList();
        expect(switches.first.value, isFalse);
      },
    );

    testWidgets(
      'turning off enabled power switch dispatches ClearPowerLimitsRequested',
      (tester) async {
        final bloc = _MockTdpBloc();
        when(() => bloc.state).thenReturn(TdpLoading());
        whenListen<TdpState>(
          bloc,
          Stream<TdpState>.fromIterable([
            TdpLoaded(
              sustainedW: 20,
              boostW: 35,
              tempLimitC: 0,
              minW: 15,
              maxW: 45,
              powerLimitsEnabled: true,
            ),
          ]),
          initialState: TdpLoading(),
        );

        await tester.pumpWidget(_buildScreen(bloc));
        await _pumpUi(tester);

        await tester.tap(find.text('Enable Power Limits'));
        await tester.pump();

        final captured = verify(() => bloc.add(captureAny())).captured;
        expect(captured.whereType<ClearPowerLimitsRequested>(), hasLength(1));
      },
    );

    testWidgets('shows loading fallback on TdpError before first load', (
      tester,
    ) async {
      final bloc = _MockTdpBloc();
      when(() => bloc.state).thenReturn(TdpLoading());
      whenListen<TdpState>(
        bloc,
        Stream<TdpState>.fromIterable([TdpError('asusd not running')]),
        initialState: TdpLoading(),
      );

      await tester.pumpWidget(_buildScreen(bloc));
      await _pumpUi(tester);

      expect(find.byType(CircularProgressIndicator), findsWidgets);
    });
  });
}
