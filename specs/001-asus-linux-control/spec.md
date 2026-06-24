# Feature Specification: ASUS Linux Control Tool

**Feature Branch**: `001-asus-linux-control`  
**Created**: 2026-04-08  
**Status**: Draft  
**Input**: User description: "ASUS Linux Control Tool - Flutter GUI with Rust backend for managing ASUS laptops (performance, fans, GPU, RGB, battery)"

## User Scenarios & Testing *(mandatory)*

<!--
  IMPORTANT: User stories should be PRIORITIZED as user journeys ordered by importance.
  Each user story/journey must be INDEPENDENTLY TESTABLE - meaning if you implement just ONE of them,
  you should still have a viable MVP (Minimum Viable Product) that delivers value.
  
  Assign priorities (P1, P2, P3, etc.) to each story, where P1 is the most critical.
  Think of each story as a standalone slice of functionality that can be:
  - Developed independently
  - Tested independently
  - Deployed independently
  - Demonstrated to users independently
-->

### User Story 1 - Control Performance Modes (Priority: P1)

As an ASUS laptop user, I want to quickly switch between Silent, Balanced, and Turbo performance modes to optimize my laptop's power consumption and performance based on my current activity (e.g., battery saving during travel, high performance for gaming).

**Why this priority**: Core functionality that directly impacts user experience. Performance modes control fan behavior, CPU/GPU power limits, and thermal management - the most frequently used feature. This is the foundation that all other features build upon.

**Independent Test**: Can be fully tested by launching the app, selecting different performance modes, and verifying system behavior changes (fan noise, thermal policy). Delivers immediate value by replacing manual BIOS adjustments or terminal commands.

**Acceptance Scenarios**:

1. **Given** the application is running on a supported ASUS laptop, **When** user selects "Silent" mode, **Then** system thermal policy changes to Silent (value 0) and UI reflects the selection
2. **Given** user is in "Balanced" mode, **When** user switches to "Turbo" mode, **Then** fans increase speed, thermal policy changes to Turbo (value 2), and change persists across app restarts
3. **Given** laptop is booted with a specific performance mode, **When** user opens the app, **Then** the UI correctly displays the current active mode
4. **Given** user attempts to change mode without proper permissions, **When** mode change is requested, **Then** system displays clear error message with permission instructions

---

### User Story 2 - Monitor System Health (Priority: P2)

As an ASUS laptop user, I want to see real-time CPU/GPU temperatures, fan speeds, and battery status in a dashboard so I can monitor my laptop's health and identify potential thermal or performance issues.

**Why this priority**: Essential monitoring capabilities that help users understand system behavior and validate that performance mode changes are working. Provides transparency and builds trust in the tool.

**Independent Test**: Can be fully tested by opening the dashboard and verifying that sensor readings update in real-time (every 1-2 seconds). Delivers value by consolidating system monitoring that would otherwise require multiple terminal commands or tools.

**Acceptance Scenarios**:

1. **Given** the dashboard is open, **When** system is under load, **Then** CPU/GPU temperatures update in real-time and display accurate values
2. **Given** laptop has multiple fans, **When** performance mode changes, **Then** all fan RPM readings update to reflect new fan curve
3. **Given** laptop is plugged into AC power, **When** user views battery status, **Then** charging status and current charge percentage are displayed accurately
4. **Given** sensors fail to read, **When** dashboard attempts to display values, **Then** user sees "N/A" or appropriate error indicator instead of stale data

---

### User Story 3 - Set Battery Charge Limit (Priority: P3)

As an ASUS laptop user who primarily uses AC power, I want to limit battery charge to 80% to extend battery lifespan by reducing charge cycles and stress on battery cells.

**Why this priority**: Important for battery longevity but not immediately critical. Many users keep laptops plugged in and benefit from charge limiting, but system still functions without this feature.

**Independent Test**: Can be fully tested by setting charge limit to 80%, plugging in laptop, and verifying that battery stops charging at the configured threshold. Delivers measurable value through extended battery lifespan.

**Acceptance Scenarios**:

1. **Given** battery is at 60% charge, **When** user sets limit to 80% and plugs in AC, **Then** battery charges to 80% and stops
2. **Given** charge limit is set to 80%, **When** user changes limit to 100%, **Then** battery resumes charging to full capacity
3. **Given** charge limit is active, **When** laptop reboots, **Then** charge limit persists and remains enforced
4. **Given** user enters invalid value (e.g., 120%), **When** attempting to set limit, **Then** system rejects input and displays validation error

---

### User Story 4 - Manage GPU Power Modes (Priority: P4)

As an ASUS laptop user with hybrid graphics, I want to switch between Eco (iGPU only), Standard (hybrid), and Ultimate (dGPU primary) modes to balance battery life and graphics performance.

**Why this priority**: Valuable for battery optimization and performance tuning, but requires understanding of GPU architecture. Lower priority than core performance modes.

**Independent Test**: Can be fully tested by switching GPU modes and verifying that discrete GPU is enabled/disabled via system power consumption and GPU detection tools.

**Acceptance Scenarios**:

1. **Given** laptop has hybrid graphics, **When** user selects "Eco" mode, **Then** discrete GPU is disabled and system uses integrated graphics only
2. **Given** user is in "Eco" mode on battery, **When** user plugs into AC power, **Then** system can optionally auto-switch to "Standard" mode if automation is enabled
3. **Given** GPU mode change requires session restart, **When** user changes mode, **Then** system warns user about restart requirement

---

### User Story 5 - Customize Fan Curves (Priority: P5)

As a power user, I want to create custom fan curves with temperature-to-speed mappings so I can fine-tune cooling behavior for specific workloads or noise preferences.

**Why this priority**: Advanced feature for enthusiasts. Most users are satisfied with built-in performance modes. Requires careful implementation to prevent hardware damage from inadequate cooling.

**Independent Test**: Can be fully tested by creating a custom fan curve, applying it, and measuring actual fan speeds at different temperatures match the configured curve.

**Acceptance Scenarios**:

1. **Given** user opens fan curve editor, **When** user sets fan speed to 50% at 60°C, **Then** when CPU reaches 60°C, fan runs at approximately 50% PWM
2. **Given** user creates unsafe curve (e.g., 0% at 90°C), **When** user attempts to apply, **Then** system rejects curve and displays safety warning
3. **Given** custom curve is active, **When** user switches to preset performance mode, **Then** custom curve is overridden by mode's default curve

---

### User Story 6 - Control RGB Lighting (Priority: P6)

As an ASUS laptop owner with RGB features, I want to control keyboard backlight, Anime Matrix display, and peripheral lighting to customize appearance and reduce distractions.

**Why this priority**: Nice-to-have cosmetic feature. Does not impact performance or system health. Appeals to enthusiasts but not essential for daily use.

**Independent Test**: Can be fully tested by adjusting backlight brightness, uploading images to Anime Matrix, and verifying visual changes on hardware.

**Acceptance Scenarios**:

