# Quickstart Guide: ASUS Linux Control Tool

**Feature**: 001-asus-linux-control  
**Date**: 2026-04-08  
**Audience**: Developers working on implementation

## Prerequisites

### System Requirements
- Linux kernel 5.6+ (5.17+ recommended for full feature support)
- ASUS laptop with asus-nb-wmi driver support
- x86_64 architecture

### Development Environment
- Rust 1.75+ with cargo
- Dart 3.0+ and Flutter 3.16+
- Git
- C compiler (gcc or clang) for native dependencies

### Hardware Requirements
- ASUS ROG, TUF, Strix, Zephyrus, or Flow series laptop (2020+)
- Supported models use asus-nb-wmi kernel driver

---

## Project Setup

### 1. Clone Repository (Future)
```bash
git clone <repository-url>
cd asus-linux-control
```

### 2. Install Rust Dependencies
```bash
# Navigate to Rust backend
cd rust_core

# Build and test
cargo build
cargo test
```

**Key Rust Crates**:
- `libmedium = "0.13"` - hwmon sensor abstraction
- `nvml-wrapper = "0.10"` - Nvidia GPU monitoring
- `hidapi = "2.4"` - USB HID for Anime Matrix
- `upower_dbus = "0.3"` - Battery status via D-Bus
- `zbus = "3.0"` - D-Bus async client
- `serde = { version = "1.0", features = ["derive"] }` - Serialization
- `serde_json = "1.0"` - JSON config persistence
- `tokio = { version = "1", features = ["full"] }` - Async runtime
- `flutter_rust_bridge = "2.0"` - FFI code generation

### 3. Install Flutter Dependencies
```bash
# Navigate to project root
cd ..

# Get Flutter packages
flutter pub get

# Verify Flutter installation
flutter doctor
```

**Key Flutter Packages** (add to `pubspec.yaml`):
```yaml
dependencies:
  flutter:
    sdk: flutter
  flutter_bloc: ^8.1.0          # BLoC state management
  bloc: ^8.1.0                  # Core BLoC library
  fl_chart: ^1.2.0              # Interactive charting (real-time graphs, fan curves)
  flutter_rust_bridge: ^2.0.0   # FFI integration

dev_dependencies:
  flutter_test:
    sdk: flutter
  integration_test:
    sdk: flutter
  bloc_test: ^9.1.0             # BLoC testing utilities
```

**Material Design 3 Configuration** (in `lib/main.dart`):
```dart
MaterialApp(
  theme: ThemeData(
    useMaterial3: true,
    brightness: Brightness.dark,  // Dark theme only
    colorScheme: ColorScheme.fromSeed(
      seedColor: Colors.blue,
      brightness: Brightness.dark,
    ),
  ),
  // ...
)
```

### 4. Generate FFI Bridge
```bash
# Install flutter_rust_bridge codegen
cargo install flutter_rust_bridge_codegen

# Generate Dart bindings from Rust API
flutter_rust_bridge_codegen \
  --rust-input rust_core/src/api.rs \
  --dart-output lib/bridge_generated.dart \
  --dart-decl-output lib/bridge_definitions.dart
```

**Regenerate after**:
- Changing Rust API signatures in `api.rs`
- Adding new functions to FFI interface
- Modifying data structures shared between Rust/Dart

### 5. Install System Permissions
```bash
# Copy udev rules (requires sudo)
sudo cp install/99-asus-control.rules /etc/udev/rules.d/
sudo udevadm control --reload-rules
sudo udevadm trigger

# Copy polkit policy (optional, for future daemon features)
sudo cp install/50-asus-control.rules /etc/polkit-1/rules.d/

# Reboot recommended for full permission changes
```

---

## Development Workflow

### Running the Application

**Development Mode** (hot reload):
```bash
flutter run -d linux
```

**Release Build**:
```bash
flutter build linux --release
./build/linux/x64/release/bundle/asus_linux_control
```

### Testing

**Rust Unit Tests**:
```bash
cd rust_core
cargo test

# Run specific test
cargo test test_performance_controller

# With output
cargo test -- --nocapture
```

**Rust Integration Tests**:
```bash
cd rust_core
cargo test --test integration_tests
```

