# Data Model: Extended GPU Modes Support

**Feature**: Extended GPU Modes Support  
**Date**: 2026-04-13  
**Status**: Complete

## Overview

This document defines the data entities for GPU mode management, including GPU modes, hardware capabilities, and current status. All entities follow the existing project pattern and integrate with the configuration system.

## Entities

### GPU Mode

Represents a GPU power mode configuration.

**Attributes**:
- `mode_type`: enum - The type of GPU mode (Integrated, Hybrid, Ultimate, AsusMuxDgpu, AsusEgpu, Vfio)
- `is_supported`: boolean - Whether this mode is supported on current hardware
- `requires_reboot`: boolean - Whether switching to this mode requires system reboot
- `requires_logout`: boolean - Whether switching to this mode requires session logout

**Validation Rules**:
- `mode_type` must be one of the defined enum values
- `requires_reboot` and `requires_logout` cannot both be true for the same mode
- If `is_supported` is false, the mode should not be selectable in UI

**State Transitions**:
- Modes can be switched from any supported mode to any other supported mode
- Transitions requiring reboot/logout must show confirmation dialog
- Transition to unsupported mode should be prevented

**Relationships**:
- Referenced by GPU Configuration (supported_modes)
- Referenced by GPU Status (active_mode)

---

### GPU Configuration

Represents detected GPU hardware capabilities.

**Attributes**:
- `has_dgpu`: boolean - Whether discrete GPU is present
- `dgpu_vendor`: enum (Nvidia, AMD, None) - Vendor of discrete GPU
- `has_mux_switch`: boolean - Whether ASUS MUX switch is supported
- `supports_egpu`: boolean - Whether external GPU switching is supported
- `external_gpu_connected`: boolean - Whether external GPU is currently connected
- `vfio_available`: boolean - Whether VFIO kernel modules are available

**Validation Rules**:
- If `has_dgpu` is false, `dgpu_vendor` must be None
- If `external_gpu_connected` is true, `supports_egpu` must be true
- `vfio_available` can be true even if vfio modules not currently loaded (check for availability)

**Relationships**:
- Used to determine which GPU modes are supported
- Stored in hardware detection cache
- Updated on application startup and hardware change events

---

### GPU Status

Represents current GPU state.

**Attributes**:
- `active_mode`: GPU Mode - Currently active GPU mode
- `dgpu_enabled`: boolean - Whether discrete GPU is currently enabled
- `temperature`: float (optional) - GPU temperature in Celsius
- `utilization`: float (optional) - GPU utilization percentage (0-100)
- `power_state`: enum (Active, Idle, Suspended, Unknown) - Current power state

**Validation Rules**:
- `temperature` range: 0 to 120°C (safety bounds)
- `utilization` range: 0 to 100%
- `dgpu_enabled` must be consistent with `active_mode` (e.g., Integrated mode should have dgpu_enabled=false)

**Relationships**:
- References GPU Mode (active_mode)
- Updated by GPU monitoring service
- Displayed in UI status cards

---

## Configuration Integration

### User Configuration File

The GPU mode preference is stored in the existing configuration file at `~/.config/asus-linux-control/config.json`.

**New Fields**:
```json
{
  "gpu_mode": "Hybrid",
  "auto_gpu_switch": true,
  "gpu_mode_on_ac": "Hybrid",
  "gpu_mode_on_battery": "Integrated"
}
```

**Field Descriptions**:
- `gpu_mode`: Last selected GPU mode (string enum)
- `auto_gpu_switch`: Whether to automatically switch GPU mode based on power state
- `gpu_mode_on_ac`: GPU mode to use when AC power is connected
- `gpu_mode_on_battery`: GPU mode to use when on battery power

**Validation Rules**:
- `gpu_mode`, `gpu_mode_on_ac`, `gpu_mode_on_battery` must be valid mode names
- If `auto_gpu_switch` is false, `gpu_mode_on_ac` and `gpu_mode_on_battery` are ignored
- Configuration defaults to "Hybrid" if not specified

---

## FFI Data Structures

### Rust to Dart

The following data structures cross the FFI boundary and must be defined in `rust_core/src/api.rs` with `#[frb(sync)]` or `#[frb(async)]` annotations.

**GpuMode Enum** (Rust):
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[frb(sync)]
pub enum GpuMode {
    Integrated,
    Hybrid,
    Ultimate,
    AsusMuxDgpu,
    AsusEgpu,
    Vfio,
}
```

**GpuConfiguration Struct** (Rust):
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[frb(sync)]
pub enum GpuVendor {
    Nvidia,
    Amd,
}
```

**GpuStatus Struct** (Rust):
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[frb(sync)]
pub enum GpuPowerState {
    Active,
    Idle,
    Suspended,
    Unknown,
}
```

---

## Flutter Data Classes

### GPU Service Interface

**IGpuService** (Dart):
```dart
abstract class IGpuService {
  Future<GpuConfiguration> getCapabilities();
  Future<GpuStatus> getStatus();
  Future<void> setGpuMode(GpuMode mode);
  Future<List<GpuMode>> getSupportedModes();
}
```

**GpuMode Enum** (Dart):
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

**GpuConfiguration Data Class** (Dart):
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

**GpuStatus Data Class** (Dart):
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

## BLoC State Management

### GPU State

**GpuState** (Dart):
```dart
abstract class GpuState {}

class GpuInitial extends GpuState {}

class GpuLoading extends GpuState {}

class GpuLoaded extends GpuState {
  final GpuConfiguration configuration;
  final GpuStatus status;
  final List<GpuMode> supportedModes;
}

class GpuUnsupported extends GpuState {
  final String reason;
}

class GpuError extends GpuState {
  final String message;
}

class GpuRequiresReboot extends GpuState {
  final GpuMode targetMode;
  final String warning;
}
```

### GPU Events

**GpuEvent** (Dart):
```dart
abstract class GpuEvent {}

class LoadCapabilitiesEvent extends GpuEvent {}

class LoadStatusEvent extends GpuEvent {}

class SetModeRequestedEvent extends GpuEvent {
  final GpuMode mode;
}

class ConfirmRebootEvent extends GpuEvent {
  final GpuMode mode;
}
```

---

## Database/Storage Schema

### Configuration JSON Schema

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "type": "object",
  "properties": {
    "gpu_mode": {
      "type": "string",
      "enum": ["Integrated", "Hybrid", "Ultimate", "AsusMuxDgpu", "AsusEgpu", "Vfio"],
      "default": "Hybrid"
    },
    "auto_gpu_switch": {
      "type": "boolean",
      "default": false
    },
    "gpu_mode_on_ac": {
      "type": "string",
      "enum": ["Integrated", "Hybrid", "Ultimate", "AsusMuxDgpu", "AsusEgpu", "Vfio"],
      "default": "Hybrid"
    },
    "gpu_mode_on_battery": {
      "type": "string",
      "enum": ["Integrated", "Hybrid", "Ultimate", "AsusMuxDgpu", "AsusEgpu", "Vfio"],
      "default": "Integrated"
    }
  }
}
```

---

## Summary

The data model follows the existing project patterns:
- Rust structs with `serde::Serialize` for FFI
- Flutter data classes matching Rust structures
- BLoC pattern for state management
- JSON configuration for persistence
- Clear validation rules and relationships

All entities are designed to be independently testable and follow the Single Responsibility principle from the constitution.
