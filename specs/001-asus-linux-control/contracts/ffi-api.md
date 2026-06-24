# FFI API Contract: Flutter ↔ Rust Bridge

**Feature**: 001-asus-linux-control  
**Date**: 2026-04-08  
**Contract Type**: Foreign Function Interface (FFI)

## Overview

This contract defines the API surface exposed by the Rust backend to the Flutter frontend via `flutter_rust_bridge`. All functions must maintain backward compatibility once released.

**Bridge Type**: flutter_rust_bridge v2.0 with automatic code generation  
**Generated Files**:
- Dart: `lib/bridge_generated.dart`, `lib/bridge_definitions.dart`
- Rust: `rust_core/src/api.rs`

---

## Contract Principles

1. **Type Safety**: All parameters and return types must be serializable via flutter_rust_bridge
2. **Error Handling**: All fallible operations return `Result<T, String>` with descriptive error messages
3. **Immutability**: Input parameters are immutable; state changes only via explicit mutation functions
4. **Async/Sync Split**: I/O operations are async, pure computations are sync
5. **Versioning**: Breaking changes require major version bump

---

## Performance Mode API

### `set_performance_mode`
```rust
#[frb(sync)]
pub fn set_performance_mode(mode: i32) -> Result<(), String>
```

**Description**: Sets system thermal policy (performance mode)

**Parameters**:
- `mode`: integer (0=Silent, 1=Balanced, 2=Turbo)

**Returns**:
- `Ok(())` on success
- `Err(String)` with error message on failure

**Errors**:
- "Invalid mode: must be 0, 1, or 2"
- "ASUS WMI thermal policy interface not found. Ensure asus-nb-wmi driver is loaded."
- "Failed to set thermal policy: {io_error}. Check permissions."

**Side Effects**: Writes to `/sys/class/platform/asus-nb-wmi/throttle_thermal_policy`

**Requirements**: FR-001, FR-002

---

### `get_performance_mode`
```rust
#[frb(sync)]
pub fn get_performance_mode() -> Result<i32, String>
```

**Description**: Reads current thermal policy from hardware

**Returns**:
- `Ok(i32)` - Current mode (0, 1, or 2)
- `Err(String)` with error message on failure

**Errors**:
- "Failed to read thermal policy: {io_error}"
- "Invalid policy format: {parse_error}"

**Requirements**: FR-003

---

## Sensor Monitoring API

### `get_sensor_readings`
```rust
#[frb(sync)]
pub fn get_sensor_readings() -> Result<SensorReading, String>
```

**Description**: Retrieves current hardware sensor snapshot

**Returns**:
- `Ok(SensorReading)` - Struct with all available sensor data
- `Err(String)` with error message if sensors unavailable

**Data Structure**:
```rust
pub struct SensorReading {
    pub timestamp: i64,           // Unix epoch milliseconds
    pub cpu_temp: Option<f32>,    // Celsius, None if unavailable
    pub gpu_temp: Option<f32>,    // Celsius, None if unavailable
    pub fan_speeds: Vec<u32>,     // RPM, empty vec if no fans
    pub battery_percentage: Option<f32>,  // 0-100, None if unavailable
    pub battery_state: Option<String>,    // "Charging", "Discharging", "Full", etc.
}
```

**Errors**:
- "Failed to parse hwmons: {error}"
- "CPU temperature sensor not found"

**Requirements**: FR-005, FR-006, FR-007, FR-008, FR-009

---

### `monitor_sensors_stream`
```rust
pub fn monitor_sensors_stream(sink: StreamSink<SensorReading>)
```

**Description**: Streams sensor readings at 1 Hz to Flutter

**Parameters**:
- `sink`: StreamSink for pushing readings to Dart

**Behavior**:
- Spawns background task
- Emits `SensorReading` every 1 second
- Stops when sink is closed by Flutter

**Requirements**: FR-005 (real-time monitoring)

---

## Battery Management API

### `set_battery_charge_limit`
```rust
#[frb(sync)]
pub fn set_battery_charge_limit(limit: u8) -> Result<(), String>
```

**Description**: Sets maximum battery charge threshold

**Parameters**:
- `limit`: integer (0-100, percentage)

**Returns**:
- `Ok(())` on success
- `Err(String)` with error message on failure

**Errors**:
- "Charge limit must be between 0-100"
- "Failed to set charge limit: {io_error}. Check permissions or hardware support."

**Side Effects**: Writes to `/sys/class/power_supply/BAT0/charge_control_end_threshold`

**Requirements**: FR-010

