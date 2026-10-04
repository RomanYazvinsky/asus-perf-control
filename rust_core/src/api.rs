// Hardware API used by the desktop UI
// Read-only sysfs access; privileged writes go through the D-Bus daemons

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
