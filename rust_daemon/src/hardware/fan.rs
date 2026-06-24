use super::HardwareError;
use std::fs;
use std::path::Path;

/// Read CPU temperature from hwmon sysfs.
/// Returns temperature in Celsius.
pub fn read_cpu_temp() -> Result<i32, HardwareError> {
    // Scan /sys/class/hwmon/hwmon*/temp*_input for CPU temperature
    // Typical CPU hwmon names: coretemp, k10temp, cpu
    let hwmon_dir = "/sys/class/hwmon";

    let entries = fs::read_dir(hwmon_dir).map_err(|e| HardwareError::ReadError {
        path: hwmon_dir.to_string(),
        source: e,
    })?;

    for entry in entries {
        let entry = entry.map_err(|e| HardwareError::ReadError {
            path: hwmon_dir.to_string(),
            source: e,
        })?;
        let hwmon_path = entry.path();
        let name_path = hwmon_path.join("name");

        if !name_path.exists() {
            continue;
        }

        let name = match fs::read_to_string(&name_path) {
            Ok(n) => n,
            Err(_) => continue,
        };

        let name = name.trim();

        // Check if this is a CPU hwmon
        if name == "coretemp" || name == "k10temp" || name.contains("cpu") {
            // Find the first temp*_input file
            let temp_entries = fs::read_dir(&hwmon_path).map_err(|e| HardwareError::ReadError {
                path: hwmon_path.to_string_lossy().to_string(),
                source: e,
            })?;

            for temp_entry in temp_entries {
                let temp_entry = temp_entry.map_err(|e| HardwareError::ReadError {
                    path: hwmon_path.to_string_lossy().to_string(),
                    source: e,
                })?;
                let file_name = temp_entry.file_name();
                let file_name_str = file_name.to_string_lossy();

                if file_name_str.starts_with("temp") && file_name_str.ends_with("_input") {
                    let temp_path = hwmon_path.join(&file_name);
                    let content =
                        fs::read_to_string(&temp_path).map_err(|e| HardwareError::ReadError {
                            path: temp_path.to_string_lossy().to_string(),
                            source: e,
                        })?;

                    let temp_millidegrees: i32 =
                        content
                            .trim()
                            .parse()
                            .map_err(|e| HardwareError::ParseError {
                                path: temp_path.to_string_lossy().to_string(),
                                source: e,
                            })?;

                    // Convert to °C
                    return Ok(temp_millidegrees / 1000);
                }
            }
        }
    }

    Err(HardwareError::ReadError {
        path: "CPU temperature sensor".to_string(),
        source: std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "CPU temperature sensor not found",
        ),
    })
}

/// Read GPU temperature from hwmon sysfs.
/// Returns temperature in Celsius.
pub fn read_gpu_temp() -> Result<i32, HardwareError> {
    // Scan /sys/class/hwmon/hwmon*/temp*_input for GPU temperature
    // Typical GPU hwmon names: amdgpu, nvidia, radeon, gpu
    let hwmon_dir = "/sys/class/hwmon";

    let entries = fs::read_dir(hwmon_dir).map_err(|e| HardwareError::ReadError {
        path: hwmon_dir.to_string(),
        source: e,
    })?;

    for entry in entries {
        let entry = entry.map_err(|e| HardwareError::ReadError {
            path: hwmon_dir.to_string(),
            source: e,
        })?;
        let hwmon_path = entry.path();
        let name_path = hwmon_path.join("name");

        if !name_path.exists() {
            continue;
        }

        let name = match fs::read_to_string(&name_path) {
            Ok(n) => n,
            Err(_) => continue,
        };

        let name = name.trim();

        // Check if this is a GPU hwmon
        if name == "amdgpu" || name == "nvidia" || name == "radeon" || name.contains("gpu") {
            // Find the first temp*_input file
            let temp_entries = fs::read_dir(&hwmon_path).map_err(|e| HardwareError::ReadError {
                path: hwmon_path.to_string_lossy().to_string(),
                source: e,
            })?;

            for temp_entry in temp_entries {
                let temp_entry = temp_entry.map_err(|e| HardwareError::ReadError {
                    path: hwmon_path.to_string_lossy().to_string(),
                    source: e,
                })?;
                let file_name = temp_entry.file_name();
                let file_name_str = file_name.to_string_lossy();

                if file_name_str.starts_with("temp") && file_name_str.ends_with("_input") {
                    let temp_path = hwmon_path.join(&file_name);
                    let content =
                        fs::read_to_string(&temp_path).map_err(|e| HardwareError::ReadError {
                            path: temp_path.to_string_lossy().to_string(),
                            source: e,
                        })?;

                    let temp_millidegrees: i32 =
                        content
                            .trim()
                            .parse()
                            .map_err(|e| HardwareError::ParseError {
                                path: temp_path.to_string_lossy().to_string(),
                                source: e,
                            })?;

                    // Convert to °C
                    return Ok(temp_millidegrees / 1000);
                }
            }
        }
    }

    Err(HardwareError::ReadError {
        path: "GPU temperature sensor".to_string(),
        source: std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "GPU temperature sensor not found",
        ),
    })
}