1. **Given** laptop has RGB keyboard, **When** user adjusts brightness slider, **Then** keyboard backlight brightness changes immediately
2. **Given** laptop has Anime Matrix display, **When** user uploads custom image, **Then** image appears on the display within 2 seconds
3. **Given** RGB animations are playing, **When** battery reaches critical level, **Then** animations can be auto-disabled to save power

---

### User Story 7 - Control CPU TDP and Temperature Limits (Priority: P7)

As an ASUS laptop user, I want to set custom CPU TDP (Thermal Design Power) limits and maximum CPU temperature (thermal throttle temperature) to fine-tune the balance between CPU performance, heat output, and fan noise — similar to the power limits control in g-helper on Windows.

**Why this priority**: Advanced power user feature. Most users are satisfied with preset performance modes, but enthusiasts want per-watt control over sustained and boost power limits, as well as the temperature ceiling at which the CPU begins to throttle. Incorrect values can cause instability, so it requires careful validation.

**Independent Test**: Can be fully tested by setting custom TDP values, running a CPU stress test, and verifying that the CPU power consumption (via `turbostat` or `powertop`) and package temperature stay within the configured bounds.

**Acceptance Scenarios**:

1. **Given** the TDP control screen is open, **When** user sets Sustained TDP to 20W and Boost TDP to 35W, **Then** CPU respects these limits under load and CPU watts remain within configured range
2. **Given** TDP limits are set, **When** user sets CPU temperature limit to 85°C, **Then** CPU throttles before reaching 85°C and package temperature stays at or below configured ceiling
3. **Given** user enters out-of-range TDP value (e.g., 0W or exceeds hardware maximum), **When** attempting to apply, **Then** system rejects input with a clear validation error and shows safe range
4. **Given** custom TDP limits are applied, **When** user switches to a preset performance mode (Silent/Balanced/Turbo), **Then** TDP limits revert to mode defaults and UI reflects the change
5. **Given** laptop is on battery, **When** user opens TDP screen, **Then** system displays current active limits read from hardware

---

### User Story 8 - Manage Display Settings (Priority: P8)

As a user with high-refresh-rate display, I want to control screen refresh rate and display overdrive to balance smoothness, image quality, and battery life.

**Why this priority**: Useful but specific to displays with variable refresh rate. Lower priority as OS-level controls often suffice.

**Independent Test**: Can be fully tested by changing refresh rate settings and verifying actual display refresh rate via system tools.

**Acceptance Scenarios**:

1. **Given** display supports 60Hz and 120Hz, **When** user selects 60Hz, **Then** screen switches to 60Hz refresh rate
2. **Given** laptop is on battery, **When** automation is enabled, **Then** system auto-switches to 60Hz to save power
3. **Given** display overdrive is available, **When** user toggles overdrive, **Then** pixel response time behavior changes

### Edge Cases

- What happens when laptop model is not recognized or not fully supported?
- How does system handle when sysfs interfaces are missing (older kernels)?
- What happens when user lacks permissions to write to system files?
- How does system behave when hardware sensors fail or return invalid data?
- What happens when USB HID devices (Anime Matrix, mice) are disconnected during operation?
- How does system handle conflicting settings from other control tools (AsusCtl, TLP)?
- What happens when performance mode change fails mid-operation?
- How does system handle battery charge threshold on models that don't support it?
- What happens when GPU mode switch requires X11/Wayland session restart?
- How does system handle fan curve that exceeds hardware PWM limits?
- What happens when user sets TDP higher than hardware-supported maximum (e.g., beyond BIOS cap)?
- How does system handle platforms where ACPI/WMI TDP interface is absent or returns errors?
- What happens when TDP limits conflict with active performance mode preset?
- How does system behave when temperature limit is set below current CPU temperature?

## Clarifications

### Session 2026-04-08

- Q: Which state management pattern should be used for the Flutter frontend? → A: BLoC (flutter_bloc) - Structured reactive pattern with Streams, ideal for real-time sensor data and complex state management
- Q: Which charting library should be used for power/temperature graphs? → A: fl_chart - Open-source, highly customizable, supports interactive scatter points for fan curve editor, good performance for 1Hz real-time updates
- Q: Which Material Design version should be implemented? → A: Material Design 3 (M3) - Modern Google standard with dynamic color theming and improved desktop components (useMaterial3: true)
- Q: Which navigation pattern is appropriate for 7 screens on desktop? → A: NavigationRail - Vertical sidebar with icons optimized for desktop applications
- Q: Should the app support light and dark themes? → A: Dark theme only - Single dark theme optimized for technical monitoring applications, reduces eye strain during prolonged system monitoring

## Requirements *(mandatory)*

<!--
  ACTION REQUIRED: The content in this section represents placeholders.
  Fill them out with the right functional requirements.
-->

### Functional Requirements

#### Performance Management
- **FR-001**: System MUST read and write thermal policy via `/sys/class/platform/asus-nb-wmi/throttle_thermal_policy`
- **FR-002**: System MUST support Silent (0), Balanced (1), and Turbo (2) performance modes
- **FR-003**: System MUST persist selected performance mode across application restarts
- **FR-004**: System MUST detect current performance mode on application startup

#### Hardware Monitoring
- **FR-005**: System MUST read CPU and GPU temperatures from hwmon sysfs interfaces
- **FR-006**: System MUST read fan speeds (RPM) from hwmon fan inputs
- **FR-007**: System MUST update sensor readings at least once per second
- **FR-008**: System MUST handle missing or unavailable sensors gracefully
- **FR-009**: System MUST read battery charge level and status

#### Battery Management
- **FR-010**: System MUST allow setting battery charge threshold between 0-100%
- **FR-011**: System MUST write charge limit to `/sys/class/power_supply/BAT0/charge_control_end_threshold`
- **FR-012**: System MUST validate charge threshold values before applying
- **FR-013**: System MUST persist charge threshold across reboots

#### GPU Management
- **FR-014**: System MUST support Eco, Standard, and Ultimate GPU modes
- **FR-015**: System MUST control discrete GPU via `/sys/class/platform/asus-nb-wmi/dgpu_disable`
- **FR-016**: System MUST detect Nvidia and AMD GPU hardware
- **FR-017**: System MUST read GPU temperature and utilization for Nvidia GPUs via NVML
- **FR-018**: System MUST read AMD GPU metrics from sysfs drm interfaces

#### Fan Control
- **FR-019**: System MUST allow users to create custom fan curves with temperature-PWM mappings
- **FR-020**: System MUST validate fan curves for safety (minimum speeds at high temps)
- **FR-021**: System MUST write fan curves to hwmon PWM auto-point interfaces
- **FR-022**: System MUST provide preset fan curves for Silent, Balanced, Performance

#### RGB & Lighting
- **FR-023**: System MUST control keyboard backlight brightness via sysfs LED interface
- **FR-024**: System MUST communicate with Anime Matrix display via USB HID
- **FR-025**: System MUST support static images and animated content on Anime Matrix
- **FR-026**: System MUST detect and control ASUS peripherals via USB HID

