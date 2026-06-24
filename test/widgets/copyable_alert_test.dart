import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import 'package:flutter_application_1/widgets/copyable_alert.dart';

void main() {
  group('CopyableAlert Tests', () {
    testWidgets('shows error alert with copyable text', (
      WidgetTester tester,
    ) async {
      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: Builder(
              builder: (context) {
                return ElevatedButton(
                  onPressed: () {
                    CopyableAlert.showError(
                      context,
                      title: 'Test Error',
                      message: 'This is a test error message',
                      errorDetails: 'Detailed error information for debugging',
                    );
                  },
                  child: const Text('Show Error'),
                );
              },
            ),
          ),
        ),
      );

      // Tap the button to show the alert
      await tester.tap(find.text('Show Error'));
      await tester.pumpAndSettle();

      // Verify alert elements
      expect(find.text('Test Error'), findsOneWidget);
      expect(find.text('This is a test error message'), findsOneWidget);
      expect(
        find.text('Detailed error information for debugging'),
        findsOneWidget,
      );
      expect(find.byIcon(Icons.copy), findsOneWidget);
      expect(find.byIcon(Icons.error), findsOneWidget);
      expect(find.text('Закрыть'), findsOneWidget);
    });

    testWidgets('shows success alert with copyable text', (
      WidgetTester tester,
    ) async {
      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: Builder(
              builder: (context) {
                return ElevatedButton(
                  onPressed: () {
                    CopyableAlert.showSuccess(
                      context,
                      title: 'Success!',
                      message: 'Operation completed successfully',
                      details: 'Success details and information',
                    );
                  },
                  child: const Text('Show Success'),
                );
              },
            ),
          ),
        ),
      );

      await tester.tap(find.text('Show Success'));
      await tester.pumpAndSettle();

      expect(find.text('Success!'), findsOneWidget);
      expect(find.text('Operation completed successfully'), findsOneWidget);
      expect(find.text('Success details and information'), findsOneWidget);
      expect(find.byIcon(Icons.copy), findsOneWidget);
      expect(find.byIcon(Icons.check_circle), findsOneWidget);
      expect(find.text('OK'), findsOneWidget);
    });

    testWidgets('shows info alert without copyable text', (
      WidgetTester tester,
    ) async {
      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: Builder(
              builder: (context) {
                return ElevatedButton(
                  onPressed: () {
                    CopyableAlert.showInfo(
                      context,
                      title: 'Information',
                      message: 'Just an informational message',
                    );
                  },
                  child: const Text('Show Info'),
                );
              },
            ),
          ),
        ),
      );

      await tester.tap(find.text('Show Info'));
      await tester.pumpAndSettle();

      expect(find.text('Information'), findsOneWidget);
      expect(find.text('Just an informational message'), findsOneWidget);
      expect(find.byIcon(Icons.copy), findsNothing); // No copyable text
      expect(find.byIcon(Icons.info), findsOneWidget);
      expect(find.text('Понятно'), findsOneWidget);
    });

    testWidgets('copy button copies text to clipboard', (
      WidgetTester tester,
    ) async {
      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: Builder(
              builder: (context) {
                return ElevatedButton(
                  onPressed: () {
                    CopyableAlert.showError(
                      context,
                      title: 'Test Error',
                      message: 'Test message',
                      errorDetails: 'Text to be copied',
                    );
                  },
                  child: const Text('Show Error'),
                );
              },
            ),
          ),
        ),
      );

      await tester.tap(find.text('Show Error'));
      await tester.pumpAndSettle();

      // Tap the copy button
      await tester.tap(find.byIcon(Icons.copy));
      await tester.pumpAndSettle();

      // Verify snackbar appears
      expect(find.text('Скопировано в буфер обмена'), findsOneWidget);
    });

    testWidgets('CopyableSnackBar shows with copy button', (
      WidgetTester tester,
    ) async {
      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: Builder(
              builder: (context) {
                return ElevatedButton(
                  onPressed: () {
                    CopyableSnackBar.show(
                      context,
                      message: 'Snackbar message',
                      copyableText: 'Copyable snackbar text',
                    );
                  },
                  child: const Text('Show SnackBar'),
                );
              },
            ),
          ),
        ),
      );

      await tester.tap(find.text('Show SnackBar'));
      await tester.pumpAndSettle();

      expect(find.text('Snackbar message'), findsOneWidget);
      expect(find.byIcon(Icons.copy), findsOneWidget);
      expect(find.text('Закрыть'), findsOneWidget);
    });
  });
}
