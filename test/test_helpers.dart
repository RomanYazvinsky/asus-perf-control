import 'package:flutter_application_1/bridge_generated.dart/frb_generated.dart';
import 'package:flutter_application_1/services/settings/app_settings.dart';
import 'package:flutter_application_1/services/settings/app_settings_store.dart';

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