---

### `get_battery_charge_limit`
```rust
#[frb(sync)]
pub fn get_battery_charge_limit() -> Result<u8, String>
```

**Description**: Reads current charge limit setting

**Returns**:
- `Ok(u8)` - Current limit (0-100)
- `Err(String)` with error message on failure

**Requirements**: FR-011

---

### `get_battery_status_dbus`
```rust
pub async fn get_battery_status_dbus() -> Result<(f64, String), String>
```

**Description**: Retrieves battery status via D-Bus (UPower)

**Returns**:
- `Ok((percentage, state))` - Tuple of (percentage 0-100, state string)
- `Err(String)` with error message on failure

**State Values**: "Charging", "Discharging", "Empty", "Fully Charged", "Unknown"

**Requirements**: FR-012, FR-013

---

## GPU Management API

### `set_dgpu_enabled`
```rust
#[frb(sync)]
pub fn set_dgpu_enabled(enabled: bool) -> Result<(), String>
```

**Description**: Enables or disables discrete GPU

**Parameters**:
- `enabled`: true to enable dGPU, false to disable

**Returns**:
- `Ok(())` on success
- `Err(String)` with error message on failure

**Errors**:
- "Failed to set dGPU state: {io_error}"

**Side Effects**: Writes to `/sys/class/platform/asus-nb-wmi/dgpu_disable`

**Requirements**: FR-014, FR-015

---

### `get_nvidia_info`
```rust
#[frb(sync)]
pub fn get_nvidia_info() -> Result<NvidiaGpuInfo, String>
```

**Description**: Retrieves Nvidia GPU statistics via NVML

**Returns**:
- `Ok(NvidiaGpuInfo)` - GPU stats struct
- `Err(String)` with error message if unavailable

**Data Structure**:
```rust
pub struct NvidiaGpuInfo {
    pub temperature: u32,         // Celsius
    pub utilization: u32,         // Percentage 0-100
    pub memory_used_mb: u64,      // Megabytes
    pub memory_total_mb: u64,     // Megabytes
}
```

**Errors**:
- "NVML not available. Nvidia drivers may not be installed."
- "Failed to get Nvidia device: {error}"

**Requirements**: FR-016, FR-017

---

### `get_amd_power_profile`
```rust
#[frb(sync)]
pub fn get_amd_power_profile() -> Result<String, String>
```

**Description**: Reads AMD GPU power profile (raw sysfs output)

**Returns**:
- `Ok(String)` - Power profile text
- `Err(String)` with error message on failure

**Requirements**: FR-018

---

## Fan Control API

### `set_fan_pwm_manual`
```rust
#[frb(sync)]
pub fn set_fan_pwm_manual(fan_index: u32, percent: f32) -> Result<(), String>
```

**Description**: Manually sets fan speed (PWM)

**Parameters**:
- `fan_index`: Fan number (0-2)
- `percent`: PWM percentage (0.0-100.0)

**Returns**:
- `Ok(())` on success
- `Err(String)` with error message on failure

**Errors**:
- "PWM percent must be 0-100"
- "Failed to set PWM to manual: {error}"
- "PWM device not found"

**Side Effects**: Writes to hwmon `pwmX_enable=1` and `pwmX={0-255}`

**Requirements**: FR-019, FR-020

---

### `set_fan_curve`
```rust
#[frb(sync)]
pub fn set_fan_curve(curve: FanCurve) -> Result<(), String>
```

**Description**: Applies custom fan curve (temperature → PWM mapping)

**Parameters**:
- `curve`: FanCurve struct with curve points

**Data Structure**:
```rust
pub struct FanCurve {
    pub name: String,
    pub points: Vec<(u8, u8)>,    // Vec of (temp_celsius, pwm_percent)
    pub min_safe_speed: u8,       // Minimum PWM %
    pub max_temp_threshold: u8,   // Temp where fan must be 100%
}
```

**Returns**:
- `Ok(())` on success
- `Err(String)` with error message on failure

**Errors**:
- "Fan curve failed safety validation"

**Validation**:
- Rejects curves with PWM < 30% when temp > 80°C

**Requirements**: FR-021, FR-022

---

## RGB & Anime Matrix API

### `set_keyboard_backlight`
```rust
#[frb(sync)]
pub fn set_keyboard_backlight(brightness: u8) -> Result<(), String>
```

**Description**: Sets keyboard backlight brightness

**Parameters**:
- `brightness`: integer (0-3)

**Returns**:
- `Ok(())` on success
- `Err(String)` with error message on failure