#### CPU TDP & Temperature Limits
- **FR-027**: System MUST allow users to set CPU Sustained TDP (SPL/PL1) power limit in watts
- **FR-028**: System MUST allow users to set CPU Boost TDP (SPL2/SPPT/PL2) power limit in watts
- **FR-029**: System MUST allow users to set the CPU thermal throttle temperature ceiling in degrees Celsius
- **FR-030**: System MUST read current TDP limits from hardware on screen open
- **FR-031**: System MUST validate TDP values against hardware-supported minimum and maximum before applying
- **FR-032**: System MUST apply TDP limits via ACPI platform_profile or asus-nb-wmi WMI interface where available
- **FR-033**: System MUST reset TDP limits to performance-mode defaults when user switches to a preset mode
- **FR-034**: System MUST persist custom TDP settings in user configuration

#### Display Control
- **FR-035**: System MUST allow refresh rate selection for supported displays
- **FR-036**: System MUST toggle display overdrive via sysfs interface if available

#### Configuration & Profiles
- **FR-037**: System MUST save user preferences to configuration file
- **FR-038**: System MUST support multiple configuration profiles
- **FR-039**: System MUST allow import and export of configuration profiles

#### Permissions & Security
- **FR-040**: System MUST detect permission issues and display helpful error messages
- **FR-041**: System MUST validate all user input before writing to system interfaces
- **FR-042**: System MUST provide installation scripts for udev rules and polkit policies
- **FR-043**: System MUST log all system modifications for troubleshooting

#### Automation
- **FR-044**: System MUST support automatic performance mode switching based on AC/battery state
- **FR-045**: System MUST support automatic GPU mode switching based on power state
- **FR-046**: System MUST support automatic refresh rate adjustment based on power state
- **FR-047**: System MUST support keyboard backlight timeout on battery

#### GUI Frontend (Flutter)
- **FR-048**: Frontend MUST use BLoC (flutter_bloc) pattern for state management with separate blocs for each feature domain
- **FR-049**: Frontend MUST implement Material Design 3 (M3) with `useMaterial3: true` configuration
- **FR-050**: Frontend MUST use NavigationRail component for primary navigation between 8 main screens
- **FR-051**: Frontend MUST implement dark theme only, optimized for system monitoring and reduced eye strain
- **FR-052**: Frontend MUST use fl_chart library for real-time temperature, fan speed, and power consumption graphs
- **FR-053**: Fan curve editor MUST support interactive scatter plot with draggable points for temperature-PWM mapping
- **FR-054**: Real-time charts MUST update smoothly at 1Hz refresh rate without stuttering
- **FR-055**: All graphs MUST display axes labels, legends, and tooltips for data point inspection
- **FR-056**: Frontend MUST organize screens as: Home (dashboard), Monitoring, Battery, GPU, Fans, RGB, TDP/Power, Settings
- **FR-057**: TDP screen MUST provide numeric input fields with hardware min/max range hints for Sustained TDP, Boost TDP, and temperature limit

### Key Entities

- **Performance Profile**: Represents a complete system configuration including performance mode, GPU mode, fan curve, and power settings. Attributes: name, thermal_policy, gpu_mode, fan_curve_points, charge_limit, display_refresh_rate, sustained_tdp_w, boost_tdp_w, cpu_temp_limit_c.

- **Sensor Reading**: Represents real-time hardware monitoring data. Attributes: timestamp, cpu_temp, gpu_temp, fan_speeds (array), battery_percentage, battery_state.

- **Fan Curve**: Represents temperature-to-PWM mapping for fan control. Attributes: curve_points (array of temperature-speed pairs), min_safe_speed, max_temp_threshold.

- **Hardware Configuration**: Represents detected laptop capabilities. Attributes: model_name, has_anime_matrix, has_dgpu, supported_refresh_rates, fan_count, sensor_count.

- **Automation Rule**: Represents automated behavior trigger. Attributes: trigger_condition (AC/battery/temperature), target_setting, enabled_state.

## Success Criteria *(mandatory)*

<!--
  ACTION REQUIRED: Define measurable success criteria.
  These must be technology-agnostic and measurable.
-->

### Measurable Outcomes

- **SC-001**: Users can switch performance modes within 2 seconds with visual feedback
- **SC-002**: Sensor readings update with less than 100ms latency from hardware change
- **SC-003**: Application detects correct laptop model and capabilities on 90%+ of supported ASUS hardware
- **SC-004**: Users can create and apply custom fan curve in under 3 minutes
- **SC-005**: Battery charge limit persists through at least 10 reboot cycles
- **SC-006**: Application memory footprint remains under 150MB during normal operation
- **SC-007**: CPU usage stays under 5% when monitoring in background
- **SC-008**: 95% of users successfully complete their primary task (mode switching) on first attempt
- **SC-009**: Application startup time is under 2 seconds on modern hardware
- **SC-010**: Permission errors provide clear actionable instructions in under 5 words
- **SC-011**: TDP changes take effect within 1 second with visual confirmation
- **SC-012**: CPU temperature stays within ±2°C of configured limit under sustained load
- **SC-013**: TDP input validation rejects out-of-range values 100% of the time before hardware write

## Assumptions

- Target users are running Linux kernel 5.4+ with asus-nb-wmi driver support
- Users have basic Linux system administration knowledge (understand file permissions, can run terminal commands)
- Supported ASUS models include ROG Zephyrus, TUF, Strix, Flow, and similar gaming/performance laptops from 2020+
- Users are willing to install udev rules and polkit policies for proper permissions (one-time setup)
- System is running on x86_64 architecture (ARM/other architectures out of scope for v1)
- Users have Flutter runtime and Rust libraries properly installed
- GUI functionality requires X11 or Wayland display server
- Network connectivity not required for core functionality (offline-first)
- RGB peripheral support limited to USB HID devices with known protocols
- Advanced features (GPU overclocking, voltage control) require explicit user opt-in due to warranty/safety concerns
- Configuration files stored in user's home directory following XDG Base Directory specification
- Multi-user support assumes each user manages their own settings (no system-wide profiles)
- Anime Matrix support requires reverse-engineered USB protocol from AsusCtl/Starlight projects
- Display control assumes standard X11/Wayland compositor support for refresh rate changes
- Windows dual-boot scenarios may have conflicts with BIOS settings (documented limitation)
- TDP control assumes hardware supports ACPI `platform_profile` or ASUS WMI PPT (Package Power Tracking) interfaces (kernel 5.17+ for `asus_wmi` PPT support)
- Sustained TDP range assumed 5–54W and Boost TDP range assumed 5–54W for ROG/TUF; actual hardware limits read at runtime and used for validation
- CPU temperature limit control assumes `asus-nb-wmi` exposes `ppt_cpu_limit` or equivalent WMI attribute; models without this attribute will have the feature disabled with a clear message
- TDP settings are volatile (reset on reboot) unless a systemd service or startup script re-applies them; persistence via config re-apply on startup is in scope

---

## Technical Implementation Research

### Session 2026-04-08

This section provides comprehensive technical research for implementing each feature, including APIs, libraries, connection methods, code examples, and external references.

