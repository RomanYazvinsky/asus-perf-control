import 'package:flutter_test/flutter_test.dart';
import 'package:mocktail/mocktail.dart';
import 'package:flutter_application_1/services/fan/fan_models.dart';
import 'package:flutter_application_1/services/fan/fan_engine_daemon_client.dart';
import 'package:flutter_application_1/services/fan/fan_service_impl.dart';
import 'package:flutter_application_1/services/asusd_fan_client.dart';
import 'package:dbus/dbus.dart';

class MockDBusClient extends Mock implements DBusClient {}

class MockDBusRemoteObject extends Mock implements DBusRemoteObject {}

class MockFanEngineDaemonClient extends Mock implements FanEngineDaemonClient {}

class MockAsusdFanClient extends Mock implements AsusdFanClient {}

void main() {
  group('FanEngineConfig JSON', () {
    test('keeps rounded curve temperatures strictly increasing', () {
      final curve = [
        FanCurvePoint(tempC: 30.0, speedPercent: 20),
        FanCurvePoint(tempC: 40.0, speedPercent: 30),
        FanCurvePoint(tempC: 50.0, speedPercent: 40),
        FanCurvePoint(tempC: 60.0, speedPercent: 50),
        FanCurvePoint(tempC: 65.0, speedPercent: 60),
        FanCurvePoint(tempC: 72.6, speedPercent: 70),
        FanCurvePoint(tempC: 72.8, speedPercent: 80),
        FanCurvePoint(tempC: 90.0, speedPercent: 100),
      ];
      final config = FanEngineConfig(cpuCurve: curve, gpuCurve: curve);

      final json = config.toJson();
      final cpuCurve = (json['cpuCurve'] as List).cast<List<dynamic>>();
      final temps = cpuCurve.map((point) => point[0] as int).toList();

      for (var i = 1; i < temps.length; i++) {
        expect(temps[i], greaterThan(temps[i - 1]));
      }
    });
  });

  group('FanServiceImpl dynamic engine lifecycle', () {
    test(
      'close does not stop daemon engine when it was never started',
      () async {
        final fanEngineClient = MockFanEngineDaemonClient();
        final asusdClient = MockAsusdFanClient();
        final service = FanServiceImpl(
          asusdClient: asusdClient,
          fanEngineClient: fanEngineClient,
        );

        when(() => fanEngineClient.stopFanEngine()).thenAnswer((_) async {});
        when(() => fanEngineClient.close()).thenAnswer((_) async {});
        when(() => asusdClient.close()).thenAnswer((_) async {});

        await service.close();

        verifyNever(() => fanEngineClient.stopFanEngine());
        verify(() => asusdClient.close()).called(1);
        verify(() => fanEngineClient.close()).called(1);
      },
    );

    test(
      'stopDynamicEngine stops daemon after running state was restored',
      () async {
        final fanEngineClient = MockFanEngineDaemonClient();
        final asusdClient = MockAsusdFanClient();
        final service = FanServiceImpl(
          asusdClient: asusdClient,
          fanEngineClient: fanEngineClient,
        );

        when(() => fanEngineClient.getRunning()).thenAnswer((_) async => true);
        when(() => fanEngineClient.stopFanEngine()).thenAnswer((_) async {});

        final running = await service.isDynamicEngineRunning();
        await service.stopDynamicEngine();

        expect(running, isTrue);
        verify(() => fanEngineClient.getRunning()).called(1);
        verify(() => fanEngineClient.stopFanEngine()).called(1);
      },
    );

    test(
      'close does not stop daemon engine when it was only observed running',
      () async {
        final fanEngineClient = MockFanEngineDaemonClient();
        final asusdClient = MockAsusdFanClient();
        final service = FanServiceImpl(
          asusdClient: asusdClient,
          fanEngineClient: fanEngineClient,
        );

        when(() => fanEngineClient.getRunning()).thenAnswer((_) async => true);
        when(() => fanEngineClient.stopFanEngine()).thenAnswer((_) async {});
        when(() => fanEngineClient.close()).thenAnswer((_) async {});
        when(() => asusdClient.close()).thenAnswer((_) async {});

        final running = await service.isDynamicEngineRunning();
        await service.close();

        expect(running, isTrue);
        verify(() => fanEngineClient.getRunning()).called(1);
        verifyNever(() => fanEngineClient.stopFanEngine());
        verify(() => asusdClient.close()).called(1);
        verify(() => fanEngineClient.close()).called(1);
      },
    );

    test(
      'close does not stop daemon engine after it was started',
      () async {
        final fanEngineClient = MockFanEngineDaemonClient();
        final asusdClient = MockAsusdFanClient();
        final service = FanServiceImpl(
          asusdClient: asusdClient,
          fanEngineClient: fanEngineClient,
        );
        final config = FanEngineConfig(
          cpuCurve: [FanCurvePoint(tempC: 60, speedPercent: 50)],
          gpuCurve: [FanCurvePoint(tempC: 60, speedPercent: 50)],
        );

        when(() => fanEngineClient.startFanEngine(config)).thenAnswer(
          (_) async {},
        );
        when(() => fanEngineClient.stopFanEngine()).thenAnswer((_) async {});
        when(() => fanEngineClient.close()).thenAnswer((_) async {});
        when(() => asusdClient.close()).thenAnswer((_) async {});

        await service.startDynamicEngine(config);
        await service.close();

        verify(() => fanEngineClient.startFanEngine(config)).called(1);
        verifyNever(() => fanEngineClient.stopFanEngine());
        verify(() => asusdClient.close()).called(1);
        verify(() => fanEngineClient.close()).called(1);
      },
    );
  });

  group('FanEngineDaemonClient', () {
    late FanEngineDaemonClient client;
    late MockDBusClient mockDBus;

    setUpAll(() {
      registerFallbackValue(DBusObjectPath('/com/asuslinuxcontrol/Daemon'));
    });

    setUp(() {
      mockDBus = MockDBusClient();
      client = FanEngineDaemonClient.withClient(mockDBus);
    });

    tearDown(() async {
      // Client close() calls _client.close() which is mocked
      // No need to explicitly close as we're using mocked client
    });

    group('startFanEngine', () {
      test('delegates to daemon StartFanEngine with config JSON', () async {
        final config = FanEngineConfig(
          cpuCurve: [
            FanCurvePoint(tempC: 40, speedPercent: 30),
            FanCurvePoint(tempC: 80, speedPercent: 100),
          ],
          gpuCurve: [
            FanCurvePoint(tempC: 40, speedPercent: 30),
            FanCurvePoint(tempC: 80, speedPercent: 100),
          ],
        );

        when(
          () => mockDBus.callMethod(
            destination: any(named: 'destination'),
            path: any(named: 'path'),
            interface: any(named: 'interface'),
            name: 'StartFanEngine',
            values: any(named: 'values'),
            noAutoStart: any(named: 'noAutoStart'),
            noReplyExpected: any(named: 'noReplyExpected'),
            allowInteractiveAuthorization: any(
              named: 'allowInteractiveAuthorization',
            ),
            replySignature: any(named: 'replySignature'),
          ),
        ).thenAnswer(
          (_) async => DBusMethodSuccessResponse([DBusBoolean(true)]),
        );

        await client.startFanEngine(config);

        verify(
          () => mockDBus.callMethod(
            destination: 'com.asuslinuxcontrol.Daemon',
            path: DBusObjectPath('/com/asuslinuxcontrol/Daemon'),
            interface: 'com.asuslinuxcontrol.Daemon',
            name: 'StartFanEngine',
            values: any(named: 'values'),
            replySignature: DBusSignature('b'),
          ),
        ).called(1);
      });
    });

    group('stopFanEngine', () {
      test('delegates to daemon StopFanEngine', () async {
        when(
          () => mockDBus.callMethod(
            destination: any(named: 'destination'),
            path: any(named: 'path'),
            interface: any(named: 'interface'),
            name: 'StopFanEngine',
            values: any(named: 'values'),
            replySignature: any(named: 'replySignature'),
          ),
        ).thenAnswer(
          (_) async => DBusMethodSuccessResponse([DBusBoolean(true)]),
        );

        await client.stopFanEngine();

        verify(
          () => mockDBus.callMethod(
            destination: 'com.asuslinuxcontrol.Daemon',
            path: DBusObjectPath('/com/asuslinuxcontrol/Daemon'),
            interface: 'com.asuslinuxcontrol.Daemon',
            name: 'StopFanEngine',
            values: [],
            replySignature: DBusSignature('b'),
          ),
        ).called(1);
      });
    });

    group('updateFanCurve', () {
      test(
        'delegates to daemon UpdateFanEngineCurve with fanId and curve JSON',
        () async {
          final curve = [
            FanCurvePoint(tempC: 40, speedPercent: 30),
            FanCurvePoint(tempC: 80, speedPercent: 100),
          ];

          when(
            () => mockDBus.callMethod(
              destination: any(named: 'destination'),
              path: any(named: 'path'),
              interface: any(named: 'interface'),
              name: 'UpdateFanEngineCurve',
              values: any(named: 'values'),
              replySignature: any(named: 'replySignature'),
            ),
          ).thenAnswer(
            (_) async => DBusMethodSuccessResponse([DBusBoolean(true)]),
          );

          await client.updateFanCurve(FanId.cpu, curve);

          verify(
            () => mockDBus.callMethod(
              destination: 'com.asuslinuxcontrol.Daemon',
              path: DBusObjectPath('/com/asuslinuxcontrol/Daemon'),
              interface: 'com.asuslinuxcontrol.Daemon',
              name: 'UpdateFanEngineCurve',
              values: any(named: 'values'),
              replySignature: DBusSignature('b'),
            ),
          ).called(1);
        },
      );
    });

    group('getRunning', () {
      test(
        'delegates to daemon GetFanEngineRunning and returns boolean',
        () async {
          when(
            () => mockDBus.callMethod(
              destination: any(named: 'destination'),
              path: any(named: 'path'),
              interface: any(named: 'interface'),
              name: 'GetFanEngineRunning',
              values: any(named: 'values'),
              replySignature: any(named: 'replySignature'),
            ),
          ).thenAnswer(
            (_) async => DBusMethodSuccessResponse([DBusBoolean(false)]),
          );

          final result = await client.getRunning();

          expect(result, false);
          verify(
            () => mockDBus.callMethod(
              destination: 'com.asuslinuxcontrol.Daemon',
              path: DBusObjectPath('/com/asuslinuxcontrol/Daemon'),
              interface: 'com.asuslinuxcontrol.Daemon',
              name: 'GetFanEngineRunning',
              values: [],
              replySignature: DBusSignature('b'),
            ),
          ).called(1);
        },
      );
    });
  });
}
