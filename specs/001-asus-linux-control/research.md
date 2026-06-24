# Technical Research: ASUS Linux Control Tool

**Feature**: 001-asus-linux-control  
**Date**: 2026-04-08  
**Status**: Complete

## Overview

This document consolidates technical research findings for implementing ASUS laptop hardware control on Linux. All decisions are based on comprehensive API research, reference implementations, and Linux kernel documentation.

---

## 1. Performance Mode Control

### Decision: Linux kernel sysfs interface via asus-nb-wmi driver

**Chosen Solution**:
- Path: `/sys/class/platform/asus-nb-wmi/throttle_thermal_policy`
- Values: 0 (Silent), 1 (Balanced), 2 (Turbo)
- Direct file I/O with Rust `std::fs`

**Rationale**:
- Kernel support since Linux 5.6 (Dec 2019) - widely available
- Direct sysfs access is fastest and most reliable
- No additional daemon dependencies
- Used by G-Helper (Windows) and AsusCtl (Linux) reference implementations

**Alternatives Considered**:
- **ACPI calls via libacpi**: Rejected - more complex, requires ACPI table parsing
- **Custom kernel module**: Rejected - asus-nb-wmi already provides interface
- **DBus daemon**: Rejected - unnecessary abstraction layer

**References**:
- Kernel docs: https://www.kernel.org/doc/Documentation/ABI/testing/sysfs-platform-asus-wmi
- Kernel patch: https://patchwork.kernel.org/project/platform-driver-x86/patch/20191215142634.13888-1-leonmaxx@gmail.com/

---

## 2. Hardware Monitoring (Sensors)

### Decision: libmedium Rust crate for hwmon abstraction

**Chosen Solution**:
- Library: `libmedium` v0.13+
- Abstracts `/sys/class/hwmon/hwmonX/` interfaces
- Provides type-safe sensor reading APIs

**Rationale**:
- Pure Rust, no C dependencies
- Handles hwmon device discovery and parsing automatically
- Type-safe temperature/fan/PWM APIs
- Active maintenance, good documentation

**Alternatives Considered**:
- **Direct sysfs parsing**: Rejected - reinventing wheel, error-prone hwmon discovery
- **lm-sensors bindings**: Rejected - requires C library, overkill for our needs
- **sysfs crate**: Rejected - too low-level, manual parsing required

**Implementation Details**:
```rust
// CPU temperature from coretemp or k10temp
// Fan speeds from asus hwmon
// PWM control for fan curves
```

**References**:
- Crate: https://crates.io/crates/libmedium
- Docs: https://docs.rs/libmedium/latest/libmedium/
- hwmon kernel spec: https://www.kernel.org/doc/Documentation/hwmon/sysfs-interface

---

## 3. Battery Management

### Decision: Dual approach - sysfs for charge control, D-Bus for monitoring

**Chosen Solution**:
- Charge limit: Direct sysfs write to `/sys/class/power_supply/BAT0/charge_control_end_threshold`
- Battery status: `upower_dbus` crate for real-time monitoring

**Rationale**:
- Charge control requires root/udev - sysfs is simplest
- UPower provides standardized battery monitoring across distributions
- D-Bus async API integrates well with tokio

**Alternatives Considered**:
- **Pure sysfs**: Rejected - battery monitoring requires polling, UPower is event-driven
- **Pure D-Bus**: Rejected - charge threshold write not exposed via UPower
- **ACPI battery interface**: Rejected - deprecated in favor of power_supply

**Implementation Details**:
```rust
// Charge limit: std::fs::write
// Battery monitoring: upower_dbus::UPowerProxy + zbus async
```

**References**:
- UPower spec: https://upower.freedesktop.org/docs/UPower.html
- upower_dbus: https://crates.io/crates/upower_dbus
- zbus: https://docs.rs/zbus/latest/zbus/

---

## 4. GPU Management

### Decision: nvml-wrapper for Nvidia, sysfs for AMD, asus-wmi for dGPU toggle

**Chosen Solution**:
- Nvidia: `nvml-wrapper` crate (safe bindings to libnvidia-ml.so)
- AMD: Direct sysfs (`/sys/class/drm/card0/device/pp_power_profile_mode`)
- dGPU control: `/sys/class/platform/asus-nb-wmi/dgpu_disable`

