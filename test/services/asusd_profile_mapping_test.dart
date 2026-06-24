import 'package:flutter_test/flutter_test.dart';
import 'package:flutter_application_1/services/dbus_client.dart';

void main() {
  group('AsusdClient platform profile mapping', () {
    test('reads Quiet and LowPower as Silent', () {
      expect(AsusdClient.modeFromPlatformProfileForTest(2), 0);
      expect(AsusdClient.modeFromPlatformProfileForTest(3), 0);
    });

    test('writes Silent as Quiet instead of LowPower', () {
      expect(AsusdClient.platformProfileFromModeForTest(0), 2);
    });
  });
}
