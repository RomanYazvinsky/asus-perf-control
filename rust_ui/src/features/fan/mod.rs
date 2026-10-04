//! Fan control — former `FanBloc` and `FanServiceImpl` contract.
//!
//! Three control paths: asusd curves (default), constant speed (flat asusd curve) and the
//! daemon's dynamic engine.

pub mod model;

use std::collections::BTreeMap;

use anyhow::Result;
use async_trait::async_trait;

pub use model::*;

use crate::logging::{log_hardware_mismatch, log_read_mismatch};
use crate::settings::{update_hardware, AppSettingsStore, FanDesiredSettings};

#[async_trait]
pub trait FanService: Send + Sync {
    async fn capabilities(&self) -> Result<Vec<FanInfo>>;
    async fn fan_rpms(&self) -> Result<Vec<(FanId, Option<u32>)>>;
    async fn temperatures(&self) -> Result<FanTemperatures>;
    /// asusd platform profile currently in effect.
    async fn active_profile(&self) -> Result<PerfProfile>;
    async fn asusd_curves(&self, profile: PerfProfile) -> Result<Vec<FanCurveData>>;
    async fn set_asusd_curve(&self, profile: PerfProfile, fan: FanId, points: &[FanCurvePoint]) -> Result<()>;
    async fn set_constant_speed(&self, profile: PerfProfile, fan: FanId, speed_percent: i64) -> Result<()>;
    async fn reset_to_defaults(&self, profile: PerfProfile) -> Result<()>;
    async fn start_dynamic_engine(&self, config: &FanEngineConfig) -> Result<()>;
    async fn stop_dynamic_engine(&self) -> Result<()>;
    async fn update_dynamic_curve(&self, fan: FanId, curve: &[FanCurvePoint]) -> Result<()>;
    async fn is_dynamic_engine_running(&self) -> Result<bool>;
    /// Runs the fan at 100 % for 2 s, returns the RPM reached and restores the previous curve.
    async fn run_max_speed_test(&self, profile: PerfProfile, fan: FanId) -> Result<u32>;
}

pub type FanProfileCurves = BTreeMap<FanId, BTreeMap<PerfProfile, FanCurveData>>;

#[derive(Debug, Clone, PartialEq)]
pub struct FanLoaded {
    pub fans: Vec<FanInfo>,
    pub fan_rpms: BTreeMap<FanId, u32>,
    pub cpu_temp_c: i32,
    pub gpu_temp_c: i32,
    pub asusd_curves: FanProfileCurves,
    pub selected_fan: FanId,
    /// Profile whose curves are being edited; does not switch the active profile.
    pub selected_profile: PerfProfile,
    pub active_profile: Option<PerfProfile>,
    pub control_modes: BTreeMap<FanId, FanControlMode>,
    pub dynamic_engine_running: bool,
    pub engine_config: Option<FanEngineConfig>,
}

impl FanLoaded {
    pub fn fan_ids(&self) -> Vec<FanId> {
        FanId::ALL
            .into_iter()
            .filter(|id| self.fans.iter().any(|f| f.id == *id))
            .collect()
    }

    pub fn control_mode(&self, fan: FanId) -> FanControlMode {
        self.control_modes.get(&fan).copied().unwrap_or(FanControlMode::AsusctlCurve)
    }

    /// asusd curve of the selected profile with the safety clamp, or the default curve.
    pub fn profile_curve(&self, fan: FanId) -> Vec<FanCurvePoint> {
        let points = self
            .asusd_curves
            .get(&fan)
            .and_then(|curves| curves.get(&self.selected_profile))
            .map(FanCurveData::to_curve_points)
            .unwrap_or_else(default_fan_curve);
        with_safety_clamp(&points)
    }

    /// Curve shown in the editor: engine curve in dynamic mode, else the asusd curve.
    pub fn editor_curve(&self, fan: FanId) -> Vec<FanCurvePoint> {
        if self.control_mode(fan) == FanControlMode::DynamicEngine {
            if let Some(config) = &self.engine_config {
                return config.curve(fan).to_vec();
            }
        }
        self.asusd_curves
            .get(&fan)
            .and_then(|curves| curves.get(&self.selected_profile))
            .map(FanCurveData::to_curve_points)
            .unwrap_or_else(default_fan_curve)
    }

