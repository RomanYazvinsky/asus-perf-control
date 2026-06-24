use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::path::Path;

use super::safety::{speed_is_safe_for_temp, speed_percent_to_pwm};

const FAN_ENGINE_CONFIG_PATH: &str = "/var/lib/asus-linux-control/fan-engine-config.json";

#[derive(Debug, Clone)]
pub enum FanEngineConfigValidationError {
    InvalidCurveLength {
        curve_name: &'static str,
        expected: usize,
        actual: usize,
    },
    NonMonotonicTemperatures {
        curve_name: &'static str,
        index: usize,
        previous_temp_c: i32,
        current_temp_c: i32,
    },
    SpeedOutOfRange {
        curve_name: &'static str,
        index: usize,
        speed_percent: i32,
    },
    UnsafeHighTempPoint {
        curve_name: &'static str,
        index: usize,
        temp_c: i32,
        speed_percent: i32,
    },
}

impl std::fmt::Display for FanEngineConfigValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidCurveLength {
                curve_name,
                expected,
                actual,
            } => write!(
                f,
                "{curve_name} must contain exactly {expected} points, got {actual}",
            ),
            Self::NonMonotonicTemperatures {
                curve_name,
                index,
                previous_temp_c,
                current_temp_c,
            } => write!(
                f,
                "{curve_name} temperatures must be strictly increasing at index {index}: {previous_temp_c} -> {current_temp_c}",
            ),
            Self::SpeedOutOfRange {
                curve_name,
                index,
                speed_percent,
            } => write!(
                f,
                "{curve_name} speed at index {index} must be in 0..100, got {speed_percent}",
            ),
            Self::UnsafeHighTempPoint {
                curve_name,
                index,
                temp_c,
                speed_percent,
            } => write!(
                f,
                "{curve_name} point at index {index} is unsafe: {temp_c}°C requires at least 30%, got {speed_percent}%",
            ),
        }
    }
}

impl std::error::Error for FanEngineConfigValidationError {}

#[derive(Debug, Clone)]
pub struct ValidatedCurvePoint {
    pub temp_c: i32,
    pub speed_percent: i32,
    pub pwm: u8,
}

#[derive(Debug, Clone)]
pub struct ValidatedFanEngineConfig {
    pub profile: String,
    pub tick_ms: u64,
    pub hysteresis_c: i32,
    pub ramp_down_delay_ms: u64,
    pub emergency_temp_c: i32,
    pub cpu_curve: Vec<ValidatedCurvePoint>,
    pub gpu_curve: Vec<ValidatedCurvePoint>,
    pub cross_fan: bool,
    pub cross_fan_curve: Option<Vec<ValidatedCurvePoint>>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FanEngineConfig {
    pub profile: String,
    pub tick_ms: u64,
    pub hysteresis_c: i32,
    pub ramp_down_delay_ms: u64,
    pub emergency_temp_c: i32,
    pub cpu_curve: Vec<EngineCurvePoint>,
    pub gpu_curve: Vec<EngineCurvePoint>,
    #[serde(default)]
    pub cross_fan: bool,
    pub cross_fan_curve: Option<Vec<EngineCurvePoint>>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EngineCurvePoint {
    pub temp_c: i32,
    pub speed_percent: i32,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum EngineCurvePointRepr {
    Array([i32; 2]),
    Tuple((i32, i32)),
    ObjectSnake {
        temp_c: i32,
        speed_percent: i32,
    },
    ObjectCamel {
        #[serde(rename = "tempC")]
        temp_c: i32,
        #[serde(rename = "speedPercent")]
        speed_percent: i32,
    },
}

impl<'de> Deserialize<'de> for EngineCurvePoint {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let repr = EngineCurvePointRepr::deserialize(deserializer)?;
        let (temp_c, speed_percent) = match repr {
            EngineCurvePointRepr::Array([temp_c, speed_percent]) => (temp_c, speed_percent),
            EngineCurvePointRepr::Tuple((temp_c, speed_percent)) => (temp_c, speed_percent),
            EngineCurvePointRepr::ObjectSnake {
                temp_c,
                speed_percent,
            } => (temp_c, speed_percent),
            EngineCurvePointRepr::ObjectCamel {
                temp_c,
                speed_percent,
            } => (temp_c, speed_percent),
        };

        Ok(Self {
            temp_c,
            speed_percent,
        })
    }
}

impl FanEngineConfig {
    pub fn validate(&self) -> Result<ValidatedFanEngineConfig, FanEngineConfigValidationError> {
        let cpu_curve = validate_curve("cpu_curve", &self.cpu_curve)?;
        let gpu_curve = validate_curve("gpu_curve", &self.gpu_curve)?;

        let cross_fan_curve = if let Some(curve) = &self.cross_fan_curve {
            Some(validate_curve("cross_fan_curve", curve)?)
        } else {
            None
        };

        Ok(ValidatedFanEngineConfig {
            profile: self.profile.clone(),
            tick_ms: self.tick_ms,
            hysteresis_c: self.hysteresis_c,
            ramp_down_delay_ms: self.ramp_down_delay_ms,
            emergency_temp_c: self.emergency_temp_c,
            cpu_curve,
            gpu_curve,
            cross_fan: self.cross_fan,
            cross_fan_curve,
        })
    }
}

#[derive(Debug)]
pub enum FanEngineConfigStoreError {
    Read {
        path: String,
        source: io::Error,
    },
    Write {
        path: String,
        source: io::Error,
    },
    Parse {
        path: String,
        source: serde_json::Error,
    },
    Serialize {
        path: String,
        source: serde_json::Error,
    },
}

impl std::fmt::Display for FanEngineConfigStoreError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Read { path, source } => write!(f, "failed to read {path}: {source}"),
            Self::Write { path, source } => write!(f, "failed to write {path}: {source}"),
            Self::Parse { path, source } => write!(f, "failed to parse {path}: {source}"),
            Self::Serialize { path, source } => {
                write!(f, "failed to serialize {path}: {source}")
            }
        }
    }
}

