import 'dart:convert';
import 'dart:io';

import '../backend_logger.dart';
import 'app_settings.dart';
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
