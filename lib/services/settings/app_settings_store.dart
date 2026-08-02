import 'dart:convert';
import 'dart:io';

import '../backend_logger.dart';
import 'app_settings.dart';
import 'hardware_profile_store.dart';
import 'settings_migration.dart';

abstract class AppSettingsStore {
  Future<AppSettings> load();
  Future<void> save(AppSettings settings);

  Future<AppSettings> update(AppSettings Function(AppSettings settings) update);
}

class FileAppSettingsStore implements AppSettingsStore {
  final File file;
  final SettingsMigrator migrator;
  Future<void> _pendingUpdate = Future<void>.value();

  FileAppSettingsStore({File? file, SettingsMigrator? migrator})
    : file = file ?? defaultSettingsFile(),
      migrator =
          migrator ??
          const SettingsMigrator(
            currentVersion: AppSettings.currentSchemaVersion,
            migrations: [_SettingsV1ToV2Migration()],
          );

  @override
  Future<AppSettings> load() async {
    if (!await file.exists()) {
      return AppSettings.defaults;
    }

    Map<String, dynamic> json;
    try {
      final decoded = jsonDecode(await file.readAsString());
      if (decoded is! Map<String, dynamic>) {
        _logReadMismatch('root invalidType actual=${decoded.runtimeType}');
        return AppSettings.defaults;
      }
      json = decoded;
    } catch (error) {
      _logReadMismatch('invalidJson error=$error');
      return AppSettings.defaults;
    }

    final migration = migrator.migrate(json);
    for (final issue in migration.issues) {
      _logReadMismatch(issue);
    }
    if (migration.issues.isNotEmpty) {
      return AppSettings.defaults;
    }

    final issues = <String>[];
    final settings = AppSettings.fromJson(migration.json, issues: issues);
    for (final issue in issues) {
      _logReadMismatch(issue);
    }

    if (migration.migrated) {
      await save(settings);
    }
    return settings;
  }

  @override
  Future<void> save(AppSettings settings) async {
    await file.parent.create(recursive: true);
    await file.writeAsString(
      const JsonEncoder.withIndent('  ').convert(settings.toJson()),
    );
  }

  @override
  Future<AppSettings> update(
    AppSettings Function(AppSettings settings) update,
  ) async {
    late AppSettings next;
    _pendingUpdate = _pendingUpdate.then((_) async {
      final current = await load();
      next = update(current);
      await save(next);
    });
    await _pendingUpdate;
    return next;
  }

  static File defaultSettingsFile() {
    final configHome = Platform.environment['XDG_CONFIG_HOME'];
    final home = Platform.environment['HOME'];
    final base =
        configHome != null &&
            configHome.isNotEmpty &&
            configHome.startsWith('/')
        ? configHome
        : '${home == null || home.isEmpty ? '.' : home}/.config';
    return File('$base/asus-linux-control/settings.json');
  }

  void _logReadMismatch(String issue) {
    BackendLogger.logEvent(
      'Settings.readMismatch',
      fields: {'file': file.path, 'issue': issue},
    );
  }
}

class ProfiledAppSettingsStore implements AppSettingsStore {
  final AppSettingsStore globalStore;
  final HardwareProfileStore profileStore;
  Future<void> _pendingUpdate = Future<void>.value();

  ProfiledAppSettingsStore({
    required this.globalStore,
    required this.profileStore,
  });

  @override
  Future<AppSettings> load() async {
    final global = await globalStore.load();
    return _withActiveProfileHardware(global);
  }

  @override
  Future<void> save(AppSettings settings) async {
    final global = await globalStore.load();
    final activeProfileId = settings.ui.activeHardwareProfileId;
    if (activeProfileId == null) {
      await globalStore.save(settings);
      return;
    }

    final profile = await profileStore.load(activeProfileId);
    if (profile == null) {
      await globalStore.save(settings);
      return;
    }

    await globalStore.save(
      global.copyWith(schemaVersion: settings.schemaVersion, ui: settings.ui),
    );
    await profileStore.save(profile.copyWith(hardware: settings.hardware));
  }

  @override
  Future<AppSettings> update(
    AppSettings Function(AppSettings settings) update,
  ) async {
    late AppSettings next;
    _pendingUpdate = _pendingUpdate.then((_) async {
      final global = await globalStore.load();
      final current = await _withActiveProfileHardware(global);
      next = update(current);

      final previousProfileId = current.ui.activeHardwareProfileId;
      final nextProfileId = next.ui.activeHardwareProfileId;
      if (nextProfileId == null) {
        await globalStore.save(next);
        return;
      }

      final profile = await profileStore.load(nextProfileId);
      if (profile == null) {
        await globalStore.save(next);
        return;
      }

      await globalStore.save(
        global.copyWith(schemaVersion: next.schemaVersion, ui: next.ui),
      );

      final profileChanged = nextProfileId != previousProfileId;
      final hardwareChanged = next.hardware != current.hardware;
      if (!profileChanged || hardwareChanged) {
        await profileStore.save(profile.copyWith(hardware: next.hardware));
      }
    });
    await _pendingUpdate;
    return next;
  }

  Future<AppSettings> _withActiveProfileHardware(AppSettings global) async {
    final activeProfileId = global.ui.activeHardwareProfileId;
    if (activeProfileId == null) return global;

    final profile = await profileStore.load(activeProfileId);
    if (profile == null) return global;
    return global.copyWith(hardware: profile.hardware);
  }
}

class _SettingsV1ToV2Migration implements SettingsMigration {
  const _SettingsV1ToV2Migration();

  @override
  int get fromVersion => 1;

  @override
  int get toVersion => 2;

  @override
  Map<String, dynamic> migrate(Map<String, dynamic> source) {
    final migrated = Map<String, dynamic>.from(source);
    migrated['schemaVersion'] = toVersion;
    return migrated;
  }

  @override
  Map<String, dynamic> rollback(Map<String, dynamic> source) {
    final rolledBack = Map<String, dynamic>.from(source);
    rolledBack['schemaVersion'] = fromVersion;
    final ui = Map<String, dynamic>.from(
      rolledBack['ui'] as Map<String, dynamic>? ?? const {},
    );
    ui.remove('activeHardwareProfileId');
    rolledBack['ui'] = ui;
    return rolledBack;
  }
}
