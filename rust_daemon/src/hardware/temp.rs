use super::HardwareError;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

const PER_MODE_SETTINGS_PATH: &str = "/var/lib/asus-linux-control/per-mode-settings.json";
const RYZENADJ_PATH: &str = "/usr/local/bin/ryzenadj";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CpuTempProfileBucket {
    Quiet,
    Balanced,
    Performance,
    Custom,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CpuTempProfileSetting {
    pub enabled: bool,
    pub limit_c: i32,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CpuTempSettings {
    #[serde(default)]
    pub quiet: Option<CpuTempProfileSetting>,
    #[serde(default)]
    pub balanced: Option<CpuTempProfileSetting>,
    #[serde(default)]
    pub performance: Option<CpuTempProfileSetting>,
    #[serde(default)]
    pub custom: Option<CpuTempProfileSetting>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PerModeSettings {
    pub version: u32,
    #[serde(default)]
    pub cpu_temp: CpuTempSettings,
}

impl Default for PerModeSettings {
    fn default() -> Self {
        Self {
            version: 1,
            cpu_temp: CpuTempSettings::default(),
        }
    }
}

pub fn normalize_platform_profile(profile: &str) -> Result<CpuTempProfileBucket, HardwareError> {
    match profile {
        "Quiet" | "LowPower" | "Silent" => Ok(CpuTempProfileBucket::Quiet),
        "Balanced" => Ok(CpuTempProfileBucket::Balanced),
        "Performance" | "Turbo" => Ok(CpuTempProfileBucket::Performance),
        "Custom" => Ok(CpuTempProfileBucket::Custom),
        other => Err(HardwareError::WriteError {
            path: "PlatformProfile".to_string(),
            source: std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("unsupported platform profile: {other}"),
            ),
        }),
    }
}

fn setting_for_bucket(
    settings: &PerModeSettings,
    bucket: CpuTempProfileBucket,
) -> Option<&CpuTempProfileSetting> {
    match bucket {
        CpuTempProfileBucket::Quiet => settings.cpu_temp.quiet.as_ref(),
        CpuTempProfileBucket::Balanced => settings.cpu_temp.balanced.as_ref(),
        CpuTempProfileBucket::Performance => settings.cpu_temp.performance.as_ref(),
        CpuTempProfileBucket::Custom => settings.cpu_temp.custom.as_ref(),
    }
}

fn set_setting_for_bucket(
    settings: &mut PerModeSettings,
    bucket: CpuTempProfileBucket,
    setting: CpuTempProfileSetting,
) {
    match bucket {
        CpuTempProfileBucket::Quiet => settings.cpu_temp.quiet = Some(setting),
        CpuTempProfileBucket::Balanced => settings.cpu_temp.balanced = Some(setting),
        CpuTempProfileBucket::Performance => settings.cpu_temp.performance = Some(setting),
        CpuTempProfileBucket::Custom => settings.cpu_temp.custom = Some(setting),
    }
}

pub fn read_per_mode_settings_at(path: &Path) -> Result<PerModeSettings, HardwareError> {
    if !path.exists() {
        return Ok(PerModeSettings::default());
    }

    let content = fs::read_to_string(path).map_err(|e| HardwareError::ReadError {
        path: path.to_string_lossy().to_string(),
        source: e,
    })?;

    serde_json::from_str(&content).map_err(|e| HardwareError::WriteError {
        path: path.to_string_lossy().to_string(),
        source: std::io::Error::new(std::io::ErrorKind::InvalidData, e),
    })
}

pub fn write_per_mode_settings_at(
    path: &Path,
    settings: &PerModeSettings,
) -> Result<(), HardwareError> {
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir).map_err(|e| HardwareError::WriteError {
            path: dir.to_string_lossy().to_string(),
            source: e,
        })?;
    }

    let content =
        serde_json::to_string_pretty(settings).map_err(|e| HardwareError::WriteError {
            path: path.to_string_lossy().to_string(),
            source: std::io::Error::new(std::io::ErrorKind::InvalidData, e),
        })?;
    fs::write(path, content).map_err(|e| HardwareError::WriteError {
        path: path.to_string_lossy().to_string(),
        source: e,
    })
}

