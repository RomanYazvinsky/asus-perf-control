use std::fs;
use std::path::Path;

use super::HardwareError;

const PLATFORM_PROFILE_PATH: &str = "/sys/firmware/acpi/platform_profile";

/// Read current platform profile and map to our mode: 0=Silent, 1=Balanced, 2=Turbo
pub fn read_thermal_policy() -> Result<i32, HardwareError> {
    if !Path::new(PLATFORM_PROFILE_PATH).exists() {
        return Err(HardwareError::ThermalPolicyNotFound);
    }
    let raw = fs::read_to_string(PLATFORM_PROFILE_PATH).map_err(|e| HardwareError::ReadError {
        path: PLATFORM_PROFILE_PATH.to_string(),
        source: e,
    })?;
    match raw.trim() {
        "low-power" => Ok(0),
        "balanced"  => Ok(1),
        "performance" => Ok(2),
        other => Err(HardwareError::ReadError {
            path: PLATFORM_PROFILE_PATH.to_string(),
            source: std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Unknown platform profile: {other}"),
            ),
        }),
    }
}

/// Write platform profile from our mode: 0=Silent, 1=Balanced, 2=Turbo
pub fn write_thermal_policy(mode: i32) -> Result<(), HardwareError> {
    if !Path::new(PLATFORM_PROFILE_PATH).exists() {
        return Err(HardwareError::ThermalPolicyNotFound);
    }
    let profile = match mode {
        0 => "low-power",
        1 => "balanced",
        2 => "performance",
        _ => return Err(HardwareError::ReadError {
            path: PLATFORM_PROFILE_PATH.to_string(),
            source: std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("Invalid mode: {mode}. Must be 0, 1, or 2"),
            ),
        }),
    };
    fs::write(PLATFORM_PROFILE_PATH, profile).map_err(|e| HardwareError::WriteError {
        path: PLATFORM_PROFILE_PATH.to_string(),
        source: e,
    })
}
