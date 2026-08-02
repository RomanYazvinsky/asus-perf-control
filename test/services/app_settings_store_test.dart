import 'dart:convert';
import 'dart:io';

import 'package:flutter_application_1/services/backend_logger.dart';
import 'package:flutter_application_1/services/settings/app_settings.dart';
import 'package:flutter_application_1/services/settings/app_settings_store.dart';
import 'package:flutter_application_1/services/settings/settings_migration.dart';
import 'package:flutter_test/flutter_test.dart';

void main() {
  group('FileAppSettingsStore', () {
    late Directory tempDir;
    late File settingsFile;
    late List<String> logs;

    setUp(() async {
      tempDir = await Directory.systemTemp.createTemp('alc_settings_test_');
      settingsFile = File('${tempDir.path}/settings.json');
      logs = <String>[];
      BackendLogger.sink = logs.add;
    });

    tearDown(() async {
      BackendLogger.resetForTests();
      if (await tempDir.exists()) {
        await tempDir.delete(recursive: true);
      }
    });

    test('loads defaults when the settings file does not exist', () async {
      final store = FileAppSettingsStore(file: settingsFile);

      final settings = await store.load();

      expect(settings.schemaVersion, AppSettings.currentSchemaVersion);
      expect(settings.ui.selectedNavigationIndex, 0);
      expect(settings.hardware.performanceMode, isNull);
    });

    test('saves and loads a formatted JSON settings file', () async {
      final store = FileAppSettingsStore(file: settingsFile);
      const settings = AppSettings(
        ui: UiSettings(selectedNavigationIndex: 4),
        hardware: HardwareSettings(
          performanceMode: 2,
          tdp: TdpDesiredSettings(
            powerLimitsEnabled: true,
            sustainedW: 25,
            boostW: 35,
            tempLimitEnabled: true,
            tempLimitC: 85,
          ),
          cpuBoostMode: 'efficient_enabled',
          batteryChargeLimit: 80,
          gpuMode: 'hybrid',
          fan: FanDesiredSettings(
            selectedProfile: 'balanced',
            selectedFan: 'cpu',
            controlModes: {'cpu': 'asusctlCurve'},
          ),
        ),
      );

      await store.save(settings);
      final loaded = await store.load();
      final raw = jsonDecode(await settingsFile.readAsString());

      expect(loaded.ui.selectedNavigationIndex, 4);
      expect(loaded.hardware.performanceMode, 2);
      expect(loaded.hardware.tdp?.tempLimitC, 85);
      expect(loaded.hardware.cpuBoostMode, 'efficient_enabled');
      expect(raw['schemaVersion'], AppSettings.currentSchemaVersion);
      expect(raw['hardware']['fan']['controlModes']['cpu'], 'asusctlCurve');
    });

    test('omits unset hardware fields from saved JSON', () async {
      final store = FileAppSettingsStore(file: settingsFile);

      await store.save(
        const AppSettings(ui: UiSettings(selectedNavigationIndex: 1)),
      );
      final raw = jsonDecode(await settingsFile.readAsString());

      expect(raw['ui']['selectedNavigationIndex'], 1);
      expect(raw['hardware'], isEmpty);
    });

    test('logs and falls back to defaults for invalid JSON', () async {
      await settingsFile.parent.create(recursive: true);
      await settingsFile.writeAsString('{not-json');
      final store = FileAppSettingsStore(file: settingsFile);

      final settings = await store.load();

      expect(settings, AppSettings.defaults);
      expect(logs.join('\n'), contains('Settings.readMismatch'));
      expect(logs.join('\n'), contains('invalidJson'));
    });

    test('logs invalid fields while keeping valid values', () async {
      await settingsFile.parent.create(recursive: true);
      await settingsFile.writeAsString(
        jsonEncode({
          'schemaVersion': AppSettings.currentSchemaVersion,
          'ui': {'selectedNavigationIndex': 'bad'},
          'hardware': {'performanceMode': 2, 'batteryChargeLimit': 130},
        }),
      );
      final store = FileAppSettingsStore(file: settingsFile);

      final settings = await store.load();

      expect(settings.ui.selectedNavigationIndex, 0);
      expect(settings.hardware.performanceMode, 2);
      expect(settings.hardware.batteryChargeLimit, isNull);
      expect(logs.join('\n'), contains('ui.selectedNavigationIndex'));
      expect(logs.join('\n'), contains('hardware.batteryChargeLimit'));
    });

    test('migrates schema version 1 settings to version 2', () async {
      await settingsFile.parent.create(recursive: true);
      await settingsFile.writeAsString(
        jsonEncode({
          'schemaVersion': 1,
          'ui': {'selectedNavigationIndex': 2},
          'hardware': {'performanceMode': 1},
        }),
      );
      final store = FileAppSettingsStore(file: settingsFile);

      final settings = await store.load();
      final migratedJson = jsonDecode(await settingsFile.readAsString());

      expect(AppSettings.currentSchemaVersion, 2);
      expect(settings.schemaVersion, 2);
      expect(settings.ui.selectedNavigationIndex, 2);
      expect(settings.ui.activeHardwareProfileId, isNull);
      expect(settings.hardware.performanceMode, 1);
      expect(migratedJson['schemaVersion'], 2);
      expect(migratedJson['ui'].containsKey('activeHardwareProfileId'), false);
    });

    test('migrates older JSON and can roll the migrated JSON back', () async {
      final migrator = SettingsMigrator(
        currentVersion: 2,
        migrations: const [_RenameTabIndexMigration()],
      );
      final store = FileAppSettingsStore(
        file: settingsFile,
        migrator: migrator,
      );
      await settingsFile.parent.create(recursive: true);
      await settingsFile.writeAsString(
        jsonEncode({
          'schemaVersion': 1,
          'ui': {'tabIndex': 5},
        }),
      );

      final settings = await store.load();
      final migratedJson = jsonDecode(await settingsFile.readAsString());
      final rolledBack = migrator.rollback(migratedJson, targetVersion: 1);

      expect(settings.schemaVersion, 2);
      expect(settings.ui.selectedNavigationIndex, 5);
      expect(migratedJson['ui']['selectedNavigationIndex'], 5);
      expect(rolledBack['schemaVersion'], 1);
      expect(rolledBack['ui']['tabIndex'], 5);
    });
  });
}

class _RenameTabIndexMigration implements SettingsMigration {
  const _RenameTabIndexMigration();

  @override
  int get fromVersion => 1;

  @override
  int get toVersion => 2;

  @override
  Map<String, dynamic> migrate(Map<String, dynamic> source) {
    final migrated = Map<String, dynamic>.from(source);
    final ui = Map<String, dynamic>.from(
      migrated['ui'] as Map<String, dynamic>? ?? const {},
    );
    ui['selectedNavigationIndex'] = ui.remove('tabIndex');
    migrated['ui'] = ui;
    migrated['schemaVersion'] = toVersion;
    return migrated;
  }

  @override
  Map<String, dynamic> rollback(Map<String, dynamic> source) {
    final rolledBack = Map<String, dynamic>.from(source);
    final ui = Map<String, dynamic>.from(
      rolledBack['ui'] as Map<String, dynamic>? ?? const {},
    );
    ui['tabIndex'] = ui.remove('selectedNavigationIndex');
    rolledBack['ui'] = ui;
    rolledBack['schemaVersion'] = fromVersion;
    return rolledBack;
  }
}
