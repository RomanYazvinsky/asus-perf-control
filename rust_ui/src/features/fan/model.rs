//! Fan data model shared by the asusd client, the daemon fan engine and the UI.

use serde_json::{json, Value};

use crate::settings::json::JsonMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum FanId {
    Cpu,
    Gpu,
}

impl FanId {
    pub const ALL: [FanId; 2] = [FanId::Cpu, FanId::Gpu];

    /// Settings / daemon id: `cpu`, `gpu`.
    pub fn name(self) -> &'static str {
        match self {
            FanId::Cpu => "cpu",
            FanId::Gpu => "gpu",
        }
    }

    /// asusd curve name: `CPU`, `GPU`.
    pub fn asusd_name(self) -> &'static str {
        match self {
            FanId::Cpu => "CPU",
            FanId::Gpu => "GPU",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            FanId::Cpu => "CPU Fan",
            FanId::Gpu => "GPU Fan",
        }
    }

    pub fn from_name(name: &str) -> Option<FanId> {
        match name {
            "cpu" => Some(FanId::Cpu),
            "gpu" => Some(FanId::Gpu),
            _ => None,
        }
    }

    /// Matches asusd curve names case-insensitively.
    pub fn from_curve_name(name: &str) -> Option<FanId> {
        Self::from_name(&name.to_lowercase())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FanControlMode {
    AsusctlCurve,
    ConstantSpeed,
    DynamicEngine,
}

impl FanControlMode {
    pub const ALL: [FanControlMode; 3] = [
        FanControlMode::AsusctlCurve,
        FanControlMode::ConstantSpeed,
        FanControlMode::DynamicEngine,
    ];

    pub fn name(self) -> &'static str {
        match self {
            FanControlMode::AsusctlCurve => "asusctlCurve",
            FanControlMode::ConstantSpeed => "constantSpeed",
            FanControlMode::DynamicEngine => "dynamicEngine",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            FanControlMode::AsusctlCurve => "Asusctl Curve",
            FanControlMode::ConstantSpeed => "Constant Speed",
            FanControlMode::DynamicEngine => "Dynamic Engine",
        }
    }

    pub fn from_name(name: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|mode| mode.name() == name)
    }
}

/// asusd platform profile.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum PerfProfile {
    Balanced,
    Performance,
    Quiet,
    LowPower,
}

impl PerfProfile {
    pub const ALL: [PerfProfile; 4] = [
        PerfProfile::Balanced,
        PerfProfile::Performance,
        PerfProfile::Quiet,
        PerfProfile::LowPower,
    ];

