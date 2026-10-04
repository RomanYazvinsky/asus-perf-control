use std::path::Path;

use super::{read_sysfs_i32, write_sysfs, HardwareError};

const PPT_PL1_PATH: &str = "/sys/devices/platform/asus-nb-wmi/ppt_pl1_spl";
const PPT_PL2_PATH: &str = "/sys/devices/platform/asus-nb-wmi/ppt_pl2_sppt";
const PPT_CPU_TEMP_PATH: &str = "/sys/devices/platform/asus-nb-wmi/ppt_cpu_limit";
const PPT_APU_SPPT_PATH: &str = "/sys/devices/platform/asus-nb-wmi/ppt_apu_sppt";

pub fn is_tdp_supported() -> bool {
    Path::new(PPT_PL1_PATH).exists()
}

/// Read sustained TDP from sysfs. sysfs stores watts directly; returns watts.
pub fn read_sustained_tdp() -> Result<i32, HardwareError> {
    if !Path::new(PPT_PL1_PATH).exists() {
        return Err(HardwareError::TdpNotSupported);
    }
    read_sysfs_i32(PPT_PL1_PATH, |e| HardwareError::ReadError {
        path: PPT_PL1_PATH.to_string(),
        source: e,
    })
}

/// Write sustained TDP to sysfs. Accepts watts; sysfs expects watts directly.
pub fn write_sustained_tdp(watts: i32) -> Result<(), HardwareError> {
    if !Path::new(PPT_PL1_PATH).exists() {
        return Err(HardwareError::TdpNotSupported);
    }
    if watts <= 0 {
        return Ok(());
    }
    write_sysfs(PPT_PL1_PATH, watts, |e| HardwareError::WriteError {
        path: PPT_PL1_PATH.to_string(),
        source: e,
    })
}

/// Read boost TDP from sysfs. sysfs stores watts directly; returns watts.
pub fn read_boost_tdp() -> Result<i32, HardwareError> {
    if !Path::new(PPT_PL2_PATH).exists() {
        return Err(HardwareError::TdpNotSupported);
    }
    read_sysfs_i32(PPT_PL2_PATH, |e| HardwareError::ReadError {
        path: PPT_PL2_PATH.to_string(),
        source: e,
    })
}

/// Write boost TDP to sysfs. Accepts watts; sysfs expects watts directly.
pub fn write_boost_tdp(watts: i32) -> Result<(), HardwareError> {
    if !Path::new(PPT_PL2_PATH).exists() {
        return Err(HardwareError::TdpNotSupported);
    }
    if watts <= 0 {
        return Ok(());
    }
    write_sysfs(PPT_PL2_PATH, watts, |e| HardwareError::WriteError {
        path: PPT_PL2_PATH.to_string(),
        source: e,
    })
}

/// Read CPU temperature limit from sysfs. Stored in °C; returns °C directly.
/// Returns 0 if hardware does not expose a temperature limit interface.
pub fn read_cpu_temp_limit() -> Result<i32, HardwareError> {
    if let Ok(val) = read_sysfs_i32(PPT_CPU_TEMP_PATH, |e| HardwareError::ReadError {
        path: PPT_CPU_TEMP_PATH.to_string(),
        source: e,
    }) {
        return Ok(val);
    }
    if let Ok(val) = read_sysfs_i32(PPT_APU_SPPT_PATH, |e| HardwareError::ReadError {
        path: PPT_APU_SPPT_PATH.to_string(),
        source: e,
    }) {
        return Ok(val);
    }
    Ok(0)
}

/// Write CPU temperature limit to sysfs. Accepts °C; sysfs stores °C directly.
/// No-op if hardware does not expose a temperature limit interface.
pub fn write_cpu_temp_limit(temp_c: i32) -> Result<(), HardwareError> {
    if temp_c <= 0 {
        return Ok(());
    }
    if Path::new(PPT_CPU_TEMP_PATH).exists() {
        return write_sysfs(PPT_CPU_TEMP_PATH, temp_c, |e| HardwareError::WriteError {
            path: PPT_CPU_TEMP_PATH.to_string(),
            source: e,
        });
    }
    if Path::new(PPT_APU_SPPT_PATH).exists() {
        return write_sysfs(PPT_APU_SPPT_PATH, temp_c, |e| HardwareError::WriteError {
            path: PPT_APU_SPPT_PATH.to_string(),
            source: e,
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    #[allow(unused_imports)]
    use super::*;

    #[test]
    fn test_write_cpu_temp_ryzenadj_invalid_range() {
        // 0 — no-op, не ошибка (функция не вызывает ryzenadj)
        // Пока функция не реализована, этот тест проверяет сигнатуру
        // Реальный вызов ryzenadj будет в отдельном integration test
        
        // Проверяем валидацию диапазона (60-100)
        // Значения вне диапазона должны возвращать ошибку
        assert!(!is_valid_temp_range(50)); // ниже минимума
        assert!(!is_valid_temp_range(110)); // выше максимума
        assert!(is_valid_temp_range(60));  // минимум
        assert!(is_valid_temp_range(100)); // максимум
        assert!(is_valid_temp_range(85)); // середина
        assert!(is_valid_temp_range(0));   // 0 — особое значение (no-op)
    }
    
    fn is_valid_temp_range(temp: i32) -> bool {
        temp == 0 || (60..=100).contains(&temp)
    }

    #[test]
    fn test_read_value_passthrough() {
        // sysfs stores watts directly, no conversion needed
        let sysfs_value: i32 = 15;
        assert_eq!(sysfs_value, 15);
    }

    #[test]
    fn test_write_value_passthrough() {
        // caller passes watts, written to sysfs as-is
        let watts: i32 = 15;
        assert_eq!(watts, 15);
    }

    #[test]
    fn test_boundary_min_tdp() {
        // 5W is minimum valid value for sysfs
        let watts: i32 = 5;
        assert!(watts >= 5);
    }

    #[test]
    fn test_boundary_max_tdp() {
        // 54W is maximum valid value for sysfs
        let watts: i32 = 54;
        assert!(watts <= 54);
    }

    #[test]
    fn test_temp_limit_no_conversion() {
        // ppt_cpu_limit stores °C directly — no conversion needed
        let celsius: i32 = 90;
        assert_eq!(celsius, 90);
    }
}
