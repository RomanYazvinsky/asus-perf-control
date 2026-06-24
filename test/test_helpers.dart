import 'package:flutter_application_1/bridge_generated.dart/frb_generated.dart';

/// Initialize FFI for tests
Future<void> initFFI() async {
  try {
    await RustLib.init();
  } catch (e) {
    // FFI may not be available in test environment
    // This is expected for pure Dart tests
  }
}
