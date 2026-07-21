abstract class SettingsMigration {
  int get fromVersion;
  int get toVersion;

  Map<String, dynamic> migrate(Map<String, dynamic> source);
  Map<String, dynamic> rollback(Map<String, dynamic> source);
}

class SettingsMigrator {
  final int currentVersion;
  final List<SettingsMigration> migrations;

  const SettingsMigrator({
    required this.currentVersion,
    this.migrations = const [],
  });

  SettingsMigrationResult migrate(Map<String, dynamic> source) {
    final initialVersion = _readVersion(source);
    if (initialVersion > currentVersion) {
      return SettingsMigrationResult(
        json: source,
        migrated: false,
        issues: [
          'schemaVersionUnsupported actual=$initialVersion expected=$currentVersion',
        ],
      );
    }

    var version = initialVersion;
    var json = Map<String, dynamic>.from(source);
    final issues = <String>[];

    while (version < currentVersion) {
      final migration = _migrationFrom(version);
      if (migration == null) {
        issues.add('migrationMissing fromVersion=$version');
        break;
      }
      json = migration.migrate(json);
      version = migration.toVersion;
    }

    return SettingsMigrationResult(
      json: json,
      migrated: version != initialVersion,
      issues: issues,
    );
  }

  Map<String, dynamic> rollback(
    Map<String, dynamic> source, {
    required int targetVersion,
  }) {
    var version = _readVersion(source);
    var json = Map<String, dynamic>.from(source);

    while (version > targetVersion) {
      final migration = _migrationTo(version);
      if (migration == null) {
        throw StateError('Missing settings rollback from version $version');
      }
      json = migration.rollback(json);
      version = migration.fromVersion;
    }

    return json;
  }

  SettingsMigration? _migrationFrom(int version) {
    for (final migration in migrations) {
      if (migration.fromVersion == version) return migration;
    }
    return null;
  }

  SettingsMigration? _migrationTo(int version) {
    for (final migration in migrations.reversed) {
      if (migration.toVersion == version) return migration;
    }
    return null;
  }

  int _readVersion(Map<String, dynamic> source) {
    final version = source['schemaVersion'];
    return version is int ? version : 1;
  }
}

class SettingsMigrationResult {
  final Map<String, dynamic> json;
  final bool migrated;
  final List<String> issues;

  const SettingsMigrationResult({
    required this.json,
    required this.migrated,
    required this.issues,
  });
}
