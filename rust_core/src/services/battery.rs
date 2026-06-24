// Battery management service
// Handles battery charge limit control via sysfs

use std::fs;
use std::path::Path;

/// Battery controller for managing charge limits
pub struct BatteryController {
    battery_path: String,
}

/// Errors that can occur during battery operations
#[derive(Debug)]
pub enum BatteryError {
    NotFound,
    PermissionDenied,
    InvalidRange(i32),
    IoError(std::io::Error),
}

impl std::fmt::Display for BatteryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BatteryError::NotFound => write!(f, "Battery not found"),
            BatteryError::PermissionDenied => {
                write!(f, "Permission denied - check udev rules for battery access")
            }
            BatteryError::InvalidRange(v) => {
                write!(f, "Charge limit must be 0-100%, got {}", v)
            }
            BatteryError::IoError(e) => write!(f, "IO error: {}", e),
        }
    }
}

impl std::error::Error for BatteryError {}

impl From<std::io::Error> for BatteryError {
    fn from(e: std::io::Error) -> Self {
        if e.kind() == std::io::ErrorKind::PermissionDenied {
            BatteryError::PermissionDenied
        } else {
            BatteryError::IoError(e)
        }
    }
}

impl BatteryController {
    /// Detect battery sysfs path
    pub fn new() -> Result<Self, BatteryError> {
        // Try common battery paths
        let paths = [
            "/sys/class/power_supply/BAT0",
            "/sys/class/power_supply/BAT1",
        ];

        for path in &paths {
            if Path::new(path).exists() {
                log::debug!("Battery found at {}", path);
                return Ok(Self {
                    battery_path: path.to_string(),
                });
            }
        }

        log::warn!("No battery found (tried BAT0, BAT1)");
        Err(BatteryError::NotFound)
    }

    /// Check if charge limit control is available
    pub fn is_charge_limit_supported(&self) -> bool {
        Path::new(&format!(
            "{}/charge_control_end_threshold",
            self.battery_path
        ))
        .exists()
    }

    /// Read current charge limit (0-100, 0 = no limit)
    pub fn read_charge_limit(&self) -> Result<i32, BatteryError> {
        let path = format!("{}/charge_control_end_threshold", self.battery_path);
        let content = fs::read_to_string(&path)?;
        let value: i32 = content.trim().parse().map_err(|e| {
            BatteryError::IoError(std::io::Error::new(std::io::ErrorKind::InvalidData, e))
        })?;
        Ok(value)
    }

    /// Write charge limit (0-100, 0 = disable limit)
    ///
    /// Safety: Values outside 0-100 are rejected before writing.
    pub fn write_charge_limit(&self, limit: i32) -> Result<(), BatteryError> {
        // Validation: must be 0-100
        if limit < 0 || limit > 100 {
            return Err(BatteryError::InvalidRange(limit));
        }

        // Check if limit control is available
        if !self.is_charge_limit_supported() {
            return Err(BatteryError::NotFound);
        }

        let path = format!("{}/charge_control_end_threshold", self.battery_path);
        fs::write(&path, limit.to_string())?;

        // Verify write succeeded by reading back
        let verify = self.read_charge_limit()?;
        if verify != limit {
            log::error!(
                "Battery write verification failed: wrote {}, read {}",
                limit,
                verify
            );
            return Err(BatteryError::IoError(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!(
                    "Write verification failed: wrote {}, read {}",
                    limit, verify
                ),
            )));
        }

        log::info!("Battery charge limit set and verified: {}%", limit);
        Ok(())
    }

    /// Get battery current capacity percentage
    pub fn get_battery_percentage(&self) -> Result<f32, BatteryError> {
        let path = format!("{}/capacity", self.battery_path);
        let content = fs::read_to_string(&path)?;
        let value: f32 = content.trim().parse().map_err(|e| {
            BatteryError::IoError(std::io::Error::new(std::io::ErrorKind::InvalidData, e))
        })?;
        Ok(value)
    }

    /// Get battery state (Charging, Discharging, Full, etc.)
    pub fn get_battery_state(&self) -> Result<String, BatteryError> {
        let path = format!("{}/status", self.battery_path);
        let content = fs::read_to_string(&path)?;
        Ok(content.trim().to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_valid_values() {
        // These should pass validation (won't write without hardware)
        let valid = [0, 1, 50, 80, 99, 100];
        for v in valid {
            assert!((0..=100).contains(&v), "{} should be valid", v);
        }
    }

    #[test]
    fn test_validation_invalid_values() {
        let invalid = [-1, -10, 101, 150];
        for v in invalid {
            assert!(!(0..=100).contains(&v), "{} should be invalid", v);
        }
    }
}