**Flutter Widget Tests**:
```bash
flutter test

# Specific test file
flutter test test/widget_test.dart
```

**Flutter Integration Tests**:
```bash
flutter test integration_test/app_test.dart
```

### Code Quality

**Rust Linting**:
```bash
cd rust_core
cargo clippy -- -D warnings
cargo fmt --check
```

**Flutter Linting**:
```bash
flutter analyze
dart format --set-exit-if-changed .
```

---

## Project Structure Quick Reference

```
asus-linux-control/
├── rust_core/                  # Rust backend
│   ├── src/
│   │   ├── api.rs             # FFI exports to Flutter
│   │   ├── services/          # Hardware control modules
│   │   └── models/            # Data structures
│   └── tests/                 # Rust tests
│
├── lib/                        # Flutter frontend
│   ├── main.dart              # App entry point
│   ├── bridge_generated.dart  # Generated FFI bindings
│   ├── screens/               # UI screens
│   ├── widgets/               # Reusable components
│   └── providers/             # State management
│
├── install/                    # Installation scripts
│   ├── 99-asus-control.rules  # udev rules
│   └── install.sh             # Setup script
│
└── specs/                      # Documentation
    └── 001-asus-linux-control/
        ├── spec.md            # Feature specification
        ├── plan.md            # Implementation plan (this workflow)
        ├── research.md        # Technical research
        ├── data-model.md      # Entity definitions
        ├── contracts/         # API contracts
        └── quickstart.md      # This file
```

---

## Common Development Tasks

### Adding a New Feature

1. **Update Specification**: Edit `specs/001-asus-linux-control/spec.md`
2. **Implement Rust Backend**:
   - Add service in `rust_core/src/services/`
   - Export function in `rust_core/src/api.rs`
   - Write tests in `rust_core/tests/`
3. **Regenerate FFI Bridge**: Run `flutter_rust_bridge_codegen`
4. **Implement Flutter UI**:
   - Add screen in `lib/screens/`
   - Create widgets in `lib/widgets/`
   - Update provider in `lib/providers/`
5. **Test End-to-End**: Run integration tests

### Debugging Hardware Access Issues

**Check driver loaded**:
```bash
lsmod | grep asus_wmi
```

**Check sysfs interfaces available**:
```bash
ls -la /sys/class/platform/asus-nb-wmi/
ls -la /sys/class/hwmon/
```

**Check permissions**:
```bash
# Thermal policy
cat /sys/class/platform/asus-nb-wmi/throttle_thermal_policy

# hwmon (replace hwmon3 with actual device)
cat /sys/class/hwmon/hwmon3/temp1_input
```

**Check USB HID devices** (for Anime Matrix):
```bash
lsusb | grep 0b05  # ASUS vendor ID
```

**Enable Rust debug logging**:
```bash
RUST_LOG=debug ./build/linux/x64/release/bundle/asus_linux_control
```

### Working with Configuration

**Config location**: `~/.config/asus-linux-control/config.json`

**Reset to defaults**:
```bash
rm ~/.config/asus-linux-control/config.json
# Restart application - will regenerate with defaults
```

**Manual config edit**:
```bash
# Edit with your preferred editor
nano ~/.config/asus-linux-control/config.json

# Validate JSON syntax
jq . ~/.config/asus-linux-control/config.json
```

---

## Hardware-Specific Notes

### Performance Modes
- **Kernel 5.6-5.16**: Only 3 modes (Silent/Balanced/Turbo)
- **Kernel 5.17+**: Extended mode support on some models
- Check: `cat /sys/class/platform/asus-nb-wmi/throttle_thermal_policy`

### GPU Control
- **dGPU Toggle**: Requires kernel 5.17+ (`dgpu_disable` interface)
- **Nvidia**: Requires proprietary drivers + `libnvidia-ml.so`
- **AMD**: Uses sysfs, works with open-source drivers
- Hybrid graphics may require `optimus-manager` or `supergfxctl`

### Fan Control
- **Manual PWM**: Requires `pwm_enable=1` support in hwmon driver
- **Fan Curves**: Hardware-dependent, not all models support auto points
- **Safety**: Application enforces minimum 30% PWM at 80°C+ to prevent thermal damage