**Rationale**:
- NVML is official Nvidia API - most reliable for Nvidia GPUs
- AMD has no stable library API - sysfs is standard approach
- asus-wmi provides hardware MUX control on supported models

**Alternatives Considered**:
- **ROCm for AMD**: Rejected - heavyweight, focused on compute not monitoring
- **libdrm bindings**: Rejected - low-level, complex for simple monitoring
- **Unified GPU abstraction**: Rejected - Nvidia and AMD APIs too different

**Implementation Details**:
```rust
// Nvidia: Nvml::init() -> device.temperature() / utilization()
// AMD: fs::read_to_string for power profile
// dGPU: fs::write "0"/"1" to dgpu_disable
```

**References**:
- nvml-wrapper: https://github.com/rust-nvml/nvml-wrapper
- NVML docs: https://docs.nvidia.com/deploy/nvml-api/
- AMD sysfs: https://docs.kernel.org/gpu/amdgpu/thermal.html

---

## 5. Fan Control

### Decision: libmedium for PWM control with safety validation

**Chosen Solution**:
- Use libmedium's `WriteablePwmSensor` interface
- Manual mode: `pwmX_enable=1`, `pwmX=0-255`
- Auto mode: `pwm_auto_point` interfaces for curves (hardware-dependent)

**Rationale**:
- libmedium provides safe PWM abstraction
- Safety validation prevents hardware damage
- Supports both manual and automatic fan control

**Alternatives Considered**:
- **fancontrol daemon**: Rejected - external dependency, conflicts with our control
- **Direct PWM writes**: Rejected - unsafe without validation
- **Custom kernel module**: Rejected - unnecessary complexity

**Safety Requirements**:
- Minimum speed validation at high temperatures (>80°C → >30% PWM)
- Curve monotonicity checking
- Hardware PWM limit enforcement (0-255)

**References**:
- libmedium PWM: https://docs.rs/libmedium/latest/libmedium/sensors/sync_sensors/pwm/
- pwm-fan kernel docs: https://docs.kernel.org/hwmon/pwm-fan.html

---

## 6. Flutter-Rust Bridge Integration

### Decision: flutter_rust_bridge v2.0 with code generation

**Chosen Solution**:
- Library: `flutter_rust_bridge` v2.0
- Code generation for type-safe FFI
- Support for sync functions, async functions, and streams

**Rationale**:
- Official "Flutter Favorite" package
- Automatic code generation - no manual FFI
- Supports complex types (Result, Vec, custom structs)
- Stream support for real-time sensor updates

**Alternatives Considered**:
- **Manual FFI via dart:ffi**: Rejected - error-prone, no type safety
- **Platform channels**: Rejected - Flutter-specific, doesn't work for desktop
- **flutter_rust_bridge v1**: Rejected - v2 has better ergonomics

**Code Generation Workflow**:
```bash
flutter_rust_bridge_codegen \
  --rust-input rust_core/src/api.rs \
  --dart-output lib/bridge_generated.dart \
  --dart-decl-output lib/bridge_definitions.dart
```

**References**:
- GitHub: https://github.com/fzyzcjy/flutter_rust_bridge
- Docs: https://cjycode.com/flutter_rust_bridge/
- Tutorial: https://www.zaynetro.com/post/flutter-rust-bridge-2023

---

## 7. RGB & Anime Matrix Control

### Decision: hidapi for USB HID devices, sysfs for keyboard backlight

**Chosen Solution**:
- Keyboard backlight: `/sys/class/leds/asus::kbd_backlight/brightness` (0-3)
- Anime Matrix: `hidapi` crate (VID: 0x0b05, PID varies by model)
- Protocol: Reverse-engineered from Starlight and AsusCtl projects

**Rationale**:
- Keyboard backlight has standard sysfs interface
- Anime Matrix requires USB HID - no kernel driver
- hidapi is cross-platform, well-maintained

**Alternatives Considered**:
- **libusb**: Rejected - hidapi is higher-level, simpler API
- **Kernel driver for Anime Matrix**: Rejected - doesn't exist, would require upstreaming
- **AsusCtl as daemon**: Rejected - want standalone application

**Protocol Details**:
- Anime Matrix packet structure from Starlight research
- LED brightness: 0-255 per pixel
- Indexing: left-to-right, top-to-bottom