#### 1. Performance Mode Control (FR-001 to FR-004)

**Linux Kernel Interface:**
- **Path**: `/sys/class/platform/asus-nb-wmi/throttle_thermal_policy`
- **Values**: `0` (Silent/Default), `1` (Balanced/Overboost), `2` (Turbo/Silent)
- **Kernel Version**: 5.6+ (Dec 2019)
- **Documentation**: https://www.kernel.org/doc/Documentation/ABI/testing/sysfs-platform-asus-wmi

**Implementation:**

```rust
use std::fs;
use std::path::Path;

const THERMAL_POLICY_PATH: &str = "/sys/class/platform/asus-nb-wmi/throttle_thermal_policy";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThermalPolicy {
    Silent = 0,
    Balanced = 1,
    Turbo = 2,
}

pub struct PerformanceController {
    policy_path: String,
}

impl PerformanceController {
    pub fn new() -> Result<Self, String> {
        if !Path::new(THERMAL_POLICY_PATH).exists() {
            return Err("ASUS WMI thermal policy interface not found. Ensure asus-nb-wmi driver is loaded.".into());
        }
        Ok(Self {
            policy_path: THERMAL_POLICY_PATH.to_string(),
        })
    }

    pub fn set_policy(&self, policy: ThermalPolicy) -> Result<(), String> {
        fs::write(&self.policy_path, format!("{}", policy as u8))
            .map_err(|e| format!("Failed to set thermal policy: {}. Check permissions.", e))
    }

    pub fn get_policy(&self) -> Result<ThermalPolicy, String> {
        let content = fs::read_to_string(&self.policy_path)
            .map_err(|e| format!("Failed to read thermal policy: {}", e))?;
        
        match content.trim().parse::<u8>() {
            Ok(0) => Ok(ThermalPolicy::Silent),
            Ok(1) => Ok(ThermalPolicy::Balanced),
            Ok(2) => Ok(ThermalPolicy::Turbo),
            Ok(n) => Err(format!("Unknown thermal policy value: {}", n)),
            Err(e) => Err(format!("Invalid policy format: {}", e)),
        }
    }
}
```

**Data Structure:**
```rust
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PerformanceProfile {
    pub name: String,
    pub thermal_policy: u8,
    pub timestamp: i64,
}
```

**References:**
- Kernel patch: https://patchwork.kernel.org/project/platform-driver-x86/patch/20191215142634.13888-1-leonmaxx@gmail.com/
- G-Helper implementation: https://github.com/seerge/g-helper (C# reference)
- AsusCtl: https://gitlab.com/asus-linux/asusctl (Rust reference)

---

#### 2. Hardware Monitoring - Sensors (FR-005 to FR-009)

**Library**: `libmedium` v0.13+
- **Crate**: https://crates.io/crates/libmedium
- **Docs**: https://docs.rs/libmedium/latest/libmedium/

**hwmon sysfs Interface:**
- **Base Path**: `/sys/class/hwmon/hwmonX/`
- **Temperature**: `tempX_input` (millidegrees Celsius)
- **Fan Speed**: `fanX_input` (RPM)
- **PWM**: `pwmX` (0-255)

**Implementation:**

```rust
use libmedium::{parse_hwmons, sensors::sync_sensors::{temp::TempSensor, fan::FanSensor, SyncSensor}};

pub struct SensorMonitor {
    hwmons: Vec<libmedium::Hwmon>,
}

impl SensorMonitor {
    pub fn new() -> Result<Self, String> {
        let hwmons = parse_hwmons()
            .map_err(|e| format!("Failed to parse hwmons: {}", e))?;
        Ok(Self { hwmons })
    }

    pub fn get_cpu_temp(&self) -> Result<f32, String> {
        for hwmon in &self.hwmons {
            // Look for CPU-related sensor (coretemp, k10temp, etc.)
            if hwmon.name().contains("coretemp") || hwmon.name().contains("k10temp") {
                for (_, temp_sensor) in hwmon.temps() {
                    if temp_sensor.name().contains("Package") || temp_sensor.name().contains("Tctl") {
                        return temp_sensor.read_input()
                            .map(|t| t.as_degrees_celsius())
                            .map_err(|e| e.to_string());
                    }
                }
            }
        }
        Err("CPU temperature sensor not found".into())
    }

    pub fn get_fan_speeds(&self) -> Result<Vec<u32>, String> {
        let mut speeds = Vec::new();
        for hwmon in &self.hwmons {
            if hwmon.name().contains("asus") {
                for (_, fan_sensor) in hwmon.fans() {
                    if let Ok(rpm) = fan_sensor.read_input() {
                        speeds.push(rpm.as_rpm());
                    }
                }
            }
        }
        Ok(speeds)
    }
}
```

**Data Structure:**
```rust
#[derive(Debug, Clone, serde::Serialize)]
pub struct SensorReading {
    pub timestamp: i64,
    pub cpu_temp: Option<f32>,
    pub gpu_temp: Option<f32>,
    pub fan_speeds: Vec<u32>,
    pub battery_percentage: Option<f32>,
    pub battery_state: Option<String>,
}
```

**References:**
- hwmon kernel docs: https://www.kernel.org/doc/Documentation/hwmon/sysfs-interface
- libmedium examples: https://github.com/saronic-technologies/libmedium-rs

---

#### 3. Battery Management (FR-010 to FR-013)

**Sysfs Interface:**
- **Path**: `/sys/class/power_supply/BAT0/charge_control_end_threshold`
- **Value Range**: 0-100 (percentage)

**D-Bus Interface (Alternative):**
- **Library**: `upower_dbus` v0.3+
- **Service**: `org.freedesktop.UPower`
- **Docs**: https://docs.rs/upower_dbus/latest/upower_dbus/

**Implementation:**

```rust
use std::fs;
use upower_dbus::UPowerProxy;
use zbus::Connection;

pub struct BatteryController {
    charge_limit_path: String,
}

impl BatteryController {
    pub fn new() -> Self {
        Self {
            charge_limit_path: "/sys/class/power_supply/BAT0/charge_control_end_threshold".to_string(),
        }
    }

    pub fn set_charge_limit(&self, limit: u8) -> Result<(), String> {
        if limit > 100 {
            return Err("Charge limit must be between 0-100".into());
        }
        
        fs::write(&self.charge_limit_path, format!("{}", limit))
            .map_err(|e| format!("Failed to set charge limit: {}. Check permissions or hardware support.", e))
    }

    pub fn get_charge_limit(&self) -> Result<u8, String> {
        let content = fs::read_to_string(&self.charge_limit_path)
            .map_err(|e| format!("Failed to read charge limit: {}", e))?;
        content.trim().parse::<u8>()
            .map_err(|e| format!("Invalid charge limit value: {}", e))
    }

    pub async fn get_battery_status_dbus(&self) -> Result<(f64, String), String> {
        let connection = Connection::system().await
            .map_err(|e| format!("Failed to connect to system bus: {}", e))?;
        
        let upower = UPowerProxy::new(&connection).await
            .map_err(|e| format!("Failed to create UPower proxy: {}", e))?;
        
        let device = upower.get_display_device().await
            .map_err(|e| format!("Failed to get battery device: {}", e))?;
        
        let percentage = device.percentage().await
            .map_err(|e| format!("Failed to read percentage: {}", e))?;
        
        let state = device.state().await
            .map_err(|e| format!("Failed to read state: {}", e))?;
        
        let state_str = match state {
            1 => "Charging",
            2 => "Discharging",
            3 => "Empty",
            4 => "Fully Charged",
            _ => "Unknown",
        };
        
        Ok((percentage, state_str.to_string()))
    }
}
```

