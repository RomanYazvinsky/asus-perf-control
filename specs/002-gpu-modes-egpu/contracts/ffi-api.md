# FFI API Contract: GPU Modes

**Feature**: Extended GPU Modes Support  
**Date**: 2026-04-13  
**Status**: Complete

## Overview

This document defines the FFI (Foreign Function Interface) contract between Rust backend and Flutter frontend for GPU mode management. All functions use `flutter_rust_bridge` for type-safe cross-language calls.

## API Functions

### GPU Capabilities Detection

#### get_gpu_capabilities()

**Purpose**: Detect and return GPU hardware capabilities

**Rust Signature**:
```rust
#[frb(sync)]
pub fn get_gpu_capabilities() -> GpuConfiguration
```

**Parameters**: None

**Returns**: `GpuConfiguration` struct

**Error Handling**: Returns GpuConfiguration with default values (all false) if detection fails

**Side Effects**: None (read-only)

---

### GPU Status

#### get_gpu_status()

**Purpose**: Get current GPU status

**Rust Signature**:
```rust
#[frb(sync)]
pub fn get_gpu_status() -> GpuStatus
```

**Parameters**: None

**Returns**: `GpuStatus` struct

**Error Handling**: Returns GpuStatus with default values if status cannot be read

**Side Effects**: None (read-only)

---

### GPU Mode Control

#### set_gpu_mode()

**Purpose**: Set GPU mode

**Rust Signature**:
```rust
#[frb(sync)]
pub fn set_gpu_mode(mode: GpuMode) -> Result<(), String>
```

**Parameters**:
- `mode`: `GpuMode` enum - Target GPU mode

**Returns**: `Result<(), String>` - Ok(()) on success, Err(String) with error message on failure

**Error Handling**:
- Returns error if mode is not supported on current hardware
- Returns error if permission denied (udev rules not installed)
- Returns error if hardware operation fails

**Side Effects**:
- Writes to sysfs interfaces
- May require logout or reboot (warning should be shown via UI)
- Updates configuration file

**Safety Checks**:
- Validates mode is supported before attempting switch
- Confirms with user before reboot-required modes (UI responsibility)

---

#### get_gpu_mode()

**Purpose**: Get current GPU mode

**Rust Signature**:
```rust
#[frb(sync)]
pub fn get_gpu_mode() -> GpuMode
```

**Parameters**: None

**Returns**: `GpuMode` enum

**Error Handling**: Returns `GpuMode::Hybrid` as default if cannot determine

**Side Effects**: None (read-only)

---

#### get_supported_gpu_modes()

**Purpose**: Get list of supported GPU modes for current hardware

**Rust Signature**:
```rust
#[frb(sync)]
pub fn get_supported_gpu_modes() -> Vec<GpuMode>
```

**Parameters**: None

**Returns**: `Vec<GpuMode>` - List of supported modes

**Error Handling**: Returns empty list if detection fails

**Side Effects**: None (read-only)

---

## Data Structures

### GpuMode Enum

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[frb(sync)]
pub enum GpuMode {
    Integrated,   // iGPU only, dGPU disabled
    Hybrid,       // dGPU-offload mode
    Ultimate,     // dGPU primary for rendering
    AsusMuxDgpu,  // MUX switch with dGPU primary (requires reboot)
    AsusEgpu,     // External GPU mode
    Vfio,         // GPU bound to vfio-pci for VM passthrough
}
```

**Flutter Mapping**:
```dart
enum GpuMode {
  integrated,
  hybrid,
  ultimate,
  asusMuxDgpu,
  asusEgpu,
  vfio,
}
```

---

### GpuConfiguration Struct

```rust
#[derive(Debug, Clone, serde::Serialize)]
#[frb(sync)]
pub struct GpuConfiguration {
    pub has_dgpu: bool,
    pub dgpu_vendor: Option<GpuVendor>,
    pub has_mux_switch: bool,
    pub supports_egpu: bool,
    pub external_gpu_connected: bool,
    pub vfio_available: bool,
}
```

**Flutter Mapping**:
```dart
class GpuConfiguration {
  final bool hasDgpu;
  final GpuVendor? dgpuVendor;
  final bool hasMuxSwitch;
  final bool supportsEgpu;
  final bool externalGpuConnected;
  final bool vfioAvailable;
}
```

---

### GpuVendor Enum

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[frb(sync)]
pub enum GpuVendor {
    Nvidia,
    Amd,
}
```

**Flutter Mapping**:
```dart
enum GpuVendor {
  nvidia,
  amd,
}
```

---

### GpuStatus Struct

```rust
#[derive(Debug, Clone, serde::Serialize)]
#[frb(sync)]
pub struct GpuStatus {
    pub active_mode: GpuMode,
    pub dgpu_enabled: bool,
    pub temperature: Option<f32>,
    pub utilization: Option<f32>,
    pub power_state: GpuPowerState,
}
```

**Flutter Mapping**:
```dart
class GpuStatus {
  final GpuMode activeMode;
  final bool dgpuEnabled;
  final double? temperature;
  final double? utilization;
  final GpuPowerState powerState;
}
```

---

### GpuPowerState Enum

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[frb(sync)]
pub enum GpuPowerState {
    Active,
    Idle,
    Suspended,
    Unknown,
}
```

**Flutter Mapping**:
```dart
enum GpuPowerState {
  active,
  idle,
  suspended,
  unknown,
}
```

---

## Error Messages

All error messages returned via `Result<T, String>` must be user-facing and actionable:

| Error Condition | Error Message |
|----------------|--------------|
| Permission denied | "Failed to set GPU mode: Permission denied. Run install.sh to set up udev rules." |
| Mode not supported | "GPU mode not supported on this hardware model." |
| Hardware not found | "GPU hardware not detected. Check that discrete GPU is present." |
| eGPU not connected | "External GPU not connected. Connect XG Mobile before enabling AsusEgpu mode." |
| vfio not available | "VFIO kernel modules not available. Install vfio-pci module for GPU passthrough." |
| MUX switch failed | "MUX switch operation failed. This may require a BIOS update." |
| Kernel too old | "Kernel version too old for GPU switching. Requires kernel 6.1 or newer." |
| Generic error | "Failed to set GPU mode: {specific reason}" |

---

## Code Generation

After any API signature change, run:

```bash
cd rust_core
flutter_rust_bridge_codegen --rust-input src/api.rs --dart-output ../lib/bridge_generated_bridge.dart
```

Or use the project's build script if available.

---

## Testing

### Integration Tests

Each FFI function must have an integration test in Flutter:

```dart
test('get_gpu_capabilities returns valid configuration', () async {
  final config = await RustLib.getGpuCapabilities();
  expect(config, isNotNull);
  // Additional assertions based on test hardware
});

test('set_gpu_mode succeeds for supported mode', () async {
  final result = await RustLib.setGpuMode(GpuMode.integrated);
  expect(result, isNull); // null = success
});

test('set_gpu_mode fails for unsupported mode', () async {
  final result = await RustLib.setGpuMode(GpuMode.asusEgpu);
  expect(result, isNotNull); // error message
});
```

---

## Versioning

This API contract is version 1.0. Breaking changes require:
1. Increment major version
2. Update this contract document
3. Update integration tests
4. Document migration path for users
