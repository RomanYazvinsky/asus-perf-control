//! Persisted settings model. JSON shape is identical to the former Flutter app:
//! `{schemaVersion, ui: {...}, hardware: {...}}`.

use std::collections::BTreeMap;

use serde_json::{json, Value};

use super::json::*;

pub const CURRENT_SCHEMA_VERSION: i64 = 2;

#[derive(Debug, Clone, PartialEq)]
pub struct AppSettings {
    pub schema_version: i64,
    pub ui: UiSettings,
    pub hardware: HardwareSettings,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            schema_version: CURRENT_SCHEMA_VERSION,
            ui: UiSettings::default(),
            hardware: HardwareSettings::default(),
        }
    }
}

impl AppSettings {
    pub fn from_json(json: &JsonMap, issues: &mut Vec<String>) -> Self {
        let schema_version = read_int(json, "schemaVersion", "schemaVersion", CURRENT_SCHEMA_VERSION, issues);
        let ui = UiSettings::from_json(&read_map(json, "ui", issues), issues);
        let hardware = HardwareSettings::from_json(&read_map(json, "hardware", issues), issues);
        Self {
            schema_version,
            ui,
            hardware,
        }
    }

    pub fn to_json(&self) -> Value {
        json!({
            "schemaVersion": self.schema_version,
            "ui": self.ui.to_json(),
            "hardware": self.hardware.to_json(),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct UiSettings {
    pub selected_navigation_index: i64,
    pub active_hardware_profile_id: Option<String>,
}

impl UiSettings {
    pub fn from_json(json: &JsonMap, issues: &mut Vec<String>) -> Self {
        Self {
            selected_navigation_index: read_ranged_int(
                json,
                "selectedNavigationIndex",
                "ui.selectedNavigationIndex",
                (0, 1000),
                0,
                issues,
            ),
            active_hardware_profile_id: read_optional_string(
                json,
                "activeHardwareProfileId",
                "ui.activeHardwareProfileId",
                issues,
            ),
        }
    }

    pub fn to_json(&self) -> Value {
        let mut map = JsonMap::new();
        map.insert("selectedNavigationIndex".into(), json!(self.selected_navigation_index));
        if let Some(id) = &self.active_hardware_profile_id {
            map.insert("activeHardwareProfileId".into(), json!(id));
        }
        Value::Object(map)
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct HardwareSettings {
    pub performance_mode: Option<i64>,
    pub tdp: Option<TdpDesiredSettings>,
    pub cpu_boost_mode: Option<String>,
    pub battery_charge_limit: Option<i64>,
    pub gpu_mode: Option<String>,
    pub fan: Option<FanDesiredSettings>,
}

impl HardwareSettings {
    pub fn from_json(json: &JsonMap, issues: &mut Vec<String>) -> Self {
        let tdp = read_optional_map(json, "tdp", "hardware.tdp", issues)
            .map(|map| TdpDesiredSettings::from_json(&map, issues));
        let fan = read_optional_map(json, "fan", "hardware.fan", issues)
            .map(|map| FanDesiredSettings::from_json(&map, issues));
        Self {
            performance_mode: read_optional_ranged_int(
                json,
                "performanceMode",
                "hardware.performanceMode",
                (0, 2),
                issues,
            ),
            tdp,
            cpu_boost_mode: read_optional_string(json, "cpuBoostMode", "hardware.cpuBoostMode", issues),
            battery_charge_limit: read_optional_ranged_int(
                json,
                "batteryChargeLimit",
                "hardware.batteryChargeLimit",
                (0, 100),
                issues,
            ),
            gpu_mode: read_optional_string(json, "gpuMode", "hardware.gpuMode", issues),
            fan,
        }
    }

    pub fn to_json(&self) -> Value {
        let mut map = JsonMap::new();
        if let Some(v) = self.performance_mode {
            map.insert("performanceMode".into(), json!(v));
        }
        if let Some(tdp) = &self.tdp {
            map.insert("tdp".into(), tdp.to_json());
        }
        if let Some(v) = &self.cpu_boost_mode {
            map.insert("cpuBoostMode".into(), json!(v));
        }
        if let Some(v) = self.battery_charge_limit {
            map.insert("batteryChargeLimit".into(), json!(v));
        }
        if let Some(v) = &self.gpu_mode {
            map.insert("gpuMode".into(), json!(v));
        }
        if let Some(fan) = &self.fan {
            map.insert("fan".into(), fan.to_json());
        }
        Value::Object(map)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TdpDesiredSettings {
    pub power_limits_enabled: bool,
    pub sustained_w: i64,
    pub boost_w: i64,
    /// CPU temperature limit per platform profile ("quiet", "balanced", "performance");
    /// 0 means no limit. Profiles without an entry are left as the daemon has them.
    pub profile_temp_limits: BTreeMap<String, i64>,
}

impl TdpDesiredSettings {
    pub fn from_json(json: &JsonMap, issues: &mut Vec<String>) -> Self {
        Self {
            power_limits_enabled: read_bool(
                json,
                "powerLimitsEnabled",
                "hardware.tdp.powerLimitsEnabled",
                false,
                issues,
            ),
            sustained_w: read_ranged_int(json, "sustainedW", "hardware.tdp.sustainedW", (0, 150), 0, issues),
            boost_w: read_ranged_int(json, "boostW", "hardware.tdp.boostW", (0, 150), 0, issues),
            // Legacy tempLimitEnabled/tempLimitC applied to whichever profile was active,
            // so they cannot be mapped to a profile and are dropped.
            profile_temp_limits: read_optional_map(json, "profileTempLimits", "hardware.tdp.profileTempLimits", issues)
                .map(|map| {
                    // Invalid entries are skipped so they cannot clear a profile's limit.
                    map.keys()
                        .filter_map(|profile| {
                            let path = format!("hardware.tdp.profileTempLimits.{profile}");
                            read_optional_ranged_int(&map, profile, &path, (0, 105), issues)
                                .map(|limit| (profile.clone(), limit))
                        })
                        .collect()
                })
                .unwrap_or_default(),
        }
    }

    pub fn to_json(&self) -> Value {
        json!({
            "powerLimitsEnabled": self.power_limits_enabled,
            "sustainedW": self.sustained_w,
            "boostW": self.boost_w,
            "profileTempLimits": self.profile_temp_limits,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct FanDesiredSettings {
    pub selected_profile: Option<String>,
    pub selected_fan: Option<String>,
    pub control_modes: BTreeMap<String, String>,
    pub dynamic_engine_config: Option<JsonMap>,
}

impl FanDesiredSettings {
    pub fn from_json(json: &JsonMap, issues: &mut Vec<String>) -> Self {
        Self {
            selected_profile: read_optional_string(
                json,
                "selectedProfile",
                "hardware.fan.selectedProfile",
                issues,
            ),
            selected_fan: read_optional_string(json, "selectedFan", "hardware.fan.selectedFan", issues),
            control_modes: read_string_map(json, "controlModes", "hardware.fan.controlModes", issues)
                .into_iter()
                .collect(),
            dynamic_engine_config: read_optional_map(
                json,
                "dynamicEngineConfig",
                "hardware.fan.dynamicEngineConfig",
                issues,
            ),
        }
    }

    pub fn to_json(&self) -> Value {
        let mut map = JsonMap::new();
        if let Some(v) = &self.selected_profile {
            map.insert("selectedProfile".into(), json!(v));
        }
        if let Some(v) = &self.selected_fan {
            map.insert("selectedFan".into(), json!(v));
        }
        if !self.control_modes.is_empty() {
            map.insert("controlModes".into(), json!(self.control_modes));
        }
        if let Some(config) = &self.dynamic_engine_config {
            map.insert("dynamicEngineConfig".into(), Value::Object(config.clone()));
        }
        Value::Object(map)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct HardwareProfile {
    pub schema_version: i64,
    pub id: String,
    pub name: String,
    pub hardware: HardwareSettings,
}

pub const PROFILE_SCHEMA_VERSION: i64 = 1;

impl HardwareProfile {
    pub fn new(id: String, name: String, hardware: HardwareSettings) -> Self {
        Self {
            schema_version: PROFILE_SCHEMA_VERSION,
            id,
            name,
            hardware,
        }
    }

    pub fn from_json(json: &JsonMap, issues: &mut Vec<String>) -> Self {
        Self {
            schema_version: read_int(json, "schemaVersion", "schemaVersion", PROFILE_SCHEMA_VERSION, issues),
            id: read_string(json, "id", issues),
            name: read_string(json, "name", issues),
            hardware: HardwareSettings::from_json(&read_map(json, "hardware", issues), issues),
        }
    }

    pub fn to_json(&self) -> Value {
        json!({
            "schemaVersion": self.schema_version,
            "id": self.id,
            "name": self.name,
            "hardware": self.hardware.to_json(),
        })
    }
}
