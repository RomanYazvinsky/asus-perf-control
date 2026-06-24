import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:flutter_application_1/widgets/mode_selector.dart';

void main() {
  group('ModeSelector Widget', () {
    testWidgets('displays all three mode buttons', (WidgetTester tester) async {
      int selectedMode = 1;

      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: ModeSelector(
              currentMode: selectedMode,
              onModeSelected: (_) {},
              isLoading: false,
            ),
          ),
        ),
      );

      // Проверяем что есть заголовок
      expect(find.text('Performance Mode'), findsOneWidget);

      // Проверяем что есть все 3 кнопки режимов
      expect(find.text('Silent'), findsOneWidget);
      expect(find.text('Balanced'), findsOneWidget);
      expect(find.text('Turbo'), findsOneWidget);

      // Проверяем описания
      expect(find.text('Quiet operation, lower performance'), findsOneWidget);
      expect(
        find.text('Optimal balance of performance and noise'),
        findsOneWidget,
      );
      expect(find.text('Maximum performance, higher noise'), findsOneWidget);
    });

    testWidgets('shows current mode as selected', (WidgetTester tester) async {
      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: ModeSelector(
              currentMode: 2, // Turbo
              onModeSelected: (_) {},
              isLoading: false,
            ),
          ),
        ),
      );

      // Ищем иконку check_circle (появляется только у выбранного режима)
      expect(find.byIcon(Icons.check_circle), findsOneWidget);
    });

    testWidgets('calls onModeSelected when button is tapped', (
      WidgetTester tester,
    ) async {
      int? selectedMode;

      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: ModeSelector(
              currentMode: 1,
              onModeSelected: (mode) {
                selectedMode = mode;
              },
              isLoading: false,
            ),
          ),
        ),
      );

      // Нажимаем на кнопку Silent (режим 0)
      await tester.tap(find.text('Silent'));
      await tester.pumpAndSettle();

      expect(selectedMode, 0);
    });

    testWidgets('calls onModeSelected with correct mode for Balanced', (
      WidgetTester tester,
    ) async {
      int? selectedMode;

      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: ModeSelector(
              currentMode: 0,
              onModeSelected: (mode) {
                selectedMode = mode;
              },
              isLoading: false,
            ),
          ),
        ),
      );

      // Нажимаем на Balanced (режим 1)
      await tester.tap(find.text('Balanced'));
      await tester.pumpAndSettle();

      expect(selectedMode, 1);
    });

    testWidgets('calls onModeSelected with correct mode for Turbo', (
      WidgetTester tester,
    ) async {
      int? selectedMode;

      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: ModeSelector(
              currentMode: 1,
              onModeSelected: (mode) {
                selectedMode = mode;
              },
              isLoading: false,
            ),
          ),
        ),
      );

      // Нажимаем на Turbo (режим 2)
      await tester.tap(find.text('Turbo'));
      await tester.pumpAndSettle();

      expect(selectedMode, 2);
    });

    testWidgets('shows loading indicator when isLoading is true', (
      WidgetTester tester,
    ) async {
      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: ModeSelector(
              currentMode: 1,
              onModeSelected: (_) {},
              isLoading: true,
            ),
          ),
        ),
      );

      // Должен быть CircularProgressIndicator
      expect(find.byType(CircularProgressIndicator), findsOneWidget);

      // Кнопки не должны отображаться
      expect(find.text('Silent'), findsNothing);
      expect(find.text('Balanced'), findsNothing);
      expect(find.text('Turbo'), findsNothing);
    });

    testWidgets('disables buttons when isLoading is true', (
      WidgetTester tester,
    ) async {
      int callCount = 0;

      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: ModeSelector(
              currentMode: 1,
              onModeSelected: (_) {
                callCount++;
              },
              isLoading: true,
            ),
          ),
        ),
      );

      // Проверяем что показан индикатор загрузки
      expect(find.byType(CircularProgressIndicator), findsOneWidget);

      // Callback не должен вызываться, т.к. кнопки скрыты
      expect(callCount, 0);
    });

    testWidgets('displays correct icons for each mode', (
      WidgetTester tester,
    ) async {
      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: ModeSelector(
              currentMode: null,
              onModeSelected: (_) {},
              isLoading: false,
            ),
          ),
        ),
      );

      // Проверяем иконки для каждого режима
      expect(find.byIcon(Icons.volume_off), findsOneWidget); // Silent
      expect(find.byIcon(Icons.balance), findsOneWidget); // Balanced
      expect(find.byIcon(Icons.speed), findsOneWidget); // Turbo
    });

    testWidgets('handles null currentMode gracefully', (
      WidgetTester tester,
    ) async {
      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: ModeSelector(
              currentMode: null,
              onModeSelected: (_) {},
              isLoading: false,
            ),
          ),
        ),
      );

      // Виджет должен отображаться без ошибок
      expect(find.text('Performance Mode'), findsOneWidget);

      // Не должно быть check_circle т.к. нет выбранного режима
      expect(find.byIcon(Icons.check_circle), findsNothing);
    });
  });
}
