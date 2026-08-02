import 'dart:convert';
import 'dart:io';

import '../backend_logger.dart';
import 'app_settings.dart';

abstract class HardwareProfileStore {
  Future<List<HardwareProfile>> list();
  Future<HardwareProfile?> load(String id);
  Future<HardwareProfile> create({
    required String name,
    required HardwareSettings hardware,
  });
  Future<void> save(HardwareProfile profile);
  Future<void> delete(String id);
}

class HardwareProfile {
  static const int currentSchemaVersion = 1;

  final int schemaVersion;
  final String id;
  final String name;
  final HardwareSettings hardware;

  const HardwareProfile({
    this.schemaVersion = currentSchemaVersion,
    required this.id,
    required this.name,
    required this.hardware,
  });

  factory HardwareProfile.fromJson(
    Map<String, dynamic> json, {
    List<String>? issues,
  }) {
    final schemaVersion = _readInt(
      json,
      'schemaVersion',
      defaultValue: currentSchemaVersion,
      issues: issues,
      path: 'schemaVersion',
    );
    return HardwareProfile(
      schemaVersion: schemaVersion,
      id: _readString(json, 'id', issues: issues),
      name: _readString(json, 'name', issues: issues),
      hardware: HardwareSettings.fromJson(
        _readMap(json, 'hardware', issues: issues),
        issues,
      ),
    );
  }

  Map<String, dynamic> toJson() {
    return {
      'schemaVersion': schemaVersion,
      'id': id,
      'name': name,
      'hardware': hardware.toJson(),
    };
  }

  HardwareProfile copyWith({
    int? schemaVersion,
    String? id,
    String? name,
    HardwareSettings? hardware,
  }) {
    return HardwareProfile(
      schemaVersion: schemaVersion ?? this.schemaVersion,
      id: id ?? this.id,
      name: name ?? this.name,
      hardware: hardware ?? this.hardware,
    );
  }

  @override
  bool operator ==(Object other) {
    return other is HardwareProfile &&
        other.schemaVersion == schemaVersion &&
        other.id == id &&
        other.name == name &&
        other.hardware == hardware;
  }

  @override
  int get hashCode => Object.hash(schemaVersion, id, name, hardware);
}

class FileHardwareProfileStore implements HardwareProfileStore {
  final Directory directory;

  const FileHardwareProfileStore({required this.directory});

  @override
  Future<List<HardwareProfile>> list() async {
    if (!await directory.exists()) return const [];

    final profiles = <HardwareProfile>[];
    await for (final entity in directory.list()) {
      if (entity is! File || !entity.path.endsWith('.json')) continue;
      final profile = await _loadFile(entity);
      if (profile != null) profiles.add(profile);
    }

    profiles.sort(
      (a, b) => a.name.toLowerCase().compareTo(b.name.toLowerCase()),
    );
    return profiles;
  }

  @override
  Future<HardwareProfile?> load(String id) async {
    final safeId = _sanitizeId(id);
    if (safeId.isEmpty || safeId != id) return null;
    final file = _fileForId(safeId);
    if (!await file.exists()) return null;
    return _loadFile(file);
  }

  @override
  Future<HardwareProfile> create({
    required String name,
    required HardwareSettings hardware,
  }) async {
    final trimmedName = name.trim();
    if (trimmedName.isEmpty) {
      throw ArgumentError.value(name, 'name', 'Profile name cannot be blank');
    }

    final id = await _nextAvailableId(_sanitizeId(trimmedName));
    final profile = HardwareProfile(
      id: id,
      name: trimmedName,
      hardware: hardware,
    );
    await save(profile);
    return profile;
  }

  @override
  Future<void> save(HardwareProfile profile) async {
    final safeId = _sanitizeId(profile.id);
    if (safeId.isEmpty || safeId != profile.id) {
      throw ArgumentError.value(profile.id, 'profile.id', 'Invalid profile id');
    }
    await directory.create(recursive: true);
    await _fileForId(safeId).writeAsString(
      const JsonEncoder.withIndent('  ').convert(profile.toJson()),
    );
  }

  @override
  Future<void> delete(String id) async {
    final safeId = _sanitizeId(id);
    if (safeId.isEmpty || safeId != id) return;
    final file = _fileForId(safeId);
    if (await file.exists()) {
      await file.delete();
    }
  }

  Future<HardwareProfile?> _loadFile(File file) async {
    Map<String, dynamic> json;
    try {
      final decoded = jsonDecode(await file.readAsString());
      if (decoded is! Map<String, dynamic>) {
        _logReadMismatch(
          file,
          'root invalidType actual=${decoded.runtimeType}',
        );
        return null;
      }
      json = decoded;
    } catch (error) {
      _logReadMismatch(file, 'invalidJson error=$error');
      return null;
    }

    final issues = <String>[];
    final profile = HardwareProfile.fromJson(json, issues: issues);
    for (final issue in issues) {
      _logReadMismatch(file, issue);
    }
    if (issues.any(
      (issue) => issue.startsWith('id ') || issue.startsWith('name '),
    )) {
      return null;
    }
    return profile;
  }

  Future<String> _nextAvailableId(String baseId) async {
    final normalizedBase = baseId.isEmpty ? 'profile' : baseId;
    var candidate = normalizedBase;
    var suffix = 2;
    while (await _fileForId(candidate).exists()) {
      candidate = '$normalizedBase-$suffix';
      suffix += 1;
    }
    return candidate;
  }

  File _fileForId(String id) => File('${directory.path}/$id.json');

  static String _sanitizeId(String source) {
    final lower = source.toLowerCase();
    final buffer = StringBuffer();
    var lastWasSeparator = false;
    for (final codeUnit in lower.codeUnits) {
      final isLetter = codeUnit >= 97 && codeUnit <= 122;
      final isDigit = codeUnit >= 48 && codeUnit <= 57;
      if (isLetter || isDigit) {
        buffer.writeCharCode(codeUnit);
        lastWasSeparator = false;
      } else if (!lastWasSeparator && buffer.isNotEmpty) {
        buffer.write('-');
        lastWasSeparator = true;
      }
    }
    return buffer.toString().replaceFirst(RegExp(r'-+$'), '');
  }

  void _logReadMismatch(File file, String issue) {
    BackendLogger.logEvent(
      'Settings.readMismatch',
      fields: {'file': file.path, 'issue': issue},
    );
  }
}

Map<String, dynamic> _readMap(
  Map<String, dynamic> json,
  String key, {
  List<String>? issues,
}) {
  final value = json[key];
  if (value == null) return const {};
  if (value is Map<String, dynamic>) return value;
  issues?.add('$key invalidType actual=${value.runtimeType}');
  return const {};
}

int _readInt(
  Map<String, dynamic> json,
  String key, {
  required int defaultValue,
  required String path,
  List<String>? issues,
}) {
  final value = json[key];
  if (value == null) return defaultValue;
  if (value is int) return value;
  issues?.add('$path invalidType actual=${value.runtimeType}');
  return defaultValue;
}

String _readString(
  Map<String, dynamic> json,
  String key, {
  List<String>? issues,
}) {
  final value = json[key];
  if (value is String && value.isNotEmpty) return value;
  issues?.add('$key invalidValue');
  return '';
}
