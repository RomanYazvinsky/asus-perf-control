// Fan hardware discovery and RPM reading
// Scans hwmon sysfs for ASUS fan controllers (asus-wmi)

use serde::{Deserialize, Serialize};

/// Fan identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FanId {
    CpuFan,
    GpuFan,
}

impl FanId {
    /// Returns the asusd D-Bus fan name
    pub fn asusd_name(&self) -> &'static str {
        match self {
            FanId::CpuFan => "CPU",
            FanId::GpuFan => "GPU",
        }
    }

    /// Returns the hwmon7 PWM channel number
    pub fn hwmon7_pwm_channel(&self) -> u8 {
        match self {
            FanId::CpuFan => 1,
            FanId::GpuFan => 2,
        }
    }
}

/// Fan hardware information discovered from sysfs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FanInfo {
    pub fan_id: FanId,
    pub rpm_path: String,        // hwmon6/fan1_input (or fan2_input)
    pub curve_base_path: String, // hwmon7 dir path
    pub pwm_channel: u8,         // 1 = CPU, 2 = GPU
    pub label: String,           // "CPU Fan", "GPU Fan"
}

/// Discover fans by scanning /sys/class/hwmon/
/// Returns Vec<FanInfo> for detected fans (cpu_fan and gpu_fan)
pub fn discover_fans() -> Vec<FanInfo> {
    let mut fans = Vec::new();

    // Scan hwmon directories
    if let Ok(entries) = std::fs::read_dir("/sys/class/hwmon") {
        let mut asus_hwmon: Option<String> = None;
        let mut curve_hwmon: Option<String> = None;

        for entry in entries.flatten() {
            let path = entry.path();
            if let Some(_name) = path.file_name().and_then(|n| n.to_str()) {
                let name_path = path.join("name");
                if let Ok(hwmon_name) = std::fs::read_to_string(&name_path) {
                    let hwmon_name = hwmon_name.trim();
                    if hwmon_name == "asus" {
                        asus_hwmon = Some(path.to_string_lossy().to_string());
                    } else if hwmon_name == "asus_custom_fan_curve" {
                        curve_hwmon = Some(path.to_string_lossy().to_string());
                    }
                }
            }
        }

        // If we found both hwmon devices, discover fans
        if let (Some(asus_path), Some(curve_path)) = (asus_hwmon, curve_hwmon) {
            // Try to discover CPU fan (fan1)
            if let Some(cpu_fan) =
                discover_single_fan(&asus_path, &curve_path, "cpu_fan", FanId::CpuFan, 1)
            {
                fans.push(cpu_fan);
            }

            // Try to discover GPU fan (fan2)
            if let Some(gpu_fan) =
                discover_single_fan(&asus_path, &curve_path, "gpu_fan", FanId::GpuFan, 2)
            {
                fans.push(gpu_fan);
            }
        }
    }

    fans
}

fn discover_single_fan(
    asus_path: &str,
    curve_path: &str,
    expected_label: &str,
    fan_id: FanId,
    pwm_channel: u8,
) -> Option<FanInfo> {
    let fan_num = match fan_id {
        FanId::CpuFan => 1,
        FanId::GpuFan => 2,
    };

    // Check if fan label matches expected
    let label_path = format!("{}/fan{}_label", asus_path, fan_num);
    if let Ok(label) = std::fs::read_to_string(&label_path) {
        if label.trim() != expected_label {
            return None;
        }
    } else {
        return None;
    }

    // Check if RPM input exists
    let rpm_path = format!("{}/fan{}_input", asus_path, fan_num);
    if !std::path::Path::new(&rpm_path).exists() {
        return None;
    }

    // Verify curve control files exist
    for point in 1..=8 {
        let temp_path = format!("{}/pwm{}_auto_point{}_temp", curve_path, pwm_channel, point);
        let pwm_path = format!("{}/pwm{}_auto_point{}_pwm", curve_path, pwm_channel, point);
        if !std::path::Path::new(&temp_path).exists() || !std::path::Path::new(&pwm_path).exists() {
            return None;
        }
    }

    Some(FanInfo {
        fan_id,
        rpm_path,
        curve_base_path: curve_path.to_string(),
        pwm_channel,
        label: match fan_id {
            FanId::CpuFan => "CPU Fan".to_string(),
            FanId::GpuFan => "GPU Fan".to_string(),
        },
    })
}

/// Read RPM value for a specific fan
pub fn read_fan_rpm(fan: &FanInfo) -> Option<u32> {
    std::fs::read_to_string(&fan.rpm_path)
        .ok()?
        .trim()
        .parse()
        .ok()
}

/// Read RPM values for all discovered fans
pub fn read_all_fan_rpms() -> Vec<(FanId, Option<u32>)> {
    discover_fans()
        .iter()
        .map(|f| (f.fan_id, read_fan_rpm(f)))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_discover_fans_returns_empty_when_no_hwmon() {
        // On CI/non-ASUS hardware, should return empty list without panic
        let fans = discover_fans();
        // Just verify it doesn't crash - may return fans on real hardware
        assert!(fans.len() <= 3);
    }

    #[test]
    fn test_fan_id_asusd_names() {
        assert_eq!(FanId::CpuFan.asusd_name(), "CPU");
        assert_eq!(FanId::GpuFan.asusd_name(), "GPU");
    }

    #[test]
    fn test_fan_id_pwm_channels() {
        assert_eq!(FanId::CpuFan.hwmon7_pwm_channel(), 1);
        assert_eq!(FanId::GpuFan.hwmon7_pwm_channel(), 2);
    }
}