/// Write 8-point fan curve to hwmon7 sysfs.
/// curve_points: array of 8 (temp, duty) pairs where temp is in °C and duty is 0-255
#[allow(dead_code)]
pub fn write_fan_curve(hwmon_path: &str, curve_points: &[(i32, i32)]) -> Result<(), HardwareError> {
    // Write the curve to the auto point files
    for (i, (temp, duty)) in curve_points.iter().enumerate() {
        let temp_path = Path::new(hwmon_path).join(&format!("pwm7_auto_point{}_temp", i + 1));
        let pwm_path = Path::new(hwmon_path).join(&format!("pwm7_auto_point{}_pwm", i + 1));

        fs::write(&temp_path, temp.to_string()).map_err(|e| HardwareError::WriteError {
            path: temp_path.to_string_lossy().to_string(),
            source: e,
        })?;

        fs::write(&pwm_path, duty.to_string()).map_err(|e| HardwareError::WriteError {
            path: pwm_path.to_string_lossy().to_string(),
            source: e,
        })?;
    }

    Ok(())
}

/// Set fan mode to auto (2) or manual (1)
#[allow(dead_code)]
pub fn set_fan_mode(hwmon_path: &str, mode: i32) -> Result<(), HardwareError> {
    let pwm_enable_path = Path::new(hwmon_path).join("pwm_enable");
    let pwm_enable_str = pwm_enable_path.to_string_lossy().to_string();
    fs::write(&pwm_enable_path, mode.to_string()).map_err(|e| HardwareError::WriteError {
        path: pwm_enable_str,
        source: e,
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FanChannel {
    Cpu,
    Gpu,
}

impl FanChannel {
    pub fn pwm_channel(self) -> u8 {
        match self {
            Self::Cpu => 1,
            Self::Gpu => 2,
        }
    }
}

pub fn discover_asus_curve_base_path() -> Result<std::path::PathBuf, HardwareError> {
    let hwmon_dir = Path::new("/sys/class/hwmon");
    let entries = fs::read_dir(hwmon_dir).map_err(|e| HardwareError::ReadError {
        path: hwmon_dir.to_string_lossy().to_string(),
        source: e,
    })?;

    for entry in entries {
        let entry = entry.map_err(|e| HardwareError::ReadError {
            path: hwmon_dir.to_string_lossy().to_string(),
            source: e,
        })?;
        let path = entry.path();
        let name_path = path.join("name");
        let name = match fs::read_to_string(&name_path) {
            Ok(value) => value,
            Err(_) => continue,
        };
        if name.trim() == "asus_custom_fan_curve" {
            return Ok(path);
        }
    }

    Err(HardwareError::ReadError {
        path: "asus_custom_fan_curve hwmon".to_string(),
        source: std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "asus_custom_fan_curve hwmon device not found",
        ),
    })
}

pub fn write_flat_fan_curve(
    curve_base_path: &Path,
    channel: FanChannel,
    pwm: u8,
) -> Result<(), HardwareError> {
    let pwm_channel = channel.pwm_channel();
    let enable_path = curve_base_path.join(format!("pwm{pwm_channel}_enable"));
    for point in 1..=8 {
        let temp_path = curve_base_path.join(format!("pwm{pwm_channel}_auto_point{point}_temp"));
        let pwm_path = curve_base_path.join(format!("pwm{pwm_channel}_auto_point{point}_pwm"));

        let temp_c = 20 + ((point - 1) * 10);
        fs::write(&temp_path, temp_c.to_string()).map_err(|e| HardwareError::WriteError {
            path: temp_path.to_string_lossy().to_string(),
            source: e,
        })?;
        fs::write(&pwm_path, pwm.to_string()).map_err(|e| HardwareError::WriteError {
            path: pwm_path.to_string_lossy().to_string(),
            source: e,
        })?;
    }
    fs::write(&enable_path, "1").map_err(|e| HardwareError::WriteError {
        path: enable_path.to_string_lossy().to_string(),
        source: e,
    })?;
    Ok(())
}

pub fn set_fan_auto_mode(curve_base_path: &Path, channel: FanChannel) -> Result<(), HardwareError> {
    let pwm_channel = channel.pwm_channel();
    let enable_path = curve_base_path.join(format!("pwm{pwm_channel}_enable"));
    fs::write(&enable_path, "2").map_err(|e| HardwareError::WriteError {
        path: enable_path.to_string_lossy().to_string(),
        source: e,
    })
}