pub fn read_per_mode_settings() -> Result<PerModeSettings, HardwareError> {
    read_per_mode_settings_at(Path::new(PER_MODE_SETTINGS_PATH))
}

pub fn write_per_mode_settings(settings: &PerModeSettings) -> Result<(), HardwareError> {
    write_per_mode_settings_at(Path::new(PER_MODE_SETTINGS_PATH), settings)
}

pub fn validate_temp_limit(temp_c: i32) -> Result<(), HardwareError> {
    if temp_c == 0 || (60..=100).contains(&temp_c) {
        return Ok(());
    }

    Err(HardwareError::WriteError {
        path: RYZENADJ_PATH.to_string(),
        source: std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("temp_c={temp_c} out of range 60-100"),
        ),
    })
}

pub fn read_profile_temp_limit(profile: &str) -> Result<i32, HardwareError> {
    let settings = read_per_mode_settings()?;
    let bucket = normalize_platform_profile(profile)?;
    Ok(setting_for_bucket(&settings, bucket)
        .filter(|setting| setting.enabled)
        .map(|setting| setting.limit_c)
        .unwrap_or(0))
}

pub fn save_profile_temp_limit(profile: &str, temp_c: i32) -> Result<(), HardwareError> {
    validate_temp_limit(temp_c)?;

    let mut settings = read_per_mode_settings()?;
    let bucket = normalize_platform_profile(profile)?;
    set_setting_for_bucket(
        &mut settings,
        bucket,
        CpuTempProfileSetting {
            enabled: temp_c != 0,
            limit_c: temp_c,
        },
    );
    write_per_mode_settings(&settings)
}

pub fn apply_profile_temp_setting<F>(
    settings: &PerModeSettings,
    profile: &str,
    mut apply_limit: F,
) -> Result<(), HardwareError>
where
    F: FnMut(i32) -> Result<(), HardwareError>,
{
    let bucket = normalize_platform_profile(profile)?;
    let Some(setting) = setting_for_bucket(settings, bucket) else {
        return Ok(());
    };

    if !setting.enabled || setting.limit_c == 0 {
        return Ok(());
    }

    validate_temp_limit(setting.limit_c)?;
    apply_limit(setting.limit_c)
}

pub fn apply_saved_temp_limit_for_profile(profile: &str) -> Result<(), HardwareError> {
    let settings = read_per_mode_settings()?;
    apply_profile_temp_setting(&settings, profile, apply_temp_limit_ryzenadj)
}

/// Apply temperature limit via ryzenadj --tctl-temp.
/// Validates range 60-100 before calling.
pub fn apply_temp_limit_ryzenadj(temp_c: i32) -> Result<(), HardwareError> {
    validate_temp_limit(temp_c)?;
    if temp_c == 0 {
        return Ok(());
    }

    let status = std::process::Command::new(RYZENADJ_PATH)
        .arg(format!("--tctl-temp={temp_c}"))
        .status()
        .map_err(|e| HardwareError::WriteError {
            path: RYZENADJ_PATH.to_string(),
            source: e,
        })?;

    if !status.success() {
        return Err(HardwareError::WriteError {
            path: RYZENADJ_PATH.to_string(),
            source: std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("ryzenadj exited with {status}"),
            ),
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_is_valid_temp_range() {
        assert!(!is_valid_temp_range(50)); // ниже минимума
        assert!(!is_valid_temp_range(110)); // выше максимума
        assert!(is_valid_temp_range(60)); // минимум
        assert!(is_valid_temp_range(100)); // максимум
        assert!(is_valid_temp_range(85)); // середина
        assert!(is_valid_temp_range(0)); // 0 — особое значение (no-op)
    }

    fn is_valid_temp_range(temp: i32) -> bool {
        temp == 0 || (60..=100).contains(&temp)
    }
}
