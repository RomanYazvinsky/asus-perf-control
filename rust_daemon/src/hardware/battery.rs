use std::path::Path;

use super::{read_sysfs_i32, write_sysfs, HardwareError};

const CHARGE_LIMIT_PATH: &str = "/sys/class/power_supply/BAT0/charge_control_end_threshold";

pub fn read_charge_limit() -> Result<i32, HardwareError> {
    if !Path::new(CHARGE_LIMIT_PATH).exists() {
        return Err(HardwareError::ChargeLimitNotFound);
    }
    read_sysfs_i32(CHARGE_LIMIT_PATH, |e| HardwareError::ReadError {
        path: CHARGE_LIMIT_PATH.to_string(),
        source: e,
    })
}

pub fn write_charge_limit(limit: i32) -> Result<(), HardwareError> {
    if !Path::new(CHARGE_LIMIT_PATH).exists() {
        return Err(HardwareError::ChargeLimitNotFound);
    }
    write_sysfs(CHARGE_LIMIT_PATH, limit, |e| HardwareError::WriteError {
        path: CHARGE_LIMIT_PATH.to_string(),
        source: e,
    })
}
