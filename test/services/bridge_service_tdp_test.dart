import 'package:flutter_test/flutter_test.dart';
import 'package:mocktail/mocktail.dart';

import 'package:flutter_application_1/services/bridge_service.dart';
import 'package:flutter_application_1/services/dbus_client.dart';

class _MockDbusClient extends Mock implements DbusClient {}

class _MockAsusdClient extends Mock implements AsusdClient {}

void main() {
  group('BridgeService TDP delegation', () {
    late _MockDbusClient dbusClient;
    late _MockAsusdClient asusdClient;
    late BridgeService service;

    setUp(() {
      dbusClient = _MockDbusClient();
      asusdClient = _MockAsusdClient();
      service = BridgeService.test(
        dbusClient: dbusClient,
        asusdClient: asusdClient,
      );
      when(
        () => asusdClient.setPowerLimitsEnabled(any()),
      ).thenAnswer((_) async {});
    });

    test('setPowerLimits delegates only to AsusdClient', () async {
      when(
        () => asusdClient.setTdpConfig(sustainedW: 25, boostW: 40),
      ).thenAnswer((_) async {});

      await service.setPowerLimits(sustainedW: 25, boostW: 40);

      verify(
        () => asusdClient.setTdpConfig(sustainedW: 25, boostW: 40),
      ).called(1);
      verifyNever(() => dbusClient.setTempLimit(any()));
    });

    test('setCpuTempLimit delegates only to DbusClient', () async {
      when(() => dbusClient.setTempLimit(87)).thenAnswer((_) async => true);

      await service.setCpuTempLimit(87);

      verify(() => dbusClient.setTempLimit(87)).called(1);
      verifyNever(
        () => asusdClient.setTdpConfig(
          sustainedW: any(named: 'sustainedW'),
          boostW: any(named: 'boostW'),
        ),
      );
    });

    test('getTdpConfig uses explicit power apply flag from asusd', () async {
      when(() => asusdClient.getTdpConfig()).thenAnswer(
        (_) async => TdpConfigDbus(
          isSupported: true,
          sustainedW: 22,
          boostW: 33,
          tempLimitC: 0,
          minSustainedW: 15,
          maxSustainedW: 45,
          minBoostW: 20,
          maxBoostW: 60,
          powerLimitsEnabled: true,
        ),
      );
      when(() => dbusClient.getTempLimit()).thenAnswer((_) async => 0);
      when(() => asusdClient.getPerformanceMode()).thenAnswer((_) async => 2);

      final config = await service.getTdpConfig();

      expect(config.sustainedW, 22);
      expect(config.boostW, 33);
      expect(config.powerLimitsEnabled, isTrue);
    });

    test('getTdpConfig reports active profile mode from asusd', () async {
      when(() => asusdClient.getTdpConfig()).thenAnswer(
        (_) async => TdpConfigDbus(
          isSupported: true,
          sustainedW: 22,
          boostW: 33,
          tempLimitC: 0,
          powerLimitsEnabled: true,
        ),
      );
      when(() => dbusClient.getTempLimit()).thenAnswer((_) async => 75);
      when(() => asusdClient.getPerformanceMode()).thenAnswer((_) async => 0);

      final config = await service.getTdpConfig();

      expect(config.activePerformanceMode, 0);
      expect(config.tempLimitC, 75);
    });

    test(
      'setPowerLimits persists power apply flag after asusd write',
      () async {
        when(
          () => asusdClient.setTdpConfig(sustainedW: 25, boostW: 40),
        ).thenAnswer((_) async {});
        when(
          () => asusdClient.setPowerLimitsEnabled(true),
        ).thenAnswer((_) async {});

        await service.setPowerLimits(sustainedW: 25, boostW: 40);

        verify(
          () => asusdClient.setTdpConfig(sustainedW: 25, boostW: 40),
        ).called(1);
        verify(() => asusdClient.setPowerLimitsEnabled(true)).called(1);
      },
    );

    test(
      'clearPowerLimits resets asusd limits and persists disabled flag',
      () async {
        when(
          () => asusdClient.resetPowerLimitsToProfileDefaults(),
        ).thenAnswer((_) async {});
        when(
          () => asusdClient.setPowerLimitsEnabled(false),
        ).thenAnswer((_) async {});

        await service.clearPowerLimits();

        verify(() => asusdClient.resetPowerLimitsToProfileDefaults()).called(1);
        verify(() => asusdClient.setPowerLimitsEnabled(false)).called(1);
        verifyNever(() => dbusClient.setTempLimit(any()));
      },
    );
  });
}
