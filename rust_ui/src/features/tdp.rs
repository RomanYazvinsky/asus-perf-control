//! CPU power limits and temperature ceiling — former `TdpBloc`.
//!
//! The profile tabs pick which platform profile is edited; they never switch the
//! active profile. Temperature limits are stored per profile by the daemon, while
//! asusd power limits can only be edited for the active profile.

use anyhow::Result;
use async_trait::async_trait;

use crate::logging::log_hardware_mismatch;
use crate::settings::{update_hardware, AppSettingsStore, TdpDesiredSettings};

/// asusd TDP values and the active profile.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TdpConfig {
    pub is_supported: bool,
    pub sustained_w: i32,
    pub boost_w: i32,
    pub active_performance_mode: i32,
    pub min_sustained_w: i32,
    pub max_sustained_w: i32,
    pub min_boost_w: i32,
    pub max_boost_w: i32,
    pub power_limits_enabled: bool,
}

impl Default for TdpConfig {
    fn default() -> Self {
        Self {
            is_supported: true,
            sustained_w: 0,
            boost_w: 0,
            active_performance_mode: 1,
            min_sustained_w: 5,
            max_sustained_w: 54,
            min_boost_w: 5,
            max_boost_w: 54,
            power_limits_enabled: false,
        }
    }
}

#[async_trait]
pub trait TdpService: Send + Sync {
    async fn get_tdp_config(&self) -> Result<TdpConfig>;
    async fn set_power_limits(&self, sustained_w: i32, boost_w: i32) -> Result<()>;
    async fn clear_power_limits(&self) -> Result<()>;
    /// Saved limit for a UI mode (0 = Silent, 1 = Balanced, 2 = Turbo); 0 °C = none.
    async fn get_profile_temp_limit(&self, mode: i32) -> Result<i32>;
    /// Saves the limit for a UI mode; the daemon applies it only while that profile is active.
    async fn set_profile_temp_limit(&self, mode: i32, temp_limit_c: i32) -> Result<()>;
}

/// Daemon profile name for a UI mode.
pub fn daemon_profile_name(mode: i32) -> &'static str {
    match mode {
        0 => "Quiet",
        2 => "Performance",
        _ => "Balanced",
    }
}

/// Settings key for a UI mode.
fn settings_profile_key(mode: i32) -> &'static str {
    match mode {
        0 => "quiet",
        2 => "performance",
        _ => "balanced",
    }
}