**Errors**:
- "Brightness must be 0-3"
- "Failed to set backlight: {io_error}"

**Side Effects**: Writes to `/sys/class/leds/asus::kbd_backlight/brightness`

**Requirements**: FR-023

---

### `set_anime_matrix_brightness`
```rust
#[frb(sync)]
pub fn set_anime_matrix_brightness(level: u8) -> Result<(), String>
```

**Description**: Sets Anime Matrix display brightness

**Parameters**:
- `level`: integer (0-255)

**Returns**:
- `Ok(())` on success
- `Err(String)` with error message on failure

**Errors**:
- "Anime Matrix device not found"
- "Failed to write to device: {error}"

**Side Effects**: Sends USB HID packet to Anime Matrix device

**Requirements**: FR-024

---

### `anime_matrix_display_image`
```rust
#[frb(sync)]
pub fn anime_matrix_display_image(width: u32, height: u32, data: Vec<u8>) -> Result<(), String>
```

**Description**: Displays custom image on Anime Matrix

**Parameters**:
- `width`: Image width in pixels
- `height`: Image height in pixels
- `data`: Flattened pixel array (brightness 0-255, row-major order)

**Returns**:
- `Ok(())` on success
- `Err(String)` with error message on failure

**Errors**:
- "Device not connected"
- "Invalid image dimensions"

**Requirements**: FR-025, FR-026

---

## Configuration API

### `save_config`
```rust
#[frb(sync)]
pub fn save_config(config: AppConfig) -> Result<(), String>
```

**Description**: Persists application configuration to disk

**Parameters**:
- `config`: AppConfig struct

**Data Structure**:
```rust
pub struct AppConfig {
    pub performance: PerformanceConfig,
    pub battery: BatteryConfig,
    pub fan_curves: Vec<FanCurve>,
    pub automation: AutomationConfig,
}

pub struct PerformanceConfig {
    pub default_mode: u8,
    pub ac_mode: u8,
    pub battery_mode: u8,
}

pub struct BatteryConfig {
    pub charge_limit: Option<u8>,
}

pub struct AutomationConfig {
    pub auto_performance_switching: bool,
    pub auto_gpu_switching: bool,
    pub auto_refresh_rate: bool,
    pub backlight_timeout_battery: Option<u32>,
}
```

**Returns**:
- `Ok(())` on success
- `Err(String)` with error message on failure

**Side Effects**: Writes to `~/.config/asus-linux-control/config.json`

**Requirements**: FR-027, FR-028

---

### `load_config`
```rust
#[frb(sync)]
pub fn load_config() -> Result<AppConfig, String>
```

**Description**: Loads application configuration from disk

**Returns**:
- `Ok(AppConfig)` on success
- `Err(String)` with error message on failure (returns default config)

**Requirements**: FR-029, FR-030

---

## Hardware Detection API

### `detect_hardware_capabilities`
```rust
#[frb(sync)]
pub fn detect_hardware_capabilities() -> Result<HardwareConfig, String>
```

**Description**: Probes system for available features

**Returns**:
- `Ok(HardwareConfig)` - Struct with capability flags
- `Err(String)` with error message on failure

**Data Structure**:
```rust
pub struct HardwareConfig {
    pub model_name: String,
    pub has_anime_matrix: bool,
    pub has_dgpu: bool,
    pub gpu_vendor: Option<String>,  // "Nvidia", "AMD", or None
    pub supported_refresh_rates: Vec<u32>,
    pub fan_count: u32,
    pub sensor_count: u32,
    pub supports_charge_limit: bool,
    pub supports_dgpu_toggle: bool,
    pub kernel_version: String,
}
```

**Requirements**: FR-031

---

## Permissions Validation API

### `check_permissions`
```rust
#[frb(sync)]
pub fn check_permissions() -> PermissionStatus
```

**Description**: Checks if user has necessary permissions for hardware control

**Returns**: PermissionStatus struct

**Data Structure**:
```rust
pub struct PermissionStatus {
    pub can_write_thermal_policy: bool,
    pub can_write_fan_control: bool,
    pub can_write_charge_limit: bool,
    pub can_access_usb_hid: bool,
    pub missing_permissions: Vec<String>,  // Human-readable list
}
```

**Requirements**: FR-032, FR-033

---

## Contract Versioning

**Current Version**: 1.0.0

### Breaking Changes Policy
- Major version bump required for:
  - Removing functions
  - Changing function signatures
  - Changing struct field types
  - Changing error semantics

