import 'package:flutter_test/flutter_test.dart';
import 'package:flutter_application_1/services/backend_logger.dart';

void main() {
  group('BackendLogger', () {
    final logs = <String>[];

    setUp(() {
      logs.clear();
      BackendLogger.sink = logs.add;
    });

    tearDown(() {
      BackendLogger.resetForTests();
    });

    test('logs async backend interaction lifecycle', () async {
      final result = await BackendLogger.trace(
        'AsusdClient.getPerformanceMode',
        () async => 1,
        fields: const {'mode': 'balanced'},
      );

      expect(result, 1);
      expect(logs, hasLength(2));
      expect(
        logs.first,
        contains('[Backend] START AsusdClient.getPerformanceMode'),
      );
      expect(logs.first, contains('mode=balanced'));
      expect(
        logs.last,
        contains('[Backend] OK AsusdClient.getPerformanceMode'),
      );
      expect(logs.last, contains('elapsedMs='));
    });

    test('logs async backend interaction failures and rethrows', () async {
      await expectLater(
        BackendLogger.trace(
          'DbusClient.setTempLimit',
          () async => throw StateError('denied'),
          fields: const {'tempLimitC': 85},
        ),
        throwsStateError,
      );

      expect(logs, hasLength(2));
      expect(logs.first, contains('[Backend] START DbusClient.setTempLimit'));
      expect(logs.last, contains('[Backend] ERROR DbusClient.setTempLimit'));
      expect(logs.last, contains('StateError: Bad state: denied'));
    });

    test('logs sync backend interaction lifecycle', () {
      final result = BackendLogger.traceSync(
        'FanServiceImpl.getFanRpms',
        () => 2,
      );

      expect(result, 2);
      expect(logs.first, contains('[Backend] START FanServiceImpl.getFanRpms'));
      expect(logs.last, contains('[Backend] OK FanServiceImpl.getFanRpms'));
    });
  });
}
