import 'package:flutter_test/flutter_test.dart';
import 'package:flutter_application_1/services/input/steam_input_service.dart';
import 'package:flutter_application_1/services/input/ui_input_command.dart';

void main() {
  test('fake service emits queued commands in order', () async {
    final service = FakeSteamInputService();
    final events = <UiInputEvent>[];
    final subscription = service.events.listen(events.add);

    service.emit(UiInputCommand.focusDown);
    service.emit(UiInputCommand.activate);
    await Future<void>.delayed(Duration.zero);

    expect(events.map((event) => event.command), [
      UiInputCommand.focusDown,
      UiInputCommand.activate,
    ]);

    await subscription.cancel();
    service.dispose();
  });

  test('command parser rejects unknown command names', () {
    expect(UiInputCommandX.tryParse('focusUp'), UiInputCommand.focusUp);
    expect(UiInputCommandX.tryParse('notACommand'), isNull);
  });
}