impl std::error::Error for FanEngineConfigStoreError {}

pub fn read_saved_fan_engine_config() -> Result<Option<FanEngineConfig>, FanEngineConfigStoreError>
{
    read_saved_fan_engine_config_at(Path::new(FAN_ENGINE_CONFIG_PATH))
}

pub fn read_saved_fan_engine_config_at(
    path: &Path,
) -> Result<Option<FanEngineConfig>, FanEngineConfigStoreError> {
    if !path.exists() {
        return Ok(None);
    }

    let content = fs::read_to_string(path).map_err(|e| FanEngineConfigStoreError::Read {
        path: path.to_string_lossy().to_string(),
        source: e,
    })?;

    serde_json::from_str(&content)
        .map(Some)
        .map_err(|e| FanEngineConfigStoreError::Parse {
            path: path.to_string_lossy().to_string(),
            source: e,
        })
}

pub fn save_fan_engine_config(config: &FanEngineConfig) -> Result<(), FanEngineConfigStoreError> {
    save_fan_engine_config_at(Path::new(FAN_ENGINE_CONFIG_PATH), config)
}

pub fn save_fan_engine_config_at(
    path: &Path,
    config: &FanEngineConfig,
) -> Result<(), FanEngineConfigStoreError> {
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir).map_err(|e| FanEngineConfigStoreError::Write {
            path: dir.to_string_lossy().to_string(),
            source: e,
        })?;
    }

    let content =
        serde_json::to_string_pretty(config).map_err(|e| FanEngineConfigStoreError::Serialize {
            path: path.to_string_lossy().to_string(),
            source: e,
        })?;
    fs::write(path, content).map_err(|e| FanEngineConfigStoreError::Write {
        path: path.to_string_lossy().to_string(),
        source: e,
    })
}

pub fn clear_saved_fan_engine_config() -> Result<(), FanEngineConfigStoreError> {
    clear_saved_fan_engine_config_at(Path::new(FAN_ENGINE_CONFIG_PATH))
}

pub fn clear_saved_fan_engine_config_at(path: &Path) -> Result<(), FanEngineConfigStoreError> {
    if !path.exists() {
        return Ok(());
    }
    fs::remove_file(path).map_err(|e| FanEngineConfigStoreError::Write {
        path: path.to_string_lossy().to_string(),
        source: e,
    })
}

pub fn config_from_validated(validated: &ValidatedFanEngineConfig) -> FanEngineConfig {
    FanEngineConfig {
        profile: validated.profile.clone(),
        tick_ms: validated.tick_ms,
        hysteresis_c: validated.hysteresis_c,
        ramp_down_delay_ms: validated.ramp_down_delay_ms,
        emergency_temp_c: validated.emergency_temp_c,
        cpu_curve: curve_from_validated(&validated.cpu_curve),
        gpu_curve: curve_from_validated(&validated.gpu_curve),
        cross_fan: validated.cross_fan,
        cross_fan_curve: validated
            .cross_fan_curve
            .as_ref()
            .map(|curve| curve_from_validated(curve)),
    }
}

fn curve_from_validated(curve: &[ValidatedCurvePoint]) -> Vec<EngineCurvePoint> {
    curve
        .iter()
        .map(|point| EngineCurvePoint {
            temp_c: point.temp_c,
            speed_percent: point.speed_percent,
        })
        .collect()
}

fn validate_curve(
    curve_name: &'static str,
    points: &[EngineCurvePoint],
) -> Result<Vec<ValidatedCurvePoint>, FanEngineConfigValidationError> {
    const REQUIRED_POINTS: usize = 8;

    if points.len() != REQUIRED_POINTS {
        return Err(FanEngineConfigValidationError::InvalidCurveLength {
            curve_name,
            expected: REQUIRED_POINTS,
            actual: points.len(),
        });
    }

    let mut validated = Vec::with_capacity(points.len());

    for (index, point) in points.iter().enumerate() {
        if !(0..=100).contains(&point.speed_percent) {
            return Err(FanEngineConfigValidationError::SpeedOutOfRange {
                curve_name,
                index,
                speed_percent: point.speed_percent,
            });
        }

        if !speed_is_safe_for_temp(point.temp_c, point.speed_percent) {
            return Err(FanEngineConfigValidationError::UnsafeHighTempPoint {
                curve_name,
                index,
                temp_c: point.temp_c,
                speed_percent: point.speed_percent,
            });
        }

        if index > 0 {
            let prev = &points[index - 1];
            if point.temp_c <= prev.temp_c {
                return Err(FanEngineConfigValidationError::NonMonotonicTemperatures {
                    curve_name,
                    index,
                    previous_temp_c: prev.temp_c,
                    current_temp_c: point.temp_c,
                });
            }
        }

        validated.push(ValidatedCurvePoint {
            temp_c: point.temp_c,
            speed_percent: point.speed_percent,
            pwm: speed_percent_to_pwm(point.speed_percent),
        });
    }

    Ok(validated)
}
