// Hardware API used by the desktop UI
// Synchronous sysfs reads and writes; privileged writes go through D-Bus daemons

use crate::services::temp;
pub use crate::services::steam_input::{SteamInputEvent, SteamInputStatus, UiInputCommand};
use serde::{Deserialize, Serialize};

/// Sensor reading snapshot from hardware
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SensorReading {
    pub timestamp: i64,
    pub cpu_temp: Option<f32>,
    pub gpu_temp: Option<f32>,
    pub fan_speeds: Vec<u32>,
    pub battery_percentage: Option<f32>,
    pub battery_state: Option<String>,
}

/// Performance profile configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceProfile {
    pub name: String,
    pub thermal_policy: i32,
    pub gpu_mode: Option<String>,
    pub fan_curve_id: Option<String>,
    pub charge_limit: Option<i32>,
    pub display_refresh_rate: Option<i32>,
}

/// Fan curve definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FanCurve {
    pub id: String,
    pub name: String,
    pub curve_points: Vec<(i32, i32)>,
    pub min_safe_speed: i32,
    pub max_temp_threshold: i32,
    pub is_preset: bool,
}

/// Battery status information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatteryStatus {
    pub percentage: f32,
    pub state: String,
    pub charge_limit: Option<i32>,
}

// ============================================================================
// Performance Mode API (User Story 1 - MVP)
// ============================================================================

/// Set system thermal policy (performance mode)
///
/// # Arguments
/// * `mode` - 0 = Silent, 1 = Balanced, 2 = Turbo
///
/// # Returns
/// * `Ok(())` on success
/// * `Err(String)` with error message on failure
pub fn set_performance_mode(mode: i32) -> Result<(), String> {
    use crate::services::performance::PerformanceController;

    let controller = PerformanceController::new();
    controller.write_thermal_policy(mode)
}

/// Get current thermal policy from hardware
///
/// # Returns
/// * `Ok(i32)` - Current mode (0, 1, or 2)
/// * `Err(String)` with error message on failure
pub fn get_performance_mode() -> Result<i32, String> {
    use crate::services::performance::PerformanceController;

    let controller = PerformanceController::new();
    controller.read_thermal_policy()
}

// ============================================================================
// Sensor Monitoring API (User Story 2)
// ============================================================================

/// Get current sensor readings
pub fn get_sensor_readings() -> Result<SensorReading, String> {
    let cpu_temp = temp::read_cpu_temp().ok().map(|t| t as f32);
    let gpu_temp = temp::read_gpu_temp().ok().map(|t| t as f32);

    Ok(SensorReading {
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as i64,
        cpu_temp,
        gpu_temp,
        fan_speeds: vec![],
        battery_percentage: None,
        battery_state: None,
    })
}

// ============================================================================
// Battery Management API (User Story 3)
// ============================================================================

/// Set battery charge limit (0-100, 0 = no limit)
pub fn set_battery_charge_limit(limit: i32) -> Result<(), String> {
    use crate::services::battery::BatteryController;

    let controller = BatteryController::new()
        .map_err(|e| format!("Failed to initialize battery controller: {}", e))?;

    controller
        .write_charge_limit(limit)
        .map_err(|e| format!("Failed to set charge limit: {}", e))
}

/// Get current battery charge limit (0-100, 0 = no limit)
pub fn get_battery_charge_limit() -> Result<i32, String> {
    use crate::services::battery::BatteryController;

    let controller = BatteryController::new()
        .map_err(|e| format!("Failed to initialize battery controller: {}", e))?;

    controller
        .read_charge_limit()
        .map_err(|e| format!("Failed to read charge limit: {}", e))
}

// ============================================================================
// CPU TDP & Temperature Limits API (User Story 7 - Phase 3 / Highest Priority)
// ============================================================================

/// TDP and CPU temperature limit configuration snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TdpConfig {
    /// Sustained TDP in watts (PL1/SPL); None if read failed or unsupported
    pub sustained_tdp_w: Option<i32>,
    /// Boost TDP in watts (PL2/SPPT); None if read failed or unsupported
    pub boost_tdp_w: Option<i32>,
    /// CPU thermal throttle ceiling in °C; None if read failed or unsupported
    pub cpu_temp_limit_c: Option<i32>,
    /// True if asus-nb-wmi PPT sysfs attributes exist (kernel 5.17+)
    pub is_supported: bool,
}

/// Read current TDP limits and CPU thermal ceiling from hardware.
/// Never fails — missing attributes produce None fields and is_supported=false.
pub fn get_tdp_config() -> TdpConfig {
    use crate::services::tdp::TdpController;

    if !TdpController::is_supported() {
        return TdpConfig {
            sustained_tdp_w: None,
            boost_tdp_w: None,
            cpu_temp_limit_c: None,
            is_supported: false,
        };
    }

    let controller = TdpController::new();
    TdpConfig {
        sustained_tdp_w: controller.get_sustained_tdp().ok().map(|v| v as i32),
        boost_tdp_w: controller.get_boost_tdp().ok().map(|v| v as i32),
        cpu_temp_limit_c: controller.get_cpu_temp_limit().ok().map(|v| v as i32),
        is_supported: true,
    }
}