    /// Engine configuration built from the selected profile's curves.
    pub fn build_engine_config(&self) -> FanEngineConfig {
        FanEngineConfig::new(
            self.selected_profile,
            self.profile_curve(FanId::Cpu),
            self.profile_curve(FanId::Gpu),
        )
    }

    fn with_all_modes(&self, mode: FanControlMode) -> BTreeMap<FanId, FanControlMode> {
        let mut modes = self.control_modes.clone();
        for id in self.fan_ids() {
            modes.insert(id, mode);
        }
        modes
    }

    fn with_curve(&self, profile: PerfProfile, fan: FanId, curve: &[FanCurvePoint]) -> FanProfileCurves {
        let mut curves = self.asusd_curves.clone();
        curves
            .entry(fan)
            .or_default()
            .insert(profile, FanCurveData::from_curve_points(fan.asusd_name(), curve, true));
        curves
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum FanState {
    Initial,
    Loading,
    Loaded(FanLoaded),
    Unsupported(String),
    Error(String),
    MaxSpeedTesting { fan: FanId },
    MaxSpeedResult { previous: FanLoaded, fan: FanId, max_rpm: u32 },
}

fn rpm_map(readings: Vec<(FanId, Option<u32>)>) -> BTreeMap<FanId, u32> {
    readings.into_iter().map(|(id, rpm)| (id, rpm.unwrap_or(0))).collect()
}

fn save_fan_settings(store: &dyn AppSettingsStore, state: &FanLoaded, config: Option<&FanEngineConfig>) -> Result<()> {
    let fan = FanDesiredSettings {
        selected_profile: Some(state.selected_profile.name().into()),
        selected_fan: Some(state.selected_fan.name().into()),
        control_modes: state
            .control_modes
            .iter()
            .map(|(id, mode)| (id.name().to_string(), mode.name().to_string()))
            .collect(),
        dynamic_engine_config: config.map(FanEngineConfig::to_json),
    };
    update_hardware(store, |hw| hw.fan = Some(fan.clone()))?;
    Ok(())
}

pub async fn load(service: &dyn FanService, store: &dyn AppSettingsStore) -> FanState {
    match load_inner(service, store).await {
        Ok(state) => state,
        Err(e) => FanState::Error(format!("Failed to load fan capabilities: {e:#}")),
    }
}

async fn load_inner(service: &dyn FanService, store: &dyn AppSettingsStore) -> Result<FanState> {
    let fans = service.capabilities().await?;
    let Some(first_fan) = fans.first().map(|f| f.id) else {
        return Ok(FanState::Unsupported("No fans detected".into()));
    };
    let fan_rpms = rpm_map(service.fan_rpms().await?);
    let temperatures = service.temperatures().await.unwrap_or_default();
    let mut dynamic_engine_running = service.is_dynamic_engine_running().await.unwrap_or(false);
    let saved = store.load().hardware.fan;

    let available: Vec<FanId> = fans.iter().map(|f| f.id).collect();
    let mut asusd_curves: FanProfileCurves = available.iter().map(|id| (*id, BTreeMap::new())).collect();
    for profile in PerfProfile::ALL {
        // asusd may not know every profile; keep the curves of the others.
        if let Ok(curves) = service.asusd_curves(profile).await {
            for curve in curves {
                if let Some(id) = FanId::from_curve_name(&curve.name) {
                    if let Some(per_profile) = asusd_curves.get_mut(&id) {
                        per_profile.insert(profile, curve);
                    }
                }
            }
        }
    }

    let fan_from_saved = |name: &str| {
        let fan = FanId::from_name(name).filter(|id| available.contains(id));
        if fan.is_none() {
            log_read_mismatch("hardware.fan.selectedFan", name);
        }
        fan
    };
    let selected_fan = saved
        .as_ref()
        .and_then(|s| s.selected_fan.as_deref())
        .and_then(fan_from_saved)
        .unwrap_or(first_fan);
    let selected_profile = saved
        .as_ref()
        .and_then(|s| s.selected_profile.as_deref())
        .and_then(|name| {
            let profile = PerfProfile::from_name(name);
            if profile.is_none() {
                log_read_mismatch("hardware.fan.selectedProfile", name);
            }
            profile
        })
        .unwrap_or(PerfProfile::Balanced);

    let mut saved_modes = BTreeMap::new();
    for (name, mode_name) in saved.iter().flat_map(|s| s.control_modes.iter()) {
        let Some(id) = fan_from_saved(name) else { continue };
        match FanControlMode::from_name(mode_name) {
            Some(mode) => {
                saved_modes.insert(id, mode);
            }
            None => log_read_mismatch("hardware.fan.controlModes", mode_name),
        }
    }
    let saved_config = saved
        .as_ref()
        .and_then(|s| s.dynamic_engine_config.as_ref())
        .map(FanEngineConfig::from_json);

    if !dynamic_engine_running && saved_modes.values().any(|m| *m == FanControlMode::DynamicEngine) {
        if let Some(config) = &saved_config {
            log_hardware_mismatch("fan.dynamicEngine", "running", "stopped");
            service.start_dynamic_engine(config).await?;
            dynamic_engine_running = true;
        }
    }
    let control_modes = if saved_modes.is_empty() {
        let mode = if dynamic_engine_running {
            FanControlMode::DynamicEngine
        } else {
            FanControlMode::AsusctlCurve
        };
        available.iter().map(|id| (*id, mode)).collect()
    } else {
        saved_modes
    };

    let loaded = FanLoaded {
        fans,
        fan_rpms,
        cpu_temp_c: temperatures.cpu_temp_c,
        gpu_temp_c: temperatures.gpu_temp_c,
        asusd_curves,
        selected_fan,
        selected_profile,
        active_profile: service.active_profile().await.ok(),
        control_modes,
        dynamic_engine_running,
        engine_config: saved_config,
    };
    if saved.is_none() {
        save_fan_settings(store, &loaded, loaded.engine_config.as_ref())?;
    }
    Ok(FanState::Loaded(loaded))
}

/// Live values refreshed every 2 s.
#[derive(Debug, Clone, PartialEq)]
pub struct FanRefresh {
    pub fan_rpms: BTreeMap<FanId, u32>,
    pub temperatures: FanTemperatures,
    pub dynamic_engine_running: bool,
    pub active_profile: Option<PerfProfile>,
}

pub async fn refresh(service: &dyn FanService) -> Result<FanRefresh> {
    Ok(FanRefresh {
        fan_rpms: rpm_map(service.fan_rpms().await?),
        temperatures: service.temperatures().await.unwrap_or_default(),
        dynamic_engine_running: service.is_dynamic_engine_running().await.unwrap_or(false),
        active_profile: service.active_profile().await.ok(),
    })
}

/// When the engine stopped behind our back, dynamic fans fall back to asusd curves.
pub fn apply_refresh(state: &mut FanLoaded, refresh: FanRefresh) {
    state.fan_rpms = refresh.fan_rpms;
    state.cpu_temp_c = refresh.temperatures.cpu_temp_c;
    state.gpu_temp_c = refresh.temperatures.gpu_temp_c;
    state.dynamic_engine_running = refresh.dynamic_engine_running;
    if refresh.active_profile.is_some() {
        state.active_profile = refresh.active_profile;
    }
    if !refresh.dynamic_engine_running {
        for mode in state.control_modes.values_mut() {
            if *mode == FanControlMode::DynamicEngine {
                *mode = FanControlMode::AsusctlCurve;
            }
        }
        state.engine_config = None;
    }
}

fn fan_error(prefix: &str, e: anyhow::Error) -> FanState {
    FanState::Error(format!("{prefix}: {e:#}"))
}

pub fn select_fan(store: &dyn AppSettingsStore, state: &FanLoaded, fan: FanId) -> Result<FanLoaded> {
    let next = FanLoaded {
        selected_fan: fan,
        ..state.clone()
    };
    save_fan_settings(store, &next, state.engine_config.as_ref())?;
    Ok(next)
}

/// Picks which profile's curves are edited; the active platform profile is left alone.
pub fn select_profile(store: &dyn AppSettingsStore, state: FanLoaded, profile: PerfProfile) -> FanState {
    let next = FanLoaded {
        selected_profile: profile,
        ..state
    };
    match save_fan_settings(store, &next, next.engine_config.as_ref()) {
        Ok(()) => FanState::Loaded(next),
        Err(e) => fan_error("Failed to save selected profile", e),
    }
}

pub async fn set_asusctl_curve(
    service: &dyn FanService,
    store: &dyn AppSettingsStore,
    state: FanLoaded,
    fan: FanId,
    profile: PerfProfile,
    curve: Vec<FanCurvePoint>,
) -> FanState {
    let result: Result<FanLoaded> = async {
        service.set_asusd_curve(profile, fan, &curve).await?;
        let mut control_modes = state.control_modes.clone();
        control_modes.insert(fan, FanControlMode::AsusctlCurve);
        let next = FanLoaded {
            control_modes,
            asusd_curves: state.with_curve(profile, fan, &curve),
            ..state
        };
        save_fan_settings(store, &next, next.engine_config.as_ref())?;
        Ok(next)
    }
    .await;
    result
        .map(FanState::Loaded)
        .unwrap_or_else(|e| fan_error("Failed to set asusd curve", e))
}

pub async fn set_constant_speed(
    service: &dyn FanService,
    store: &dyn AppSettingsStore,
    state: FanLoaded,
    fan: FanId,
    profile: PerfProfile,
    speed_percent: i64,
) -> FanState {
    let result: Result<FanLoaded> = async {
        service.set_constant_speed(profile, fan, speed_percent).await?;
        let flat: Vec<FanCurvePoint> = (0..8)
            .map(|i| FanCurvePoint::new(30.0 + i as f64 * 10.0, speed_percent as f64))
            .collect();
        let mut control_modes = state.control_modes.clone();
        control_modes.insert(fan, FanControlMode::ConstantSpeed);
        let next = FanLoaded {
            control_modes,
            asusd_curves: state.with_curve(profile, fan, &flat),
            ..state
        };
        save_fan_settings(store, &next, next.engine_config.as_ref())?;
        Ok(next)
    }
    .await;
    result
        .map(FanState::Loaded)
        .unwrap_or_else(|e| fan_error("Failed to set constant speed", e))
}

pub async fn start_dynamic_engine(
    service: &dyn FanService,
    store: &dyn AppSettingsStore,
    state: FanLoaded,
    config: FanEngineConfig,
) -> FanState {
    let result: Result<FanLoaded> = async {
        service.start_dynamic_engine(&config).await?;
        let next = FanLoaded {
            control_modes: state.with_all_modes(FanControlMode::DynamicEngine),
            dynamic_engine_running: true,
            engine_config: Some(config.clone()),
            ..state
        };
        save_fan_settings(store, &next, Some(&config))?;
        Ok(next)
    }
    .await;
    result
        .map(FanState::Loaded)
        .unwrap_or_else(|e| fan_error("Failed to start dynamic engine", e))
}

pub async fn stop_dynamic_engine(service: &dyn FanService, store: &dyn AppSettingsStore, state: FanLoaded) -> FanState {
    let result: Result<FanLoaded> = async {
        service.stop_dynamic_engine().await?;
        let next = FanLoaded {
            control_modes: state.with_all_modes(FanControlMode::AsusctlCurve),
            dynamic_engine_running: false,
            engine_config: None,
            ..state
        };
        save_fan_settings(store, &next, None)?;
        Ok(next)
    }
    .await;
    result
        .map(FanState::Loaded)
        .unwrap_or_else(|e| fan_error("Failed to stop dynamic engine", e))
}

pub async fn update_dynamic_curve(
    service: &dyn FanService,
    store: &dyn AppSettingsStore,
    state: FanLoaded,
    fan: FanId,
    curve: Vec<FanCurvePoint>,
) -> FanState {
    let result: Result<FanLoaded> = async {
        service.update_dynamic_curve(fan, &curve).await?;
        let config = state
            .engine_config
            .clone()
            .unwrap_or_else(|| state.build_engine_config())
            .with_curve(fan, curve);
        let next = FanLoaded {
            engine_config: Some(config.clone()),
            ..state
        };
        save_fan_settings(store, &next, Some(&config))?;
        Ok(next)
    }
    .await;
    result
        .map(FanState::Loaded)
        .unwrap_or_else(|e| fan_error("Failed to update dynamic curve", e))
}

/// Applies edited curves of several fans in order; stops at the first failure.
/// Dynamic-engine fans update the engine, the others the selected profile's asusd curve.
pub async fn apply_curves(
    service: &dyn FanService,
    store: &dyn AppSettingsStore,
    state: FanLoaded,
    curves: Vec<(FanId, Vec<FanCurvePoint>)>,
) -> FanState {
    let mut state = state;
    for (fan, curve) in curves {
        let next = match state.control_mode(fan) {
            FanControlMode::DynamicEngine => update_dynamic_curve(service, store, state, fan, curve).await,
            _ => {
                let profile = state.selected_profile;
                set_asusctl_curve(service, store, state, fan, profile, curve).await
            }
        };
        match next {
            FanState::Loaded(loaded) => state = loaded,
            other => return other,
        }
    }
    FanState::Loaded(state)
}

pub async fn reset_to_defaults(
    service: &dyn FanService,
    store: &dyn AppSettingsStore,
    state: FanLoaded,
    profile: PerfProfile,
) -> FanState {
    let result: Result<FanLoaded> = async {
        service.reset_to_defaults(profile).await?;
        let mut asusd_curves = state.asusd_curves.clone();
        // If the reload fails the previous curves stay visible.
        if let Ok(curves) = service.asusd_curves(profile).await {
            for curve in curves {
                if let Some(id) = FanId::from_curve_name(&curve.name) {
                    asusd_curves.entry(id).or_default().insert(profile, curve);
                }
            }
        }
        let next = FanLoaded {
            control_modes: state.with_all_modes(FanControlMode::AsusctlCurve),
            asusd_curves,
            ..state
        };
        save_fan_settings(store, &next, next.engine_config.as_ref())?;
        Ok(next)
    }
    .await;
    result
        .map(FanState::Loaded)
        .unwrap_or_else(|e| fan_error("Failed to reset to defaults", e))
}

/// Returns the result state, or `(error, previous)` so the screen can show the error and restore.
pub async fn max_speed_test(service: &dyn FanService, state: FanLoaded, fan: FanId) -> Result<FanState, Box<(String, FanLoaded)>> {
    // The edited profile may not be active; test on the one actually driving the fans.
    let profile = service
        .active_profile()
        .await
        .ok()
        .or(state.active_profile)
        .unwrap_or(state.selected_profile);
    match service.run_max_speed_test(profile, fan).await {
        Ok(max_rpm) => Ok(FanState::MaxSpeedResult {
            previous: state,
            fan,
            max_rpm,
        }),
        Err(e) => Err(Box::new((format!("Max speed test failed: {e:#}"), state))),
    }
}

/// Editor drag/keyboard update: keeps temperatures ordered (≥1 °C apart) and the curve
/// monotonic by pulling neighbours along.
pub fn update_point_position(points: &mut [FanCurvePoint], index: usize, temp: f64, speed: f64) {
    if index >= points.len() {
        return;
    }
    let mut new_temp = temp.clamp(20.0, 100.0);
    let new_speed = speed.clamp(0.0, 100.0);

    if index > 0 {
        let min_temp = points[index - 1].temp_c.round() + 1.0;
        new_temp = new_temp.clamp(min_temp.min(100.0), 100.0);
    }
    if index + 1 < points.len() {
        let max_temp = points[index + 1].temp_c.round() - 1.0;
        if max_temp >= 20.0 {
            new_temp = new_temp.clamp(20.0_f64.min(max_temp), max_temp);
        }
    }
    points[index] = FanCurvePoint::new(new_temp, new_speed);

    for i in (0..index).rev() {
        if points[i].speed_percent > new_speed {
            points[i].speed_percent = new_speed;
        } else {
            break;
        }
    }
    for point in points.iter_mut().skip(index + 1) {
        if point.speed_percent < new_speed {
            point.speed_percent = new_speed;
        } else {
            break;
        }
    }
}

#[cfg(test)]
mod tests;