    pub fn name(self) -> &'static str {
        match self {
            PerfProfile::Balanced => "balanced",
            PerfProfile::Performance => "performance",
            PerfProfile::Quiet => "quiet",
            PerfProfile::LowPower => "lowPower",
        }
    }

    pub fn from_name(name: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|p| p.name() == name)
    }

    /// asusd `PlatformProfile` id.
    pub fn asusd_id(self) -> u32 {
        match self {
            PerfProfile::Balanced => 0,
            PerfProfile::Performance => 1,
            PerfProfile::Quiet => 2,
            PerfProfile::LowPower => 3,
        }
    }

    pub fn from_asusd_id(id: u32) -> Option<Self> {
        Self::ALL.into_iter().find(|p| p.asusd_id() == id)
    }

    /// UI mode index (0 Silent, 1 Balanced, 2 Turbo).
    pub fn to_mode(self) -> i32 {
        match self {
            PerfProfile::Quiet | PerfProfile::LowPower => 0,
            PerfProfile::Balanced => 1,
            PerfProfile::Performance => 2,
        }
    }

    pub fn from_mode(mode: i32) -> Self {
        match mode {
            0 => PerfProfile::Quiet,
            2 => PerfProfile::Performance,
            _ => PerfProfile::Balanced,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FanCurvePoint {
    pub temp_c: f64,
    pub speed_percent: f64,
}

impl FanCurvePoint {
    pub fn new(temp_c: f64, speed_percent: f64) -> Self {
        Self { temp_c, speed_percent }
    }
}

/// Linear ramp 20 °C/30 % → 90 °C/100 %.
pub fn default_fan_curve() -> Vec<FanCurvePoint> {
    (0..8)
        .map(|i| FanCurvePoint::new(20.0 + i as f64 * 10.0, 30.0 + (i as f64 / 7.0) * 70.0))
        .collect()
}

/// Temps >= 80 °C require at least 30 % speed.
pub fn with_safety_clamp(points: &[FanCurvePoint]) -> Vec<FanCurvePoint> {
    points
        .iter()
        .map(|p| {
            if p.temp_c >= 80.0 && p.speed_percent < 30.0 {
                FanCurvePoint::new(p.temp_c, 30.0)
            } else {
                *p
            }
        })
        .collect()
}

/// Rounded temperatures forced to be strictly increasing.
pub fn normalized_rounded_temps(curve: &[FanCurvePoint]) -> Vec<i64> {
    let mut temps: Vec<i64> = curve.iter().map(|p| p.temp_c.round() as i64).collect();
    for i in 1..temps.len() {
        if temps[i] <= temps[i - 1] {
            temps[i] = temps[i - 1] + 1;
        }
    }
    temps
}

/// `[[temp, speed%], ...]` as sent to the daemon fan engine.
pub fn curve_to_json_pairs(curve: &[FanCurvePoint]) -> Value {
    let temps = normalized_rounded_temps(curve);
    Value::Array(
        curve
            .iter()
            .zip(temps)
            .map(|(p, t)| json!([t, (p.speed_percent.round() as i64).clamp(0, 100)]))
            .collect(),
    )
}

/// Parses `[[temp, speed], ...]`; anything but exactly 8 valid pairs yields the default curve.
pub fn curve_from_json(value: Option<&Value>) -> Vec<FanCurvePoint> {
    let Some(Value::Array(entries)) = value else {
        return default_fan_curve();
    };
    let points: Vec<FanCurvePoint> = entries
        .iter()
        .filter_map(|entry| {
            let pair = entry.as_array().filter(|a| a.len() >= 2)?;
            let temp = pair[0].as_f64()?;
            let speed = pair[1].as_f64()?;
            Some(FanCurvePoint::new(temp, speed.clamp(0.0, 100.0)))
        })
        .collect();
    if points.len() == 8 {
        points
    } else {
        default_fan_curve()
    }
}

/// Curve as returned by asusd: 8 PWM bytes (0–255) and 8 temperatures.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FanCurveData {
    pub name: String,
    pub pwm_values: Vec<u8>,
    pub temp_values: Vec<u8>,
    pub enabled: bool,
}

impl FanCurveData {
    pub fn to_curve_points(&self) -> Vec<FanCurvePoint> {
        self.temp_values
            .iter()
            .zip(&self.pwm_values)
            .map(|(t, pwm)| FanCurvePoint::new(f64::from(*t), (f64::from(*pwm) / 255.0 * 100.0).round()))
            .collect()
    }

    pub fn from_curve_points(name: &str, points: &[FanCurvePoint], enabled: bool) -> Self {
        Self {
            name: name.to_string(),
            pwm_values: points
                .iter()
                .map(|p| (p.speed_percent / 100.0 * 255.0).round().clamp(0.0, 255.0) as u8)
                .collect(),
            temp_values: normalized_rounded_temps(points)
                .into_iter()
                .map(|t| t.clamp(0, 255) as u8)
                .collect(),
            enabled,
        }
    }

    /// Constant speed: all 8 points share one PWM value.
    pub fn flat(name: &str, speed_percent: i64) -> Self {
        let pwm = (speed_percent as f64 * 255.0 / 100.0).clamp(0.0, 255.0).round() as u8;
        Self {
            name: name.to_string(),
            pwm_values: vec![pwm; 8],
            temp_values: vec![30, 40, 50, 60, 70, 80, 90, 100],
            enabled: true,
        }
    }
}

/// Configuration of the daemon dynamic fan engine (JSON shape is the daemon contract).
#[derive(Debug, Clone, PartialEq)]
pub struct FanEngineConfig {
    pub profile: PerfProfile,
    pub tick_ms: i64,
    pub hysteresis_c: i64,
    pub ramp_down_delay_ms: i64,
    pub emergency_temp_c: i64,
    pub cpu_curve: Vec<FanCurvePoint>,
    pub gpu_curve: Vec<FanCurvePoint>,
    pub cross_fan: bool,
    pub cross_fan_curve: Option<Vec<FanCurvePoint>>,
}

impl FanEngineConfig {
    pub fn new(profile: PerfProfile, cpu_curve: Vec<FanCurvePoint>, gpu_curve: Vec<FanCurvePoint>) -> Self {
        Self {
            profile,
            tick_ms: 1000,
            hysteresis_c: 2,
            ramp_down_delay_ms: 3000,
            emergency_temp_c: 95,
            cpu_curve,
            gpu_curve,
            cross_fan: false,
            cross_fan_curve: None,
        }
    }

    pub fn from_json(json: &JsonMap) -> Self {
        let int = |key: &str, default: i64| json.get(key).and_then(Value::as_i64).unwrap_or(default);
        Self {
            profile: json
                .get("profile")
                .and_then(Value::as_str)
                .and_then(PerfProfile::from_name)
                .unwrap_or(PerfProfile::Balanced),
            tick_ms: int("tickMs", 1000),
            hysteresis_c: int("hysteresisC", 2),
            ramp_down_delay_ms: int("rampDownDelayMs", 3000),
            emergency_temp_c: int("emergencyTempC", 95),
            cpu_curve: curve_from_json(json.get("cpuCurve")),
            gpu_curve: curve_from_json(json.get("gpuCurve")),
            cross_fan: json.get("crossFan") == Some(&Value::Bool(true)),
            cross_fan_curve: match json.get("crossFanCurve") {
                None | Some(Value::Null) => None,
                value => Some(curve_from_json(value)),
            },
        }
    }

    pub fn to_json(&self) -> JsonMap {
        let mut map = JsonMap::new();
        map.insert("profile".into(), json!(self.profile.name()));
        map.insert("tickMs".into(), json!(self.tick_ms));
        map.insert("hysteresisC".into(), json!(self.hysteresis_c));
        map.insert("rampDownDelayMs".into(), json!(self.ramp_down_delay_ms));
        map.insert("emergencyTempC".into(), json!(self.emergency_temp_c));
        map.insert("cpuCurve".into(), curve_to_json_pairs(&self.cpu_curve));
        map.insert("gpuCurve".into(), curve_to_json_pairs(&self.gpu_curve));
        map.insert("crossFan".into(), json!(self.cross_fan));
        map.insert(
            "crossFanCurve".into(),
            self.cross_fan_curve
                .as_deref()
                .map(curve_to_json_pairs)
                .unwrap_or(Value::Null),
        );
        map
    }

    pub fn with_curve(&self, fan: FanId, curve: Vec<FanCurvePoint>) -> Self {
        let mut next = self.clone();
        match fan {
            FanId::Cpu => next.cpu_curve = curve,
            FanId::Gpu => next.gpu_curve = curve,
        }
        next
    }

    pub fn curve(&self, fan: FanId) -> &[FanCurvePoint] {
        match fan {
            FanId::Cpu => &self.cpu_curve,
            FanId::Gpu => &self.gpu_curve,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FanInfo {
    pub id: FanId,
    pub label: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct FanTemperatures {
    pub cpu_temp_c: i32,
    pub gpu_temp_c: i32,
}