### Non-Breaking Changes
- Minor version bump for:
  - Adding new functions
  - Adding optional struct fields
  - Adding new error messages

---

## Testing Requirements

All API functions must have:
1. **Unit tests** in Rust (`rust_core/tests/`)
2. **Integration tests** for FFI bridge
3. **Error case tests** for all documented error conditions
4. **Mock implementations** for CI environments without hardware

---

## Error Message Standards

All error messages must:
- Start with what failed (e.g., "Failed to set thermal policy")
- Include the underlying error if from I/O (e.g., "{io_error}")
- Provide actionable guidance (e.g., "Check permissions")
- Never expose internal implementation details
- Be suitable for display to end users

---

---

## CPU TDP & Temperature Limits API

### `TdpConfig` (Data Structure)

```rust
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TdpConfig {
    pub sustained_tdp_w: Option<i32>,   // PL1/SPL in watts; None if read failed
    pub boost_tdp_w: Option<i32>,       // PL2/SPPT in watts; None if read failed
    pub cpu_temp_limit_c: Option<i32>,  // Thermal ceiling in °C; None if read failed
    pub is_supported: bool,             // False on kernels < 5.17 or non-ASUS hardware
}
```

**Requirements**: FR-027, FR-028, FR-029, FR-030

---

### `get_tdp_config`

```rust
#[frb(sync)]
pub fn get_tdp_config() -> TdpConfig
```

**Description**: Reads current TDP limits and thermal ceiling from hardware. Returns `is_supported: false` if sysfs attributes are absent.

**Parameters**: None

**Returns**: `TdpConfig` struct (never fails — missing attributes produce `None` fields)

**Side Effects**: Reads from:
- `/sys/devices/platform/asus-nb-wmi/ppt_pl1_spl`
- `/sys/devices/platform/asus-nb-wmi/ppt_pl2_sppt`
- `/sys/devices/platform/asus-nb-wmi/ppt_cpu_limit`

**Requirements**: FR-030

---

### `set_tdp_config`

```rust
#[frb(sync)]
pub fn set_tdp_config(sustained_w: i32, boost_w: i32, temp_limit_c: i32) -> Result<(), String>
```

**Description**: Applies CPU TDP limits and thermal ceiling. Pass `-1` for any parameter to skip setting it.

**Parameters**:
- `sustained_w`: Sustained TDP in watts (5–54W), or -1 to skip
- `boost_w`: Boost TDP in watts (5–54W, must be >= sustained_w when both are != -1), or -1 to skip
- `temp_limit_c`: CPU thermal throttle ceiling in °C (60–100°C), or -1 to skip

**Returns**:
- `Ok(())` on success
- `Err(String)` with validation or I/O error

**Errors**:
- "TDP not supported on this hardware. Kernel 5.17+ with asus-nb-wmi required."
- "Sustained TDP must be 5–54W, got {n}W"
- "Boost TDP must be 5–54W, got {n}W"
- "Boost TDP must be >= Sustained TDP"
- "CPU temperature limit must be 60–100°C, got {n}°C"
- "Failed to set sustained TDP: {io_error}. Check udev rules."

**Side Effects**: Writes milliwatts (`watts * 1000`) to `ppt_pl1_spl` and `ppt_pl2_sppt`; writes Celsius directly to `ppt_cpu_limit`. TDP values are **volatile** — reset on reboot.

**Requirements**: FR-027, FR-028, FR-029, FR-031, FR-032

---

### `is_tdp_supported`

```rust
#[frb(sync)]
pub fn is_tdp_supported() -> bool
```

**Description**: Fast check whether TDP control sysfs attributes exist. Use to gate UI visibility before loading full config.

**Parameters**: None

**Returns**: `true` if `/sys/devices/platform/asus-nb-wmi/ppt_pl1_spl` exists

**Side Effects**: None (read-only filesystem check)

**Requirements**: FR-030, FR-031

---

## Performance Requirements

- **Sync functions**: Must complete in < 100ms
- **Sensor reading**: < 50ms per reading
- **Stream latency**: < 100ms from hardware read to Flutter sink
- **Config I/O**: < 200ms for save/load

---

## Deprecation Policy

When deprecating API functions:
1. Mark as `#[deprecated]` in Rust
2. Document migration path in deprecation message
3. Maintain for minimum 2 minor versions
4. Remove in next major version

**Example**:
```rust
#[deprecated(since = "1.2.0", note = "Use set_performance_mode_v2 instead")]
pub fn set_performance_mode(mode: i32) -> Result<(), String>
```
