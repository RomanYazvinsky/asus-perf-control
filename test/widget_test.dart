// This is a basic Flutter widget test.
//
// To perform an interaction with a widget in your test, use the WidgetTester
// utility in the flutter_test package. For example, you can send tap and scroll
// gestures. You can also use WidgetTester to find child widgets in the widget
// tree, read text, and verify that the values of widget properties are correct.

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import 'package:flutter_application_1/main.dart';

void main() {
  testWidgets('App shell launches', (WidgetTester tester) async {
    await tester.pumpWidget(const MaterialApp(home: AppShell()));

    // Avoid pumpAndSettle timeout: app contains continuously animating widgets
    // (e.g. progress indicators in mounted tabs).
    await tester.pump(const Duration(milliseconds: 200));

    expect(find.byType(AppShell), findsOneWidget);
    expect(find.byType(NavigationRail), findsOneWidget);
    expect(find.text('Performance'), findsOneWidget);
  });
}