fn mode_from_settings_key(key: &str) -> Option<i32> {
    match key {
        "quiet" => Some(0),
        "balanced" => Some(1),
        "performance" => Some(2),
        _ => None,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TdpLoaded {
    pub sustained_w: i32,
    pub boost_w: i32,
    /// Temperature limit of `editing_mode`.
    pub temp_limit_c: i32,
    pub active_performance_mode: i32,
    /// Profile shown and edited on the page.
    pub editing_mode: i32,
    pub min_w: i32,
    pub max_w: i32,
    pub min_boost_w: i32,
    pub max_boost_w: i32,
    pub power_limits_enabled: bool,
    pub temp_limit_enabled: bool,
}

impl TdpLoaded {
    pub fn from_config(config: &TdpConfig, editing_mode: i32, temp_limit_c: i32) -> Self {
        Self {
            sustained_w: config.sustained_w,
            boost_w: config.boost_w,
            temp_limit_c,
            active_performance_mode: config.active_performance_mode,
            editing_mode,
            min_w: config.min_sustained_w,
            max_w: config.max_sustained_w,
            min_boost_w: config.min_boost_w,
            max_boost_w: config.max_boost_w,
            power_limits_enabled: config.power_limits_enabled,
            temp_limit_enabled: temp_limit_c > 0,
        }
    }

    pub fn editing_active_profile(&self) -> bool {
        self.editing_mode == self.active_performance_mode
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TdpState {
    Initial,
    Loading,
    Loaded(TdpLoaded),
    Unsupported,
    Error(String),
}

fn error_state(e: anyhow::Error) -> TdpState {
    TdpState::Error(format!("{e:#}"))
}

/// Reads asusd limits and the temperature limit of `editing_mode` (the active profile when None).
async fn read_state(service: &dyn TdpService, editing_mode: Option<i32>) -> Result<TdpState> {
    let config = service.get_tdp_config().await?;
    if !config.is_supported {
        return Ok(TdpState::Unsupported);
    }
    let editing_mode = editing_mode.unwrap_or(config.active_performance_mode);
    // Non-critical: daemon down means "no saved limit".
    let temp_limit_c = service.get_profile_temp_limit(editing_mode).await.unwrap_or(0);
    Ok(TdpState::Loaded(TdpLoaded::from_config(&config, editing_mode, temp_limit_c)))
}

pub async fn load(service: &dyn TdpService, store: &dyn AppSettingsStore) -> TdpState {
    let result: Result<TdpState> = async {
        restore_saved_tdp_if_needed(service, store).await?;
        read_state(service, None).await
    }
    .await;
    result.unwrap_or_else(error_state)
}

/// Shows another profile's settings for editing; the active profile is not changed.
pub async fn select_profile(service: &dyn TdpService, mode: i32) -> TdpState {
    read_state(service, Some(mode)).await.unwrap_or_else(error_state)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SetTdpRequest {
    pub editing_mode: i32,
    /// Ignored unless `editing_mode` is the active profile.
    pub apply_power_limits: bool,
    /// When false the edited profile's temperature limit is cleared.
    pub temp_limit_enabled: bool,
    pub sustained_w: i32,
    pub boost_w: i32,
    pub temp_limit_c: i32,
}

pub async fn set_tdp(service: &dyn TdpService, store: &dyn AppSettingsStore, request: SetTdpRequest) -> TdpState {
    let result: Result<TdpState> = async {
        let active_mode = service.get_tdp_config().await?.active_performance_mode;
        let apply_power_limits = request.apply_power_limits && request.editing_mode == active_mode;
        if apply_power_limits {
            service.set_power_limits(request.sustained_w, request.boost_w).await?;
        }
        let temp_limit_c = if request.temp_limit_enabled { request.temp_limit_c } else { 0 };
        service.set_profile_temp_limit(request.editing_mode, temp_limit_c).await?;
        update_hardware(store, |hw| {
            let mut tdp = hw.tdp.clone().unwrap_or(TdpDesiredSettings {
                power_limits_enabled: false,
                sustained_w: 0,
                boost_w: 0,
                profile_temp_limits: Default::default(),
            });
            if apply_power_limits {
                tdp.power_limits_enabled = true;
                tdp.sustained_w = i64::from(request.sustained_w);
                tdp.boost_w = i64::from(request.boost_w);
            }
            tdp.profile_temp_limits
                .insert(settings_profile_key(request.editing_mode).into(), i64::from(temp_limit_c));
            hw.tdp = Some(tdp);
        })?;
        read_state(service, Some(request.editing_mode)).await
    }
    .await;
    result.unwrap_or_else(error_state)
}

/// "Reload from Hardware": discards UI edits.
pub async fn reload(service: &dyn TdpService, editing_mode: Option<i32>) -> TdpState {
    read_state(service, editing_mode).await.unwrap_or_else(error_state)
}

pub async fn clear_power_limits(
    service: &dyn TdpService,
    store: &dyn AppSettingsStore,
    previous: Option<TdpLoaded>,
) -> TdpState {
    let prev = previous.as_ref();
    let result: Result<TdpState> = async {
        service.clear_power_limits().await?;
        update_hardware(store, |hw| {
            let saved = hw.tdp.clone();
            hw.tdp = Some(TdpDesiredSettings {
                power_limits_enabled: false,
                sustained_w: prev
                    .map(|p| i64::from(p.sustained_w))
                    .or(saved.as_ref().map(|t| t.sustained_w))
                    .unwrap_or(0),
                boost_w: prev
                    .map(|p| i64::from(p.boost_w))
                    .or(saved.as_ref().map(|t| t.boost_w))
                    .unwrap_or(0),
                profile_temp_limits: saved.map(|t| t.profile_temp_limits).unwrap_or_default(),
            });
        })?;
        let state = read_state(service, prev.map(|p| p.editing_mode)).await?;
        // Keep the slider values so re-enabling starts from them.
        Ok(match (state, prev) {
            (TdpState::Loaded(loaded), Some(p)) => TdpState::Loaded(TdpLoaded {
                sustained_w: p.sustained_w,
                boost_w: p.boost_w,
                power_limits_enabled: false,
                ..loaded
            }),
            (state, _) => state,
        })
    }
    .await;
    result.unwrap_or_else(error_state)
}

async fn restore_saved_tdp_if_needed(service: &dyn TdpService, store: &dyn AppSettingsStore) -> Result<()> {
    let config = service.get_tdp_config().await?;
    if !config.is_supported {
        return Ok(());
    }
    let Some(desired) = store.load().hardware.tdp else {
        let mut profile_temp_limits = std::collections::BTreeMap::new();
        for mode in 0..=2 {
            if let Ok(limit) = service.get_profile_temp_limit(mode).await {
                profile_temp_limits.insert(settings_profile_key(mode).into(), i64::from(limit));
            }
        }
        update_hardware(store, |hw| {
            hw.tdp = Some(TdpDesiredSettings {
                power_limits_enabled: config.power_limits_enabled,
                sustained_w: i64::from(config.sustained_w),
                boost_w: i64::from(config.boost_w),
                profile_temp_limits: profile_temp_limits.clone(),
            })
        })?;
        return Ok(());
    };

    let power_mismatch = desired.power_limits_enabled != config.power_limits_enabled
        || (desired.power_limits_enabled
            && (desired.sustained_w != i64::from(config.sustained_w) || desired.boost_w != i64::from(config.boost_w)));
    if power_mismatch {
        log_hardware_mismatch(
            "tdp.powerLimits",
            format!("{}:{}/{}", desired.power_limits_enabled, desired.sustained_w, desired.boost_w),
            format!("{}:{}/{}", config.power_limits_enabled, config.sustained_w, config.boost_w),
        );
        if desired.power_limits_enabled {
            service
                .set_power_limits(desired.sustained_w as i32, desired.boost_w as i32)
                .await?;
        } else {
            service.clear_power_limits().await?;
        }
    }

    for (key, desired_temp) in &desired.profile_temp_limits {
        let Some(mode) = mode_from_settings_key(key) else { continue };
        let desired_temp = *desired_temp as i32;
        let current = service.get_profile_temp_limit(mode).await?;
        if desired_temp != current {
            log_hardware_mismatch(&format!("tdp.profileTempLimits.{key}"), desired_temp, current);
            service.set_profile_temp_limit(mode, desired_temp).await?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::sync::Mutex;

    use super::*;
    use crate::settings::{AppSettings, HardwareSettings, MemorySettingsStore};

    #[derive(Default)]
    struct Fake {
        config: Mutex<TdpConfig>,
        temps: Mutex<BTreeMap<i32, i32>>,
        calls: Mutex<Vec<String>>,
    }

    #[async_trait]
    impl TdpService for Fake {
        async fn get_tdp_config(&self) -> Result<TdpConfig> {
            Ok(self.config.lock().unwrap().clone())
        }
        async fn set_power_limits(&self, sustained_w: i32, boost_w: i32) -> Result<()> {
            self.calls.lock().unwrap().push(format!("power {sustained_w}/{boost_w}"));
            let mut config = self.config.lock().unwrap();
            config.sustained_w = sustained_w;
            config.boost_w = boost_w;
            config.power_limits_enabled = true;
            Ok(())
        }
        async fn clear_power_limits(&self) -> Result<()> {
            self.calls.lock().unwrap().push("clear".into());
            self.config.lock().unwrap().power_limits_enabled = false;
            Ok(())
        }
        async fn get_profile_temp_limit(&self, mode: i32) -> Result<i32> {
            Ok(self.temps.lock().unwrap().get(&mode).copied().unwrap_or(0))
        }
        async fn set_profile_temp_limit(&self, mode: i32, temp_limit_c: i32) -> Result<()> {
            self.calls.lock().unwrap().push(format!("temp {mode} {temp_limit_c}"));
            self.temps.lock().unwrap().insert(mode, temp_limit_c);
            Ok(())
        }
    }

    fn fake(config: TdpConfig, temps: &[(i32, i32)]) -> Fake {
        Fake {
            config: Mutex::new(config),
            temps: Mutex::new(temps.iter().copied().collect()),
            calls: Mutex::default(),
        }
    }

    fn store(tdp: Option<TdpDesiredSettings>) -> MemorySettingsStore {
        MemorySettingsStore::new(AppSettings {
            hardware: HardwareSettings {
                tdp,
                ..Default::default()
            },
            ..Default::default()
        })
    }

    fn config(sustained: i32, boost: i32, enabled: bool, active_mode: i32) -> TdpConfig {
        TdpConfig {
            sustained_w: sustained,
            boost_w: boost,
            power_limits_enabled: enabled,
            active_performance_mode: active_mode,
            ..TdpConfig::default()
        }
    }

    fn loaded(state: TdpState) -> TdpLoaded {
        match state {
            TdpState::Loaded(loaded) => loaded,
            other => panic!("expected Loaded, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn first_load_saves_hardware_values_for_every_profile() {
        let service = fake(config(25, 35, true, 2), &[(2, 85), (0, 70)]);
        let store = store(None);

        let state = loaded(load(&service, &store).await);

        assert_eq!((state.editing_mode, state.temp_limit_c, state.sustained_w), (2, 85, 25));
        assert!(state.temp_limit_enabled);
        let saved = store.load().hardware.tdp.unwrap();
        assert!(saved.power_limits_enabled);
        assert_eq!(
            saved.profile_temp_limits,
            BTreeMap::from([("quiet".into(), 70), ("balanced".into(), 0), ("performance".into(), 85)])
        );
        assert!(service.calls.lock().unwrap().is_empty());
    }

    #[tokio::test]
    async fn restores_saved_power_and_per_profile_temp_limits() {
        let service = fake(config(15, 20, false, 1), &[(0, 90)]);
        let store = store(Some(TdpDesiredSettings {
            power_limits_enabled: true,
            sustained_w: 30,
            boost_w: 40,
            profile_temp_limits: BTreeMap::from([("quiet".into(), 70), ("balanced".into(), 80)]),
        }));

        let state = loaded(load(&service, &store).await);

        assert_eq!(*service.calls.lock().unwrap(), vec!["power 30/40", "temp 1 80", "temp 0 70"]);
        assert_eq!((state.sustained_w, state.temp_limit_c, state.editing_mode), (30, 80, 1));
    }

    #[tokio::test]
    async fn restores_disabled_power_limits_by_clearing() {
        let service = fake(config(15, 20, true, 1), &[]);
        let store = store(Some(TdpDesiredSettings {
            power_limits_enabled: false,
            sustained_w: 0,
            boost_w: 0,
            profile_temp_limits: BTreeMap::new(),
        }));

        load(&service, &store).await;

        assert_eq!(*service.calls.lock().unwrap(), vec!["clear"]);
    }

    #[tokio::test]
    async fn select_profile_loads_that_profile_without_switching() {
        let service = fake(config(25, 35, true, 1), &[(0, 70), (1, 85)]);

        let state = loaded(select_profile(&service, 0).await);

        assert_eq!((state.editing_mode, state.active_performance_mode, state.temp_limit_c), (0, 1, 70));
        assert!(!state.editing_active_profile());
        assert!(service.calls.lock().unwrap().is_empty());
    }

    #[tokio::test]
    async fn set_tdp_for_inactive_profile_saves_temp_only() {
        let service = fake(config(15, 20, false, 1), &[]);
        let store = store(None);

        let state = loaded(
            set_tdp(
                &service,
                &store,
                SetTdpRequest {
                    editing_mode: 2,
                    apply_power_limits: true,
                    temp_limit_enabled: true,
                    sustained_w: 28,
                    boost_w: 33,
                    temp_limit_c: 75,
                },
            )
            .await,
        );

        assert_eq!(*service.calls.lock().unwrap(), vec!["temp 2 75"]);
        assert_eq!((state.editing_mode, state.temp_limit_c, state.sustained_w), (2, 75, 15));
        let saved = store.load().hardware.tdp.unwrap();
        assert!(!saved.power_limits_enabled);
        assert_eq!(saved.profile_temp_limits, BTreeMap::from([("performance".into(), 75)]));
    }

    #[tokio::test]
    async fn set_tdp_for_active_profile_applies_power_and_clears_disabled_temp() {
        let service = fake(config(15, 20, false, 1), &[(1, 85)]);
        let store = store(None);

        let state = loaded(
            set_tdp(
                &service,
                &store,
                SetTdpRequest {
                    editing_mode: 1,
                    apply_power_limits: true,
                    temp_limit_enabled: false,
                    sustained_w: 28,
                    boost_w: 33,
                    temp_limit_c: 70,
                },
            )
            .await,
        );

        assert_eq!(*service.calls.lock().unwrap(), vec!["power 28/33", "temp 1 0"]);
        assert_eq!((state.sustained_w, state.boost_w, state.temp_limit_c), (28, 33, 0));
        assert!(!state.temp_limit_enabled);
        let saved = store.load().hardware.tdp.unwrap();
        assert!(saved.power_limits_enabled);
        assert_eq!(saved.profile_temp_limits, BTreeMap::from([("balanced".into(), 0)]));
    }

    #[tokio::test]
    async fn clear_power_limits_keeps_previous_values_in_settings() {
        let service = fake(config(25, 35, true, 1), &[(1, 85)]);
        let store = store(None);
        let previous = TdpLoaded::from_config(&config(25, 35, true, 1), 1, 85);

        let state = loaded(clear_power_limits(&service, &store, Some(previous)).await);

        assert!(!state.power_limits_enabled);
        assert_eq!((state.sustained_w, state.temp_limit_c), (25, 85));
        let saved = store.load().hardware.tdp.unwrap();
        assert!(!saved.power_limits_enabled);
        assert_eq!(saved.sustained_w, 25);
    }

    #[tokio::test]
    async fn unsupported_hardware() {
        let service = fake(
            TdpConfig {
                is_supported: false,
                ..TdpConfig::default()
            },
            &[],
        );

        assert_eq!(load(&service, &store(None)).await, TdpState::Unsupported);
    }
}