**Data Structure:**
```rust
#[derive(Debug, Clone, serde::Serialize)]
pub struct BatteryInfo {
    pub percentage: f64,
    pub state: String,
    pub charge_limit: Option<u8>,
    pub voltage_now: Option<f32>,
    pub current_now: Option<f32>,
}
```

**References:**
- UPower D-Bus spec: https://upower.freedesktop.org/docs/UPower.html
- zbus crate: https://docs.rs/zbus/latest/zbus/
- upower_dbus example: https://crates.io/crates/upower_dbus

---

#### 4. GPU Management (FR-014 to FR-018)

**ASUS WMI GPU Control:**
- **Path**: `/sys/class/platform/asus-nb-wmi/dgpu_disable`
- **Values**: `0` (Enable dGPU), `1` (Disable dGPU)

**Nvidia NVML:**
- **Library**: `nvml-wrapper` v0.10+
- **C Library**: `libnvidia-ml.so` (included with Nvidia drivers)
- **Docs**: https://docs.rs/nvml-wrapper/latest/nvml_wrapper/

**AMD GPU:**
- **Path**: `/sys/class/drm/card0/device/pp_od_clk_voltage`
- **Power Profile**: `/sys/class/drm/card0/device/pp_power_profile_mode`

**Implementation:**

```rust
use std::fs;
use nvml_wrapper::Nvml;
use nvml_wrapper::enum_wrappers::device::TemperatureSensor;

pub struct GpuController {
    dgpu_path: String,
    nvml: Option<Nvml>,
}

impl GpuController {
    pub fn new() -> Self {
        let nvml = Nvml::init().ok();
        Self {
            dgpu_path: "/sys/class/platform/asus-nb-wmi/dgpu_disable".to_string(),
            nvml,
        }
    }

    pub fn set_dgpu_enabled(&self, enabled: bool) -> Result<(), String> {
        let value = if enabled { "0" } else { "1" };
        fs::write(&self.dgpu_path, value)
            .map_err(|e| format!("Failed to set dGPU state: {}", e))
    }

    pub fn get_nvidia_info(&self) -> Result<NvidiaGpuInfo, String> {
        let nvml = self.nvml.as_ref()
            .ok_or("NVML not available. Nvidia drivers may not be installed.".to_string())?;
        
        let device = nvml.device_by_index(0)
            .map_err(|e| format!("Failed to get Nvidia device: {}", e))?;
        
        let temperature = device.temperature(TemperatureSensor::Gpu)
            .map_err(|e| format!("Failed to read GPU temperature: {}", e))?;
        
        let utilization = device.utilization_rates()
            .map_err(|e| format!("Failed to read GPU utilization: {}", e))?;
        
        let memory_info = device.memory_info()
            .map_err(|e| format!("Failed to read memory info: {}", e))?;
        
        Ok(NvidiaGpuInfo {
            temperature,
            utilization: utilization.gpu,
            memory_used_mb: memory_info.used / 1024 / 1024,
            memory_total_mb: memory_info.total / 1024 / 1024,
        })
    }

    pub fn get_amd_power_profile(&self) -> Result<String, String> {
        let content = fs::read_to_string("/sys/class/drm/card0/device/pp_power_profile_mode")
            .map_err(|e| format!("Failed to read AMD power profile: {}", e))?;
        Ok(content)
    }
}
```

**Data Structures:**
```rust
#[derive(Debug, Clone, serde::Serialize)]
pub struct NvidiaGpuInfo {
    pub temperature: u32,
    pub utilization: u32,
    pub memory_used_mb: u64,
    pub memory_total_mb: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GpuMode {
    Eco,       // iGPU only
    Standard,  // Hybrid
    Ultimate,  // dGPU primary
}
```

**References:**
- NVML wrapper: https://github.com/rust-nvml/nvml-wrapper
- NVML docs: https://docs.nvidia.com/deploy/nvml-api/
- AMD GPU thermal: https://docs.kernel.org/gpu/amdgpu/thermal.html
- AMD sysfs: https://wiki.archlinux.org/title/AMDGPU

---

#### 5. Fan Control & PWM (FR-019 to FR-022)

**hwmon PWM Interface:**
- **PWM Value**: `pwmX` (0-255, where 255 = 100%)
- **PWM Enable**: `pwmX_enable` (0=full speed, 1=manual, 2=automatic)
- **Fan RPM**: `fanX_input` (reads current RPM)

**Fan Curve Control (ASUS-specific):**
- **Per-profile curves**: Each of 3 thermal policies (Silent/Balanced/Turbo) has its own curve
- **CPU Fan**: `pwm1_auto_pointN_temp` + `pwm1_auto_pointN_pwm` (N=1-8 points)
- **GPU Fan**: `pwm2_auto_pointN_temp` + `pwm2_auto_pointN_pwm` (N=1-8 points)
- **ACPI Method**: Custom fan curves via WMI calls (kernel 5.19+)

**Kernel Support:**
- **Patch**: "asus-wmi: Add support for custom fan curves" (v15 merged in kernel 6.0+)
  - https://www.spinics.net/lists/kernel/msg4096977.html
  - https://www.spinics.net/lists/kernel/msg4092520.html (v11 discussion)
- **ACPI Methods**: 
  - `SFUN` method 0x07 for fan curve write
  - Device IDs: `FAN_CURVE_DEV_MID = 0x02`
  - PWM mask: `FAN_CURVE_PWM_MASK = 0x04`

**Reference Implementations:**
- **asusctl** fan control: https://gitlab.com/asus-linux/asusctl/-/blob/main/rog-dbus/src/ctrl_fan.rs
- **Arch Wiki** (Fan control guide): https://wiki.archlinux.org/title/Fan_speed_control#ASUS_laptops
- **lm-sensors** hwmon: https://hwmon.wiki.kernel.org/

**Implementation:**

