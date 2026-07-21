import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:flutter_application_1/services/input/steam_input_service.dart';
import 'package:flutter_application_1/services/input/ui_input_command.dart';
import 'package:flutter_application_1/widgets/input/gamepad_input_scope.dart';

void main() {
  testWidgets('focus commands move focus between controls', (tester) async {
    final service = FakeSteamInputService();

    await tester.pumpWidget(
      MaterialApp(
        home: GamepadInputScope(
          service: service,
          child: Column(
            children: [
              ElevatedButton(onPressed: () {}, child: const Text('One')),
              ElevatedButton(onPressed: () {}, child: const Text('Two')),
            ],
          ),
        ),
      ),
    );

    FocusScope.of(
      tester.element(find.text('One')),
    ).requestFocus(Focus.of(tester.element(find.text('One'))));
    await tester.pump();

    service.emit(UiInputCommand.focusDown);
    await tester.pump();

    expect(Focus.of(tester.element(find.text('Two'))).hasFocus, isTrue);
  });

  testWidgets('activate command invokes focused button', (tester) async {
    final service = FakeSteamInputService();
    var pressed = 0;

    await tester.pumpWidget(
      MaterialApp(
        home: GamepadInputScope(
          service: service,
          child: ElevatedButton(
            onPressed: () => pressed += 1,
            child: const Text('Apply'),
          ),
        ),
      ),
    );

    FocusScope.of(
      tester.element(find.text('Apply')),
    ).requestFocus(Focus.of(tester.element(find.text('Apply'))));
    await tester.pump();

    service.emit(UiInputCommand.activate);
    await tester.pump();

    expect(pressed, 1);
  });

  testWidgets('back command without dismiss action is a no-op', (tester) async {
    final service = FakeSteamInputService();

    await tester.pumpWidget(
      MaterialApp(
        home: GamepadInputScope(
          service: service,
          child: ElevatedButton(onPressed: () {}, child: const Text('Close')),
        ),
      ),
    );

    FocusScope.of(
      tester.element(find.text('Close')),
    ).requestFocus(Focus.of(tester.element(find.text('Close'))));
    await tester.pump();

    service.emit(UiInputCommand.back);
    await tester.pump();

    expect(tester.takeException(), isNull);
  });

  testWidgets('direction command seeds focus when nothing is focused', (
    tester,
  ) async {
    final service = FakeSteamInputService();

    await tester.pumpWidget(
      MaterialApp(
        home: GamepadInputScope(
          service: service,
          child: Column(
            children: [
              ElevatedButton(onPressed: () {}, child: const Text('One')),
              ElevatedButton(onPressed: () {}, child: const Text('Two')),
            ],
          ),
        ),
      ),
    );

    expect(Focus.of(tester.element(find.text('One'))).hasFocus, isFalse);
    expect(Focus.of(tester.element(find.text('Two'))).hasFocus, isFalse);

    service.emit(UiInputCommand.focusDown);
    await tester.pump();

    final firstButtonFocused = Focus.of(
      tester.element(find.text('One')),
    ).hasFocus;
    final secondButtonFocused = Focus.of(
      tester.element(find.text('Two')),
    ).hasFocus;
    expect(firstButtonFocused || secondButtonFocused, isTrue);
  });

  testWidgets(
    'service replacement starts new service and disposes old service',
    (tester) async {
      final first = _TrackingSteamInputService();
      final second = _TrackingSteamInputService();
      var useSecond = false;

      await tester.pumpWidget(
        StatefulBuilder(
          builder: (context, setState) {
            return MaterialApp(
              home: Column(
                children: [
                  GamepadInputScope(
                    service: useSecond ? second : first,
                    child: const SizedBox.shrink(),
                  ),
                  TextButton(
                    onPressed: () => setState(() => useSecond = true),
                    child: const Text('Swap'),
                  ),
                ],
              ),
            );
          },
        ),
      );

      expect(first.starts, 1);
      expect(first.disposes, 0);
      expect(second.starts, 0);

      await tester.tap(find.text('Swap'));
      await tester.pump();

      expect(first.disposes, 1);
      expect(second.starts, 1);
    },
  );
}

class _TrackingSteamInputService implements SteamInputService {
  final _events = StreamController<UiInputEvent>.broadcast();
  int starts = 0;
  int disposes = 0;

  @override
  Stream<UiInputEvent> get events => _events.stream;

  @override
  Future<SteamInputAvailability> getStatus() async {
    return const SteamInputAvailability(
      available: true,
      connectedControllers: 1,
      activeActionSet: 'ui',
    );
  }

  @override
  void start() {
    starts += 1;
  }

  @override
  void dispose() {
    disposes += 1;
    _events.close();
  }
}
