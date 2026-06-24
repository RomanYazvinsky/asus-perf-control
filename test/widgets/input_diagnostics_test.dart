import 'package:flutter/material.dart';
import 'package:flutter/gestures.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:flutter_application_1/widgets/input_diagnostics.dart';

void main() {
  testWidgets('reports touch pointer down and up events', (tester) async {
    final events = <PointerEvent>[];

    await tester.pumpWidget(
      MaterialApp(
        home: InputDiagnostics(
          enabled: true,
          onEvent: events.add,
          child: const SizedBox.expand(),
        ),
      ),
    );

    final gesture = await tester.startGesture(
      tester.getCenter(find.byType(SizedBox)),
      kind: PointerDeviceKind.touch,
    );
    await gesture.up();

    expect(events, contains(isA<PointerDownEvent>()));
    expect(events, contains(isA<PointerUpEvent>()));
    expect(
      events.every((event) => event.kind == PointerDeviceKind.touch),
      isTrue,
    );
  });
}
