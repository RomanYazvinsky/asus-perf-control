import 'dart:convert';
import 'dart:io';

import 'package:flutter_application_1/services/backend_logger.dart';
import 'package:flutter_application_1/services/settings/app_settings.dart';
import 'package:flutter_application_1/services/settings/hardware_profile_store.dart';
import 'package:flutter_test/flutter_test.dart';

void main() {
  group('FileHardwareProfileStore', () {
    late Directory tempDir;
    late Directory profilesDir;
    late List<String> logs;

    setUp(() async {
      tempDir = await Directory.systemTemp.createTemp('alc_profiles_test_');
      profilesDir = Directory('${tempDir.path}/profiles');
      logs = <String>[];
      BackendLogger.sink = logs.add;
    });

    tearDown(() async {
      BackendLogger.resetForTests();
      if (await tempDir.exists()) {
        await tempDir.delete(recursive: true);
      }
    });

    test('creates one JSON file per profile and lists it', () async {
      final store = FileHardwareProfileStore(directory: profilesDir);

      final profile = await store.create(
        name: 'Gaming Mode',
        hardware: const HardwareSettings(performanceMode: 2),
      );
      final profiles = await store.list();
      final raw = jsonDecode(
        await File('${profilesDir.path}/${profile.id}.json').readAsString(),
      );

      expect(profile.id, 'gaming-mode');
      expect(profile.name, 'Gaming Mode');
      expect(profiles, hasLength(1));
      expect(profiles.single.hardware.performanceMode, 2);
      expect(raw['schemaVersion'], HardwareProfile.currentSchemaVersion);
      expect(raw['id'], 'gaming-mode');
      expect(raw['name'], 'Gaming Mode');
      expect(raw['hardware']['performanceMode'], 2);
    });

    test('deduplicates colliding ids with numeric suffixes', () async {
      final store = FileHardwareProfileStore(directory: profilesDir);

      final first = await store.create(
        name: 'Gaming Mode',
        hardware: const HardwareSettings(performanceMode: 1),
      );
      final second = await store.create(
        name: 'Gaming Mode',
        hardware: const HardwareSettings(performanceMode: 2),
      );

      expect(first.id, 'gaming-mode');
      expect(second.id, 'gaming-mode-2');
      expect(await File('${profilesDir.path}/gaming-mode.json').exists(), true);
      expect(
        await File('${profilesDir.path}/gaming-mode-2.json').exists(),
        true,
      );
    });

    test('loads and saves an existing profile by id', () async {
      final store = FileHardwareProfileStore(directory: profilesDir);
      final profile = await store.create(
        name: 'Balanced',
        hardware: const HardwareSettings(performanceMode: 1),
      );

      await store.save(
        profile.copyWith(
          hardware: const HardwareSettings(
            performanceMode: 0,
            batteryChargeLimit: 80,
          ),
        ),
      );
      final loaded = await store.load(profile.id);

      expect(loaded?.id, profile.id);
      expect(loaded?.hardware.performanceMode, 0);
      expect(loaded?.hardware.batteryChargeLimit, 80);
    });

    test('skips invalid JSON files and logs the mismatch', () async {
      await profilesDir.create(recursive: true);
      await File('${profilesDir.path}/broken.json').writeAsString('{not-json');
      final store = FileHardwareProfileStore(directory: profilesDir);

      final profiles = await store.list();

      expect(profiles, isEmpty);
      expect(logs.join('\n'), contains('Settings.readMismatch'));
      expect(logs.join('\n'), contains('broken.json'));
    });

    test('deletes profile file by id', () async {
      final store = FileHardwareProfileStore(directory: profilesDir);
      final profile = await store.create(
        name: 'Quiet',
        hardware: const HardwareSettings(performanceMode: 0),
      );

      await store.delete(profile.id);

      expect(
        await File('${profilesDir.path}/${profile.id}.json').exists(),
        false,
      );
      expect(await store.list(), isEmpty);
    });
  });
}