**References**:
- hidapi: https://docs.rs/hidapi/latest/hidapi/
- Starlight: https://github.com/vddCore/Starlight
- AsusCtl rog-aura: https://gitlab.com/asus-linux/asusctl/-/tree/main/rog-aura

---

## 8. Configuration Persistence

### Decision: JSON files in XDG Base Directory

**Chosen Solution**:
- Location: `~/.config/asus-linux-control/config.json`
- Format: JSON via serde_json
- Structure: Nested config objects (performance, battery, fan_curves, automation)

**Rationale**:
- XDG Base Directory spec compliance (Linux standard)
- JSON is human-readable for debugging
- serde provides type-safe serialization

**Alternatives Considered**:
- **TOML**: Rejected - JSON better for nested structures
- **Binary format**: Rejected - not user-editable
- **SQLite**: Rejected - overkill for configuration
- **System-wide config in /etc**: Rejected - per-user settings needed

**References**:
- XDG spec: https://specifications.freedesktop.org/basedir-spec/basedir-spec-latest.html
- serde_json: https://docs.rs/serde_json/

---

## 9. Permissions & Security

### Decision: udev rules + polkit policies for privilege escalation

**Chosen Solution**:
- udev rules: chmod sysfs files to 0666 on device hotplug
- polkit: Allow wheel group systemd unit management (for potential daemon)
- Installation script: Copy rules, reload udev

**Rationale**:
- Standard Linux privilege escalation mechanism
- Avoids running entire application as root
- Users control permission grant via installation

**Alternatives Considered**:
- **setuid binary**: Rejected - security risk, discouraged
- **Run as root**: Rejected - violates least privilege principle
- **pkexec for every operation**: Rejected - poor UX, password prompts

**Security Considerations**:
- Input validation before sysfs writes
- Sanitize file paths
- Verify numeric ranges (PWM 0-255, thermal policy 0-2)

**Installation Files**:
- `/etc/udev/rules.d/99-asus-control.rules`
- `/etc/polkit-1/rules.d/50-asus-control.rules`
- `install.sh` script for automated setup

---

## 10. Testing Strategy

### Decision: Rust unit/integration tests + Flutter widget tests

**Chosen Solution**:
- Rust unit tests: Each service module (`cargo test`)
- Rust integration tests: FFI bridge functionality
- Flutter widget tests: UI components
- Flutter integration tests: End-to-end user flows

**Rationale**:
- Rust tests verify backend logic without GUI
- FFI bridge tests catch type mismatches
- Flutter tests verify UI behavior

**Testing Challenges**:
- Hardware dependencies: Mock sysfs interfaces for CI
- Permission requirements: Skip privileged tests in CI, run locally

**References**:
- Rust testing: https://doc.rust-lang.org/book/ch11-00-testing.html
- Flutter testing: https://docs.flutter.dev/testing

---

## External References Summary

All technical decisions are backed by:
- ✅ Linux Kernel documentation (asus-wmi, hwmon, power_supply, drm)
- ✅ Rust crate documentation (libmedium, nvml-wrapper, hidapi, upower_dbus, zbus, flutter_rust_bridge)
- ✅ Reference implementations:
  - G-Helper (Windows, C#): https://github.com/seerge/g-helper
  - AsusCtl (Linux, Rust): https://gitlab.com/asus-linux/asusctl
  - Starlight (Anime Matrix): https://github.com/vddCore/Starlight
- ✅ Official API documentation (NVML, UPower, XDG)

---

## Risk Assessment

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| Kernel API changes | Low | High | Pin minimum kernel version (5.6+), document requirements |
| Hardware incompatibility | Medium | Medium | Model detection, graceful degradation for missing features |
| Permission issues | High | High | Clear installation instructions, helpful error messages |
| Fan control safety | Low | Critical | Multi-layer validation, conservative defaults |
| FFI bridge bugs | Low | High | Extensive integration testing, type safety via codegen |

---

## Conclusion

All technical unknowns have been resolved through comprehensive API research. Implementation can proceed directly to Phase 1 (data model design) and Phase 2 (task breakdown).

**Key Takeaways**:
1. Linux kernel interfaces (sysfs) are stable and well-documented
2. Rust ecosystem has mature libraries for all required functionality
3. flutter_rust_bridge provides robust FFI without manual bindings
4. Reference implementations (G-Helper, AsusCtl) validate approach
5. Permission model (udev + polkit) is standard and secure
