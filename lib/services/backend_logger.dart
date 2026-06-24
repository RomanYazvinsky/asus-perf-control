import 'dart:async';

import 'package:flutter/foundation.dart';

typedef BackendLogSink = void Function(String message);

class BackendLogger {
  static BackendLogSink sink = _defaultSink;
  static bool enabled = true;

  static Future<T> trace<T>(
    String operation,
    FutureOr<T> Function() action, {
    Map<String, Object?> fields = const {},
  }) async {
    if (!enabled) {
      return await Future<T>.sync(action);
    }

    final stopwatch = Stopwatch()..start();
    _write('START', operation, fields);
    try {
      final result = await Future<T>.sync(action);
      stopwatch.stop();
      _write('OK', operation, fields, elapsedMs: stopwatch.elapsedMilliseconds);
      return result;
    } catch (error, stackTrace) {
      stopwatch.stop();
      _write(
        'ERROR',
        operation,
        fields,
        elapsedMs: stopwatch.elapsedMilliseconds,
        error: error,
      );
      Error.throwWithStackTrace(error, stackTrace);
    }
  }

  static T traceSync<T>(
    String operation,
    T Function() action, {
    Map<String, Object?> fields = const {},
  }) {
    if (!enabled) {
      return action();
    }

    final stopwatch = Stopwatch()..start();
    _write('START', operation, fields);
    try {
      final result = action();
      stopwatch.stop();
      _write('OK', operation, fields, elapsedMs: stopwatch.elapsedMilliseconds);
      return result;
    } catch (error, stackTrace) {
      stopwatch.stop();
      _write(
        'ERROR',
        operation,
        fields,
        elapsedMs: stopwatch.elapsedMilliseconds,
        error: error,
      );
      Error.throwWithStackTrace(error, stackTrace);
    }
  }

  static void logEvent(
    String operation, {
    Map<String, Object?> fields = const {},
  }) {
    if (!enabled) return;
    _write('EVENT', operation, fields);
  }

  static void resetForTests() {
    sink = _defaultSink;
    enabled = true;
  }

  static void _write(
    String status,
    String operation,
    Map<String, Object?> fields, {
    int? elapsedMs,
    Object? error,
  }) {
    final parts = <String>['[Backend]', status, operation];
    final formattedFields = _formatFields(fields);
    if (formattedFields.isNotEmpty) {
      parts.add(formattedFields);
    }
    if (elapsedMs != null) {
      parts.add('elapsedMs=$elapsedMs');
    }
    if (error != null) {
      parts.add('error=${error.runtimeType}: $error');
    }
    sink(parts.join(' '));
  }

  static String _formatFields(Map<String, Object?> fields) {
    return fields.entries
        .where((entry) => entry.value != null)
        .map((entry) => '${entry.key}=${entry.value}')
        .join(' ');
  }

  static void _defaultSink(String message) {
    debugPrint(message);
  }
}