/// Apply CPU TDP limits and thermal ceiling.
/// Pass -1 for any parameter to skip setting it.
/// sustained_w: 5–54W, boost_w: 5–54W (must be >= sustained_w), temp_limit_c: 60–100°C
pub fn set_tdp_config(sustained_w: i32, boost_w: i32, temp_limit_c: i32) -> Result<(), String> {
    use crate::services::tdp::TdpController;

    if !TdpController::is_supported() {
        return Err(
            "TDP not supported on this hardware. Kernel 5.17+ with asus-nb-wmi required."
                .to_string(),
        );
    }

    // Validate that parameters are either -1 (skip) or non-negative
    if sustained_w < -1 {
        return Err(format!(
            "Invalid sustained_w: {sustained_w}. Use -1 to skip or a value >= 0."
        ));
    }
    if boost_w < -1 {
        return Err(format!(
            "Invalid boost_w: {boost_w}. Use -1 to skip or a value >= 0."
        ));
    }
    if temp_limit_c < -1 {
        return Err(format!(
            "Invalid temp_limit_c: {temp_limit_c}. Use -1 to skip or a value >= 0."
        ));
    }

    let controller = TdpController::new();

    // Validate ranges using internal validation (without writing) - we need to check BEFORE any writes
    // to avoid partial state corruption. TdpController methods write immediately, so we duplicate validation.
    if sustained_w != -1 && (sustained_w < 5 || sustained_w > 54) {
        return Err(format!("Sustained TDP must be 5–54W, got {sustained_w}W"));
    }
    if boost_w != -1 && (boost_w < 5 || boost_w > 54) {
        return Err(format!("Boost TDP must be 5–54W, got {boost_w}W"));
    }
    if temp_limit_c != -1 && (temp_limit_c < 60 || temp_limit_c > 100) {
        return Err(format!(
            "CPU temperature limit must be 60–100°C, got {temp_limit_c}°C"
        ));
    }

    // Cross-field validation: boost must be >= sustained (when both are set)
    if sustained_w != -1 && boost_w != -1 && boost_w < sustained_w {
        return Err("Boost TDP must be >= Sustained TDP".to_string());
    }

    // All validations passed - now apply changes
    if sustained_w != -1 {
        controller.set_sustained_tdp(sustained_w as u32)?;
    }
    if boost_w != -1 {
        controller.set_boost_tdp(boost_w as u32)?;
    }
    if temp_limit_c != -1 {
        controller.set_cpu_temp_limit(temp_limit_c as u32)?;
    }

    Ok(())
}

/// Fast check whether TDP control sysfs attributes exist.
/// Use to gate UI visibility before loading full config.
pub fn is_tdp_supported() -> bool {
    use crate::services::tdp::TdpController;
    TdpController::is_supported()
}

// ============================================================================
// Fan Control API (Fan & Fan Curve Control)
// ============================================================================

/// Fan capabilities discovered from hardware
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FanCapabilities {
    pub fans: Vec<FanInfoDto>,
}

/// Fan information for FFI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FanInfoDto {
    pub id: String, // "cpu", "gpu"
    pub label: String,
    pub has_rpm: bool,
    pub has_curve: bool,
}

/// Fan RPM reading
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FanReading {
    pub id: String,
    pub rpm: Option<u32>,
}

/// Get fan capabilities (discovered fans and their features)
pub fn get_fan_capabilities() -> FanCapabilities {
    use crate::services::fan;

    let discovered = fan::discover_fans();
    let fans: Vec<FanInfoDto> = discovered
        .iter()
        .map(|f| FanInfoDto {
            id: match f.fan_id {
                fan::FanId::CpuFan => "cpu".to_string(),
                fan::FanId::GpuFan => "gpu".to_string(),
            },
            label: f.label.clone(),
            has_rpm: true,   // All discovered fans have RPM
            has_curve: true, // All discovered fans have curve control
        })
        .collect();

    FanCapabilities { fans }
}

/// Get current RPM readings for all fans
pub fn get_fan_rpms() -> Vec<FanReading> {
    use crate::services::fan;

    fan::read_all_fan_rpms()
        .into_iter()
        .map(|(fan_id, rpm)| FanReading {
            id: match fan_id {
                fan::FanId::CpuFan => "cpu".to_string(),
                fan::FanId::GpuFan => "gpu".to_string(),
            },
            rpm,
        })
        .collect()
}

/// Get CPU temperature in Celsius
pub fn get_cpu_temp() -> i32 {
    use crate::services::temp;

    match temp::read_cpu_temp() {
        Ok(temp) => temp,
        Err(_) => -1, // Return -1 on error
    }
}

/// Get GPU temperature in Celsius
pub fn get_gpu_temp() -> i32 {
    use crate::services::temp;

    match temp::read_gpu_temp() {
        Ok(temp) => temp,
        Err(_) => -1, // Return -1 on error
    }
}

// Placeholder functions for other user stories (to be implemented later)

pub fn get_steam_input_status() -> SteamInputStatus {
    crate::services::steam_input::SteamInputPoller::new_stub().status()
}

pub fn poll_steam_input_events() -> Vec<SteamInputEvent> {
    let mut poller = crate::services::steam_input::SteamInputPoller::new_stub();
    poller.poll()
}