```rust
use libmedium::{parse_hwmons, sensors::sync_sensors::pwm::WriteablePwmSensor, units::{Pwm, PwmEnable}};

pub struct FanController {
    hwmons: Vec<libmedium::Hwmon>,
}

impl FanController {
    pub fn new() -> Result<Self, String> {
        let hwmons = parse_hwmons()
            .map_err(|e| format!("Failed to parse hwmons: {}", e))?;
        Ok(Self { hwmons })
    }

    pub fn set_pwm_manual(&self, pwm_index: usize, percent: f32) -> Result<(), String> {
        if percent < 0.0 || percent > 100.0 {
            return Err("PWM percent must be 0-100".into());
        }

        for hwmon in &self.hwmons {
            if hwmon.name().contains("asus") {
                if let Some((_, pwm)) = hwmon.writeable_pwms().into_iter().nth(pwm_index) {
                    pwm.write_enable(PwmEnable::ManualControl)
                        .map_err(|e| format!("Failed to set PWM to manual: {}", e))?;
                    
                    let pwm_value = Pwm::try_from_percent(percent)
                        .map_err(|e| format!("Invalid PWM percent: {}", e))?;
                    
                    pwm.write_pwm(pwm_value)
                        .map_err(|e| format!("Failed to write PWM: {}", e))?;
                    
                    return Ok(());
                }
            }
        }
        Err("PWM device not found".into())
    }

    pub fn set_fan_curve(&self, curve: &FanCurve) -> Result<(), String> {
        // Safety validation
        if !self.validate_curve(curve) {
            return Err("Fan curve failed safety validation".into());
        }

        // Write curve points to pwm_auto_point interfaces
        for (temp, pwm) in &curve.points {
            // Implementation depends on specific hardware support
            // Some systems use pwm1_auto_point1_temp, pwm1_auto_point1_pwm, etc.
        }
        Ok(())
    }

    fn validate_curve(&self, curve: &FanCurve) -> bool {
        // Ensure minimum speed at high temps (safety check)
        for (temp, pwm) in &curve.points {
            if *temp > 80 && *pwm < 30 {
                return false; // Reject: too slow at high temp
            }
        }
        true
    }
}
```

**Data Structure:**
```rust
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FanCurve {
    pub name: String,
    pub points: Vec<(u8, u8)>, // (temperature°C, PWM%)
    pub min_safe_speed: u8,
    pub max_temp_threshold: u8,
}
```

**References:**
- hwmon PWM docs: https://docs.kernel.org/hwmon/pwm-fan.html
- libmedium PWM: https://docs.rs/libmedium/latest/libmedium/sensors/sync_sensors/pwm/
- Fan control guide: https://wiki.archlinux.org/title/Fan_speed_control

---

#### 6. Flutter-Rust Bridge Integration

**Library**: `flutter_rust_bridge` v2.0+
- **Crate**: https://crates.io/crates/flutter_rust_bridge
- **Docs**: https://cjycode.com/flutter_rust_bridge/

**Setup:**

1. Add to `Cargo.toml`:
```toml
[lib]
crate-type = ["cdylib", "staticlib"]

[dependencies]
flutter_rust_bridge = "2.0"
tokio = { version = "1", features = ["full"] }
```

2. Rust API (`api.rs`):
```rust
use flutter_rust_bridge::frb;

#[frb(sync)]
pub fn set_performance_mode(mode: i32) -> Result<(), String> {
    let policy = match mode {
        0 => ThermalPolicy::Silent,
        1 => ThermalPolicy::Balanced,
        2 => ThermalPolicy::Turbo,
        _ => return Err("Invalid mode".into()),
    };
    
    let controller = PerformanceController::new()?;
    controller.set_policy(policy)
}

#[frb(sync)]
pub fn get_sensor_readings() -> Result<SensorReading, String> {
    let monitor = SensorMonitor::new()?;
    
    Ok(SensorReading {
        timestamp: chrono::Utc::now().timestamp(),
        cpu_temp: monitor.get_cpu_temp().ok(),
        gpu_temp: None,
        fan_speeds: monitor.get_fan_speeds().unwrap_or_default(),
        battery_percentage: None,
        battery_state: None,
    })
}

// Stream for real-time updates
pub fn monitor_sensors_stream(sink: StreamSink<SensorReading>) {
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(1));
        
        loop {
            interval.tick().await;
            
            if let Ok(reading) = get_sensor_readings() {
                sink.add(reading);
            }
        }
    });
}
```

3. Flutter Usage (`main.dart`):
```dart
import 'bridge_generated.dart';

// Set performance mode
try {
  await setPerformanceMode(mode: 2); // Turbo
  print('Performance mode set');
} catch (e) {
  print('Error: $e');
}

// Stream sensor data
monitorSensorsStream().listen((reading) {
  print('CPU Temp: ${reading.cpuTemp}°C');
  print('Fan Speeds: ${reading.fanSpeeds}');
});
```

**Code Generation Command:**
```bash
flutter_rust_bridge_codegen \
  --rust-input rust_core/src/api.rs \
  --dart-output lib/bridge_generated.dart \
  --dart-decl-output lib/bridge_definitions.dart
```

**References:**
- FRB Quickstart: https://github.com/fzyzcjy/flutter_rust_bridge#-quickstart
- FRB Examples: https://github.com/fzyzcjy/flutter_rust_bridge/tree/master/frb_example
- Tutorial: https://www.zaynetro.com/post/flutter-rust-bridge-2023

---

#### 7. RGB & Anime Matrix (FR-023 to FR-026)

**Keyboard Backlight:**
- **Path**: `/sys/class/leds/asus::kbd_backlight/brightness`
- **Values**: 0-3

**Anime Matrix USB HID:**
- **Library**: `hidapi` v2.4+
- **Vendor ID**: `0x0b05` (ASUS)
- **Product ID**: Varies by model (e.g., `0x193b`)

**Implementation:**

```rust
use hidapi::{HidApi, HidDevice};

pub struct AnimeMatrixController {
    device: Option<HidDevice>,
}

impl AnimeMatrixController {
    pub fn new() -> Result<Self, String> {
        let api = HidApi::new()
            .map_err(|e| format!("Failed to init HID API: {}", e))?;
        
        // Try to find Anime Matrix device
        let device = api.open(0x0b05, 0x193b).ok();
        
        if device.is_none() {
            return Err("Anime Matrix device not found".into());
        }
        
        Ok(Self { device })
    }

    pub fn set_brightness(&mut self, level: u8) -> Result<(), String> {
        let device = self.device.as_ref()
            .ok_or("Device not connected")?;
        
        // Anime Matrix protocol (from AsusCtl/Starlight research)
        let packet = [0x5a, 0xba, level, 0x00];
        device.write(&packet)
            .map_err(|e| format!("Failed to write to device: {}", e))?;
        Ok(())
    }

    pub fn display_image(&mut self, width: usize, height: usize, data: &[u8]) -> Result<(), String> {
        let device = self.device.as_ref()
            .ok_or("Device not connected")?;
        
        // Send image data in chunks
        // Protocol based on Starlight project reverse engineering
        // Each LED brightness: 0-255
        // Indexed left-to-right, top-to-bottom
        
        Ok(())
    }
}
```

**References:**
- hidapi crate: https://docs.rs/hidapi/latest/hidapi/
- Starlight (Anime Matrix protocol): https://github.com/vddCore/Starlight
- AsusCtl Anime Matrix: https://gitlab.com/asus-linux/asusctl/-/tree/main/rog-aura

---

#### 8. Configuration Persistence

