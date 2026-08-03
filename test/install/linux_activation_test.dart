import 'dart:io';

import 'package:flutter_test/flutter_test.dart';

void main() {
  test('Linux activation raises the existing single-instance window', () {
    final source = File('linux/runner/my_application.cc').readAsStringSync();

    expect(source, contains('gtk_application_get_active_window'));
    expect(source, contains('gtk_window_present'));
    expect(source, contains('G_APPLICATION_DEFAULT_FLAGS'));
    expect(source, isNot(contains('G_APPLICATION_NON_UNIQUE')));

    final activeWindow = source.indexOf('gtk_application_get_active_window');
    final newWindow = source.indexOf('gtk_application_window_new');
    expect(activeWindow, lessThan(newWindow));
  });
}
