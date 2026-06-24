use std::fs;

#[derive(Debug)]
pub enum TempError {
    ReadError {
        path: String,
        source: std::io::Error,
    },
    ParseError {
        path: String,
        source: std::num::ParseIntError,
    },
}

impl std::fmt::Display for TempError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TempError::ReadError { path, source } => {
                write!(f, "Failed to read from {}: {}", path, source)
            }
            TempError::ParseError { path, source } => {
                write!(f, "Failed to parse {}: {}", path, source)
            }
        }
    }
}

impl std::error::Error for TempError {}

impl From<std::io::Error> for TempError {
    fn from(e: std::io::Error) -> Self {
        TempError::ReadError {
            path: "unknown".to_string(),
            source: e,
        }
    }
}

/// Read CPU temperature from hwmon sysfs.
/// Returns temperature in millidegrees Celsius (divide by 1000 for °C).
pub fn read_cpu_temp() -> Result<i32, TempError> {
    // Scan /sys/class/hwmon/hwmon*/temp*_input for CPU temperature
    // Typical CPU hwmon names: coretemp, k10temp, cpu
    let hwmon_dir = "/sys/class/hwmon";

    for entry in fs::read_dir(hwmon_dir).map_err(|e| TempError::ReadError {
        path: hwmon_dir.to_string(),
        source: e,
    })? {
        let entry = entry?;
        let hwmon_path = entry.path();
        let name_path = hwmon_path.join("name");

        if !name_path.exists() {
            continue;
        }

        let name = fs::read_to_string(&name_path).map_err(|e| TempError::ReadError {
            path: name_path.to_string_lossy().to_string(),
            source: e,
        })?;

        let name = name.trim();

        // Check if this is a CPU hwmon
        if name == "coretemp" || name == "k10temp" || name.contains("cpu") {
            // Find the first temp*_input file
            for temp_entry in fs::read_dir(&hwmon_path).map_err(|e| TempError::ReadError {
                path: hwmon_path.to_string_lossy().to_string(),
                source: e,
            })? {
                let temp_entry = temp_entry?;
                let file_name = temp_entry.file_name();
                let file_name_str = file_name.to_string_lossy();

                if file_name_str.starts_with("temp") && file_name_str.ends_with("_input") {
                    let temp_path = hwmon_path.join(&file_name);
                    let content =
                        fs::read_to_string(&temp_path).map_err(|e| TempError::ReadError {
                            path: temp_path.to_string_lossy().to_string(),
                            source: e,
                        })?;

                    let temp_millidegrees: i32 =
                        content.trim().parse().map_err(|e| TempError::ParseError {
                            path: temp_path.to_string_lossy().to_string(),
                            source: e,
                        })?;

                    // Convert to °C
                    return Ok(temp_millidegrees / 1000);
                }
            }
        }
    }

    Err(TempError::ReadError {
        path: "CPU temperature sensor".to_string(),
        source: std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "CPU temperature sensor not found",
        ),
    })
}

/// Read GPU temperature from hwmon sysfs.
/// Returns temperature in millidegrees Celsius (divide by 1000 for °C).
pub fn read_gpu_temp() -> Result<i32, TempError> {
    // Scan /sys/class/hwmon/hwmon*/temp*_input for GPU temperature
    // Typical GPU hwmon names: amdgpu, nvidia, radeon, gpu
    let hwmon_dir = "/sys/class/hwmon";

    for entry in fs::read_dir(hwmon_dir).map_err(|e| TempError::ReadError {
        path: hwmon_dir.to_string(),
        source: e,
    })? {
        let entry = entry?;
        let hwmon_path = entry.path();
        let name_path = hwmon_path.join("name");

        if !name_path.exists() {
            continue;
        }

        let name = fs::read_to_string(&name_path).map_err(|e| TempError::ReadError {
            path: name_path.to_string_lossy().to_string(),
            source: e,
        })?;

        let name = name.trim();

        // Check if this is a GPU hwmon
        if name == "amdgpu" || name == "nvidia" || name == "radeon" || name.contains("gpu") {
            // Find the first temp*_input file
            for temp_entry in fs::read_dir(&hwmon_path).map_err(|e| TempError::ReadError {
                path: hwmon_path.to_string_lossy().to_string(),
                source: e,
            })? {
                let temp_entry = temp_entry?;
                let file_name = temp_entry.file_name();
                let file_name_str = file_name.to_string_lossy();

                if file_name_str.starts_with("temp") && file_name_str.ends_with("_input") {
                    let temp_path = hwmon_path.join(&file_name);
                    let content =
                        fs::read_to_string(&temp_path).map_err(|e| TempError::ReadError {
                            path: temp_path.to_string_lossy().to_string(),
                            source: e,
                        })?;

                    let temp_millidegrees: i32 =
                        content.trim().parse().map_err(|e| TempError::ParseError {
                            path: temp_path.to_string_lossy().to_string(),
                            source: e,
                        })?;

                    // Convert to °C
                    return Ok(temp_millidegrees / 1000);
                }
            }
        }
    }

    Err(TempError::ReadError {
        path: "GPU temperature sensor".to_string(),
        source: std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "GPU temperature sensor not found",
        ),
    })
}