**Location**: `$XDG_CONFIG_HOME/asus-linux-control/` or `~/.config/asus-linux-control/`

**Data Structure:**
```rust
use serde::{Serialize, Deserialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
pub struct AppConfig {
    pub performance: PerformanceConfig,
    pub battery: BatteryConfig,
    pub fan_curves: Vec<FanCurve>,
    pub automation: AutomationConfig,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PerformanceConfig {
    pub default_mode: u8,
    pub ac_mode: u8,
    pub battery_mode: u8,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AutomationConfig {
    pub auto_performance_switching: bool,
    pub auto_gpu_switching: bool,
    pub auto_refresh_rate: bool,
    pub backlight_timeout_battery: Option<u32>,
}

pub fn save_config(config: &AppConfig) -> Result<(), String> {
    let config_dir = dirs::config_dir()
        .ok_or("Failed to get config directory")?
        .join("asus-linux-control");
    
    fs::create_dir_all(&config_dir)
        .map_err(|e| format!("Failed to create config dir: {}", e))?;
    
    let config_path = config_dir.join("config.json");
    let json = serde_json::to_string_pretty(config)
        .map_err(|e| format!("Failed to serialize config: {}", e))?;
    
    fs::write(config_path, json)
        .map_err(|e| format!("Failed to write config: {}", e))
}

pub fn load_config() -> Result<AppConfig, String> {
    let config_path = dirs::config_dir()
        .ok_or("Failed to get config directory")?
        .join("asus-linux-control/config.json");
    
    let json = fs::read_to_string(config_path)
        .map_err(|e| format!("Failed to read config: {}", e))?;
    
    serde_json::from_str(&json)
        .map_err(|e| format!("Failed to parse config: {}", e))
}
```

---

#### 9. Permissions & Security (FR-032 to FR-035)

**udev Rules** (`/etc/udev/rules.d/99-asus-control.rules`):
```bash
# ASUS WMI interfaces
KERNEL=="hwmon*", SUBSYSTEM=="hwmon", RUN+="/bin/chmod 0666 /sys/class/hwmon/%n/pwm*"
SUBSYSTEM=="platform", DRIVER=="asus-nb-wmi", RUN+="/bin/chmod 0666 /sys/class/platform/asus-nb-wmi/*"

# Battery charge control
SUBSYSTEM=="power_supply", KERNEL=="BAT*", RUN+="/bin/chmod 0666 /sys/class/power_supply/%k/charge_control_end_threshold"

# Keyboard backlight
SUBSYSTEM=="leds", KERNEL=="asus::kbd_backlight", RUN+="/bin/chmod 0666 /sys/class/leds/%k/brightness"
```

**Polkit Policy** (`/etc/polkit-1/rules.d/50-asus-control.rules`):
```javascript
polkit.addRule(function(action, subject) {
    if (action.id == "org.freedesktop.systemd1.manage-units" &&
        subject.isInGroup("wheel")) {
        return polkit.Result.YES;
    }
});
```

**Installation Script:**
```bash
#!/bin/bash
# install.sh

echo "Installing ASUS Linux Control..."

# Copy udev rules
sudo cp 99-asus-control.rules /etc/udev/rules.d/
sudo udevadm control --reload-rules
sudo udevadm trigger

# Copy polkit policy
sudo cp 50-asus-control.rules /etc/polkit-1/rules.d/

echo "Installation complete. Please reboot for changes to take effect."
```

---

#### 11. CPU TDP & Temperature Limits (FR-027 to FR-034)

**Linux Kernel / WMI Interfaces:**
- **Sustained TDP (SPL/PL1)**: `/sys/devices/platform/asus-nb-wmi/ppt_pl1_spl` (milliwatts)
  - Intel: PL1 (Power Limit 1), AMD: SPL (Sustained Power Limit)
  - Range: 5-250W (model dependent)
- **Boost TDP (SPL2/SPPT)**: `/sys/devices/platform/asus-nb-wmi/ppt_pl2_sppt` (milliwatts)
  - Intel: PL2, AMD: SPPT (Slow Package Power Tracking)
  - Range: 5-250W
- **Slow PPT (FPPT)**: `/sys/devices/platform/asus-nb-wmi/ppt_fppt` (milliwatts, AMD only)
- **APU SPPT**: `/sys/devices/platform/asus-nb-wmi/ppt_apu_sppt` (milliwatts, AMD APU systems)
- **Platform SPPT**: `/sys/devices/platform/asus-nb-wmi/ppt_platform_sppt` (milliwatts, AMD only)
- **CPU Temperature Limit**: `/sys/devices/platform/asus-nb-wmi/ppt_cpu_limit` (degrees Celsius)
- **Kernel Version**: 5.17+ for WMI PPT attributes; 6.5+ for full PPT suite; 6.10+ for all tunables

**⚠️ Important**: sysfs attributes marked DEPRECATED in kernel 6.10+. New `asus-armoury` driver is recommended:
- https://github.com/torvalds/linux/tree/master/drivers/platform/x86/asus-armoury.c
- All PPT attributes moving to `asus-armoury` driver

**Kernel Documentation:**
- sysfs-platform-asus-wmi ABI: https://www.kernel.org/doc/Documentation/ABI/testing/sysfs-platform-asus-wmi
- asus-wmi driver source: https://github.com/torvalds/linux/blob/master/drivers/platform/x86/asus-wmi.c
- asus-wmi header (WMI device IDs): https://github.com/torvalds/linux/blob/master/include/linux/platform_data/x86/asus-wmi.h

