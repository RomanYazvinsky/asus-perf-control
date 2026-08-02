import 'package:flutter_application_1/bridge_generated.dart/frb_generated.dart';
import 'package:flutter_application_1/services/settings/app_settings.dart';
import 'package:flutter_application_1/services/settings/app_settings_store.dart';
import 'package:flutter_application_1/services/settings/hardware_profile_store.dart';

/// Initialize FFI for tests
Future<void> initFFI() async {
  try {
    await RustLib.init();
  } catch (e) {
    // FFI may not be available in test environment
    // This is expected for pure Dart tests
  }
}

class MemoryAppSettingsStore implements AppSettingsStore {
  AppSettings _settings;
  final savedSettings = <AppSettings>[];

  MemoryAppSettingsStore(this._settings);

  AppSettings get current => _settings;

  @override
  Future<AppSettings> load() async => _settings;

  @override
  Future<void> save(AppSettings settings) async {
    _settings = settings;
    savedSettings.add(settings);
  }

  @override
  Future<AppSettings> update(
    AppSettings Function(AppSettings settings) update,
  ) async {
    final next = update(_settings);
    await save(next);
    return next;
  }
}

class MemoryHardwareProfileStore implements HardwareProfileStore {
  final List<HardwareProfile> profiles;
  final savedProfiles = <HardwareProfile>[];
  final deletedProfileIds = <String>[];

  MemoryHardwareProfileStore([List<HardwareProfile>? profiles])
    : profiles = List<HardwareProfile>.from(profiles ?? const []);

  @override
  Future<List<HardwareProfile>> list() async => List.unmodifiable(profiles);

  @override
  Future<HardwareProfile?> load(String id) async {
    for (final profile in profiles) {
      if (profile.id == id) return profile;
    }
    return null;
  }

  @override
  Future<HardwareProfile> create({
    required String name,
    required HardwareSettings hardware,
  }) async {
    final id = _nextId(name);
    final profile = HardwareProfile(
      id: id,
      name: name.trim(),
      hardware: hardware,
    );
    profiles.add(profile);
    savedProfiles.add(profile);
    return profile;
  }

  @override
  Future<void> save(HardwareProfile profile) async {
    final index = profiles.indexWhere((item) => item.id == profile.id);
    if (index == -1) {
      profiles.add(profile);
    } else {
      profiles[index] = profile;
    }
    savedProfiles.add(profile);
  }

  @override
  Future<void> delete(String id) async {
    deletedProfileIds.add(id);
    profiles.removeWhere((profile) => profile.id == id);
  }

  String _nextId(String name) {
    final base = _slug(name).isEmpty ? 'profile' : _slug(name);
    var candidate = base;
    var suffix = 2;
    while (profiles.any((profile) => profile.id == candidate)) {
      candidate = '$base-$suffix';
      suffix += 1;
    }
    return candidate;
  }

  String _slug(String name) {
    final lower = name.trim().toLowerCase();
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
}