### Anime Matrix
- **USB Permissions**: Requires udev rules or running as root
- **Protocol**: Reverse-engineered, may vary by model year
- **Detection**: VID=0x0b05, PID varies (e.g., 0x193b for some models)

---

## Troubleshooting

### "ASUS WMI thermal policy interface not found"
**Cause**: asus-nb-wmi driver not loaded or unsupported hardware  
**Solution**: 
```bash
sudo modprobe asus_wmi
dmesg | grep asus  # Check for driver messages
```

### "Permission denied" errors
**Cause**: udev rules not installed or not active  
**Solution**:
```bash
# Reinstall udev rules
sudo cp install/99-asus-control.rules /etc/udev/rules.d/
sudo udevadm control --reload-rules
sudo udevadm trigger

# Or reboot system
sudo reboot
```

### FFI bridge errors after Rust changes
**Cause**: Generated bindings out of sync  
**Solution**:
```bash
flutter_rust_bridge_codegen \
  --rust-input rust_core/src/api.rs \
  --dart-output lib/bridge_generated.dart \
  --dart-decl-output lib/bridge_definitions.dart

# Clean build
flutter clean
flutter pub get
flutter run
```

### Application crashes on sensor reading
**Cause**: hwmon device enumeration changed (hotplug)  
**Solution**: Application should handle this gracefully. If not, file a bug report with:
```bash
# Capture hwmon state
ls -la /sys/class/hwmon/
cat /sys/class/hwmon/*/name
```

---

## Next Steps

After completing quickstart setup:

1. **Read Specification**: `specs/001-asus-linux-control/spec.md`
2. **Study Data Model**: `specs/001-asus-linux-control/data-model.md`
3. **Review API Contract**: `specs/001-asus-linux-control/contracts/ffi-api.md`
4. **Implement Tasks**: Follow `specs/001-asus-linux-control/tasks.md` (generated by `/speckit.tasks`)

---

## Resources

### Documentation
- **Kernel Docs**: https://www.kernel.org/doc/Documentation/ABI/testing/sysfs-platform-asus-wmi
- **hwmon Interface**: https://www.kernel.org/doc/Documentation/hwmon/sysfs-interface
- **Flutter Rust Bridge**: https://cjycode.com/flutter_rust_bridge/
- **NVML API**: https://docs.nvidia.com/deploy/nvml-api/

### Reference Implementations
- **G-Helper** (Windows): https://github.com/seerge/g-helper
- **AsusCtl** (Linux): https://gitlab.com/asus-linux/asusctl
- **Starlight** (Anime Matrix): https://github.com/vddCore/Starlight

### Community
- **ASUS Linux**: https://asus-linux.org/
- **Reddit**: r/ASUS, r/linux_gaming
- **Discord**: ASUS Linux community server

---

## Contributing

### Before Submitting PRs
1. Run all tests (`cargo test` + `flutter test`)
2. Check code style (`cargo clippy` + `flutter analyze`)
3. Update documentation if adding features
4. Test on real hardware if possible

### Code Style
- **Rust**: Follow official Rust style guide, use `rustfmt`
- **Dart**: Follow official Dart style guide, use `dartfmt`
- **Commit Messages**: Conventional Commits format

### Testing Hardware Compatibility
If you have an ASUS laptop not yet tested:
1. Run hardware detection: `detect_hardware_capabilities()`
2. Test each feature module independently
3. Report results with model name and kernel version
4. Submit compatibility matrix update

---

## Known Limitations

- **x86_64 only**: ARM laptops not supported (though Linux kernel interfaces are arch-agnostic)
- **ASUS only**: Requires asus-nb-wmi driver (some ROG Ally models use different interface)
- **Anime Matrix**: Protocol may differ between model years, reverse engineering ongoing
- **Fan curves**: Not all models support hardware auto points
- **Undervolting**: Not implemented (safety concerns, may be added with explicit warnings)

---

**Last Updated**: 2026-04-08  
**Maintainers**: See `CONTRIBUTING.md`  
**License**: See `LICENSE` file