**Reference Implementations:**
- **g-helper** (C#, Windows): https://github.com/seerge/g-helper
  - File: `app/Mode/CPUBoost.cs` - TDP control logic
  - File: `app/Mode/GPUMode.cs` - GPU power management
- **asusctl** (Rust, Linux): https://gitlab.com/asus-linux/asusctl
  - Crate: `rog-platform` - Platform abstraction
  - File: `daemon/src/ctrl_platform.rs` - D-Bus service for platform control
  - File: `rog-platform/src/platform.rs` - WMI/sysfs interactions
- **asusctl** D-Bus API: https://asus-linux.org/wiki/asusctl-api/

**WMI Device IDs (from kernel header):**
```c
#define ASUS_WMI_DEVID_PPT_PL2_SPPT     0x001200A0
#define ASUS_WMI_DEVID_PPT_PL1_SPL      0x001200A3
#define ASUS_WMI_DEVID_PPT_APU_SPPT     0x001200B0
#define ASUS_WMI_DEVID_PPT_PLAT_SPPT    0x001200B1
#define ASUS_WMI_DEVID_PPT_PL3_FPPT     0x001200C1
#define ASUS_WMI_DEVID_NV_DYN_BOOST     0x001200C0
```

**Implementation:**

```rust
use std::fs;
use std::path::Path;

const PPT_PL1_PATH: &str = "/sys/devices/platform/asus-nb-wmi/ppt_pl1_spl";
const PPT_PL2_PATH: &str = "/sys/devices/platform/asus-nb-wmi/ppt_pl2_sppt";
const PPT_FPPT_PATH: &str = "/sys/devices/platform/asus-nb-wmi/ppt_fppt";
const PPT_CPU_TEMP_PATH: &str = "/sys/devices/platform/asus-nb-wmi/ppt_cpu_limit";

pub struct TdpController;

impl TdpController {
    /// Set sustained TDP (PL1/SPL) in watts. Writes milliwatts to sysfs.
    pub fn set_sustained_tdp(watts: u32) -> Result<(), String> {
        Self::validate_tdp(watts)?;
        let mw = watts * 1000;
        fs::write(PPT_PL1_PATH, mw.to_string())
            .map_err(|e| format!("Failed to set sustained TDP: {}. Check permissions.", e))
    }

    /// Set boost TDP (PL2/SPPT) in watts.
    pub fn set_boost_tdp(watts: u32) -> Result<(), String> {
        Self::validate_tdp(watts)?;
        let mw = watts * 1000;
        fs::write(PPT_PL2_PATH, mw.to_string())
            .map_err(|e| format!("Failed to set boost TDP: {}", e))
    }

    /// Set CPU thermal throttle temperature ceiling in degrees Celsius.
    pub fn set_cpu_temp_limit(celsius: u32) -> Result<(), String> {
        if celsius < 60 || celsius > 100 {
            return Err(format!("Temperature limit must be 60-100°C, got {}", celsius));
        }
        fs::write(PPT_CPU_TEMP_PATH, celsius.to_string())
            .map_err(|e| format!("Failed to set CPU temp limit: {}", e))
    }

    pub fn get_sustained_tdp() -> Result<u32, String> {
        Self::read_mw_as_watts(PPT_PL1_PATH)
    }

    pub fn get_boost_tdp() -> Result<u32, String> {
        Self::read_mw_as_watts(PPT_PL2_PATH)
    }

    pub fn get_cpu_temp_limit() -> Result<u32, String> {
        fs::read_to_string(PPT_CPU_TEMP_PATH)
            .map_err(|e| format!("Failed to read CPU temp limit: {}", e))?
            .trim()
            .parse::<u32>()
            .map_err(|e| format!("Invalid CPU temp limit value: {}", e))
    }

    pub fn is_supported() -> bool {
        Path::new(PPT_PL1_PATH).exists()
    }

    fn validate_tdp(watts: u32) -> Result<(), String> {
        if watts < 5 || watts > 54 {
            return Err(format!("TDP must be 5-54W, got {}W", watts));
        }
        Ok(())
    }

    fn read_mw_as_watts(path: &str) -> Result<u32, String> {
        let content = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read {}: {}", path, e))?;
        let mw = content.trim().parse::<u32>()
            .map_err(|e| format!("Invalid value: {}", e))?;
        Ok(mw / 1000)
    }
}
```

**Data Structure:**
```rust
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TdpConfig {
    pub sustained_tdp_w: Option<u32>,  // PL1/SPL in watts
    pub boost_tdp_w: Option<u32>,      // PL2/SPPT in watts
    pub cpu_temp_limit_c: Option<u32>, // Thermal ceiling in °C
}
```

**FFI API additions (`api.rs`):**
```rust
#[flutter_rust_bridge::frb(sync)]
pub fn get_tdp_config() -> Result<TdpConfig, String> {
    Ok(TdpConfig {
        sustained_tdp_w: TdpController::get_sustained_tdp().ok(),
        boost_tdp_w: TdpController::get_boost_tdp().ok(),
        cpu_temp_limit_c: TdpController::get_cpu_temp_limit().ok(),
    })
}

#[flutter_rust_bridge::frb(sync)]
pub fn set_tdp_config(sustained_w: i32, boost_w: i32, temp_limit_c: i32) -> Result<(), String> {
    if sustained_w > 0 {
        TdpController::set_sustained_tdp(sustained_w as u32)?;
    }
    if boost_w > 0 {
        TdpController::set_boost_tdp(boost_w as u32)?;
    }
    if temp_limit_c > 0 {
        TdpController::set_cpu_temp_limit(temp_limit_c as u32)?;
    }
    Ok(())
}

#[flutter_rust_bridge::frb(sync)]
pub fn is_tdp_supported() -> bool {
    TdpController::is_supported()
}
```

**Flutter UI sketch (TDP screen):**
```dart
// tdp_bloc events: LoadTdpConfig, SetTdpRequested(sustained, boost, tempLimit)
// tdp_bloc states: TdpLoading, TdpLoaded(config), TdpError, TdpUnsupported

// Widgets:
// - IntSlider "Sustained TDP" (5–54W, step 1W)
// - IntSlider "Boost TDP" (5–54W, step 1W)
// - IntSlider "CPU Temp Limit" (60–100°C, step 1°C)
// - "Apply" button → dispatches SetTdpRequested
// - "Reset to Mode Defaults" button
```

**References:**
- G-Helper TDP control: https://github.com/seerge/g-helper#power-limits
- AsusCtl PPT: https://gitlab.com/asus-linux/asusctl/-/blob/main/rog-control-center/src/pages/ppt.rs
- Kernel asus-wmi PPT attrs: https://www.kernel.org/doc/html/latest/admin-guide/laptops/asus-wmi.html
- Platform profile: https://docs.kernel.org/userspace-api/sysfs-platform_profile.html

---

#### 10. Complete Service Layer Architecture

```rust
// services/mod.rs
pub mod performance;
pub mod sensors;
pub mod battery;
pub mod gpu;
pub mod fans;
pub mod rgb;
pub mod tdp;

pub struct AsusControlService {
    performance: performance::PerformanceController,
    sensors: sensors::SensorMonitor,
    battery: battery::BatteryController,
    gpu: gpu::GpuController,
    fans: fans::FanController,
    tdp: tdp::TdpController,
}

impl AsusControlService {
    pub fn new() -> Result<Self, String> {
        Ok(Self {
            performance: performance::PerformanceController::new()?,
            sensors: sensors::SensorMonitor::new()?,
            battery: battery::BatteryController::new(),
            gpu: gpu::GpuController::new(),
            fans: fans::FanController::new()?,
        })
    }

    pub async fn start_monitoring(&self) -> Result<(), String> {
        // Start background monitoring tasks
        Ok(())
    }
}
```

**External References Summary:**
- Linux Kernel ASUS WMI: https://github.com/torvalds/linux/blob/master/include/linux/platform_data/x86/asus-wmi.h
- hwmon Documentation: https://www.kernel.org/doc/Documentation/hwmon/sysfs-interface
- G-Helper (Windows reference): https://github.com/seerge/g-helper
- AsusCtl (Linux reference): https://gitlab.com/asus-linux/asusctl
- Starlight (Anime Matrix): https://github.com/vddCore/Starlight
- Flutter Rust Bridge: https://github.com/fzyzcjy/flutter_rust_bridge
- NVML Rust Wrapper: https://github.com/rust-nvml/nvml-wrapper
- AMD GPU Docs: https://docs.kernel.org/gpu/amdgpu/thermal.html
