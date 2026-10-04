//! CPU power limits and temperature ceiling — former `TdpBloc`.

use anyhow::Result;
use async_trait::async_trait;

use crate::logging::log_hardware_mismatch;
use crate::settings::{update_hardware, AppSettingsStore, TdpDesiredSettings};

/// asusd TDP values merged with the daemon temperature limit and active profile.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TdpConfig {
    pub is_supported: bool,
    pub sustained_w: i32,
    pub boost_w: i32,
    pub temp_limit_c: i32,
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
            temp_limit_c: 0,
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
    async fn set_performance_mode(&self, mode: i32) -> Result<()>;
    async fn set_power_limits(&self, sustained_w: i32, boost_w: i32) -> Result<()>;
    async fn clear_power_limits(&self) -> Result<()>;
    async fn set_cpu_temp_limit(&self, temp_limit_c: i32) -> Result<()>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TdpLoaded {
    pub sustained_w: i32,
    pub boost_w: i32,
    pub temp_limit_c: i32,
    pub active_performance_mode: i32,
    pub min_w: i32,
    pub max_w: i32,
    pub min_boost_w: i32,
    pub max_boost_w: i32,
    pub power_limits_enabled: bool,
    pub temp_limit_enabled: bool,
}

impl TdpLoaded {
    pub fn from_config(config: &TdpConfig) -> Self {
        Self {
            sustained_w: config.sustained_w,
            boost_w: config.boost_w,
            temp_limit_c: config.temp_limit_c,
            active_performance_mode: config.active_performance_mode,
            min_w: config.min_sustained_w,
            max_w: config.max_sustained_w,
            min_boost_w: config.min_boost_w,
            max_boost_w: config.max_boost_w,
            power_limits_enabled: config.power_limits_enabled,
            temp_limit_enabled: config.temp_limit_c > 0,
        }
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

fn loaded_or_unsupported(config: &TdpConfig) -> TdpState {
    if config.is_supported {
        TdpState::Loaded(TdpLoaded::from_config(config))
    } else {
        TdpState::Unsupported
    }
}

fn error_state(e: anyhow::Error) -> TdpState {
    TdpState::Error(format!("{e:#}"))
}

pub async fn load(service: &dyn TdpService, store: &dyn AppSettingsStore) -> TdpState {
    let result: Result<TdpConfig> = async {
        let config = service.get_tdp_config().await?;
        restore_saved_tdp_if_needed(service, store, config).await
    }
    .await;
    result.map(|c| loaded_or_unsupported(&c)).unwrap_or_else(error_state)
}

/// Switches the active performance profile, then reloads the limits.
pub async fn select_profile(service: &dyn TdpService, store: &dyn AppSettingsStore, mode: i32) -> TdpState {
    let result: Result<TdpConfig> = async {
        service.set_performance_mode(mode).await?;
        update_hardware(store, |hw| hw.performance_mode = Some(i64::from(mode)))?;
        service.get_tdp_config().await
    }
    .await;
    result.map(|c| loaded_or_unsupported(&c)).unwrap_or_else(error_state)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SetTdpRequest {
    pub apply_power_limits: bool,
    pub apply_temp_limit: bool,
    pub sustained_w: i32,
    pub boost_w: i32,
    pub temp_limit_c: i32,
}

pub async fn set_tdp(
    service: &dyn TdpService,
    store: &dyn AppSettingsStore,
    request: SetTdpRequest,
    previous: Option<TdpLoaded>,
) -> TdpState {
    let result: Result<()> = async {
        if request.apply_power_limits {
            service.set_power_limits(request.sustained_w, request.boost_w).await?;
        }
        if request.apply_temp_limit {
            service.set_cpu_temp_limit(request.temp_limit_c).await?;
        }
        update_hardware(store, |hw| {
            hw.tdp = Some(TdpDesiredSettings {
                power_limits_enabled: request.apply_power_limits,
                sustained_w: i64::from(request.sustained_w),
                boost_w: i64::from(request.boost_w),
                temp_limit_enabled: request.apply_temp_limit,
                temp_limit_c: if request.apply_temp_limit {
                    i64::from(request.temp_limit_c)
                } else {
                    0
                },
            })
        })?;
        Ok(())
    }
    .await;
    if let Err(e) = result {
        return error_state(e);
    }
    let prev = previous.as_ref();
    TdpState::Loaded(TdpLoaded {
        sustained_w: if request.apply_power_limits {
            request.sustained_w
        } else {
            prev.map_or(request.sustained_w, |p| p.sustained_w)
        },
        boost_w: if request.apply_power_limits {
            request.boost_w
        } else {
            prev.map_or(request.boost_w, |p| p.boost_w)
        },
        temp_limit_c: if request.apply_temp_limit {
            request.temp_limit_c
        } else {
            prev.map_or(request.temp_limit_c, |p| p.temp_limit_c)
        },
        active_performance_mode: prev.map_or(1, |p| p.active_performance_mode),
        min_w: prev.map_or(5, |p| p.min_w),
        max_w: prev.map_or(54, |p| p.max_w),
        min_boost_w: prev.map_or(5, |p| p.min_boost_w),
        max_boost_w: prev.map_or(54, |p| p.max_boost_w),
        power_limits_enabled: request.apply_power_limits,
        temp_limit_enabled: request.apply_temp_limit,
    })
}

/// "Reload from Hardware": discards UI edits.
pub async fn reload(service: &dyn TdpService) -> TdpState {
    match service.get_tdp_config().await {
        Ok(config) => TdpState::Loaded(TdpLoaded::from_config(&config)),
        Err(e) => error_state(e),
    }
}

pub async fn clear_power_limits(
    service: &dyn TdpService,
    store: &dyn AppSettingsStore,
    previous: Option<TdpLoaded>,
) -> TdpState {
    let prev = previous.as_ref();
    let result: Result<()> = async {
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
                temp_limit_enabled: prev
                    .map(|p| p.temp_limit_enabled)
                    .or(saved.as_ref().map(|t| t.temp_limit_enabled))
                    .unwrap_or(false),
                temp_limit_c: prev
                    .map(|p| i64::from(p.temp_limit_c))
                    .or(saved.as_ref().map(|t| t.temp_limit_c))
                    .unwrap_or(0),
            });
        })?;
        Ok(())
    }
    .await;
    if let Err(e) = result {
        return error_state(e);
    }
    let temp_limit_c = prev.map_or(0, |p| p.temp_limit_c);
    TdpState::Loaded(TdpLoaded {
        sustained_w: prev.map_or(0, |p| p.sustained_w),
        boost_w: prev.map_or(0, |p| p.boost_w),
        temp_limit_c,
        active_performance_mode: prev.map_or(1, |p| p.active_performance_mode),
        min_w: prev.map_or(5, |p| p.min_w),
        max_w: prev.map_or(54, |p| p.max_w),
        min_boost_w: prev.map_or(5, |p| p.min_boost_w),
        max_boost_w: prev.map_or(54, |p| p.max_boost_w),
        power_limits_enabled: false,
        temp_limit_enabled: prev.map_or(temp_limit_c > 0, |p| p.temp_limit_enabled),
    })
}

async fn restore_saved_tdp_if_needed(
    service: &dyn TdpService,
    store: &dyn AppSettingsStore,
    config: TdpConfig,
) -> Result<TdpConfig> {
    if !config.is_supported {
        return Ok(config);
    }
    let Some(desired) = store.load().hardware.tdp else {
        update_hardware(store, |hw| {
            hw.tdp = Some(TdpDesiredSettings {
                power_limits_enabled: config.power_limits_enabled,
                sustained_w: i64::from(config.sustained_w),
                boost_w: i64::from(config.boost_w),
                temp_limit_enabled: config.temp_limit_c > 0,
                temp_limit_c: i64::from(config.temp_limit_c),
            })
        })?;
        return Ok(config);
    };

    let mut changed = false;
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
        changed = true;
    }

    let desired_temp = if desired.temp_limit_enabled {
        desired.temp_limit_c as i32
    } else {
        0
    };
    if desired_temp != config.temp_limit_c {
        log_hardware_mismatch("tdp.tempLimitC", desired_temp, config.temp_limit_c);
        service.set_cpu_temp_limit(desired_temp).await?;
        changed = true;
    }

    if changed {
        service.get_tdp_config().await
    } else {
        Ok(config)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use super::*;
    use crate::settings::{AppSettings, HardwareSettings, MemorySettingsStore};

    #[derive(Default)]
    struct Fake {
        config: Mutex<TdpConfig>,
        calls: Mutex<Vec<String>>,
    }

    #[async_trait]
    impl TdpService for Fake {
        async fn get_tdp_config(&self) -> Result<TdpConfig> {
            Ok(self.config.lock().unwrap().clone())
        }
        async fn set_performance_mode(&self, mode: i32) -> Result<()> {
            self.calls.lock().unwrap().push(format!("mode {mode}"));
            self.config.lock().unwrap().active_performance_mode = mode;
            Ok(())
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
        async fn set_cpu_temp_limit(&self, temp_limit_c: i32) -> Result<()> {
            self.calls.lock().unwrap().push(format!("temp {temp_limit_c}"));
            self.config.lock().unwrap().temp_limit_c = temp_limit_c;
            Ok(())
        }
    }

    fn fake(config: TdpConfig) -> Fake {
        Fake {
            config: Mutex::new(config),
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

    fn config(sustained: i32, boost: i32, enabled: bool, temp: i32) -> TdpConfig {
        TdpConfig {
            sustained_w: sustained,
            boost_w: boost,
            power_limits_enabled: enabled,
            temp_limit_c: temp,
            ..TdpConfig::default()
        }
    }

    #[tokio::test]
    async fn first_load_saves_hardware_values() {
        let service = fake(config(25, 35, true, 85));
        let store = store(None);

        let state = load(&service, &store).await;

        assert!(matches!(&state, TdpState::Loaded(l) if l.temp_limit_enabled && l.sustained_w == 25));
        let saved = store.load().hardware.tdp.unwrap();
        assert!(saved.power_limits_enabled);
        assert_eq!((saved.sustained_w, saved.boost_w, saved.temp_limit_c), (25, 35, 85));
        assert!(service.calls.lock().unwrap().is_empty());
    }

    #[tokio::test]
    async fn restores_saved_power_and_temp_limits() {
        let service = fake(config(15, 20, false, 0));
        let store = store(Some(TdpDesiredSettings {
            power_limits_enabled: true,
            sustained_w: 30,
            boost_w: 40,
            temp_limit_enabled: true,
            temp_limit_c: 80,
        }));

        let state = load(&service, &store).await;

        assert_eq!(*service.calls.lock().unwrap(), vec!["power 30/40", "temp 80"]);
        assert!(matches!(state, TdpState::Loaded(l) if l.sustained_w == 30 && l.temp_limit_c == 80));
    }

    #[tokio::test]
    async fn restores_disabled_power_limits_by_clearing() {
        let service = fake(config(15, 20, true, 0));
        let store = store(Some(TdpDesiredSettings {
            power_limits_enabled: false,
            sustained_w: 0,
            boost_w: 0,
            temp_limit_enabled: false,
            temp_limit_c: 0,
        }));

        load(&service, &store).await;

        assert_eq!(*service.calls.lock().unwrap(), vec!["clear"]);
    }

    #[tokio::test]
    async fn set_tdp_applies_selected_limits_and_keeps_previous_others() {
        let service = fake(config(15, 20, false, 0));
        let store = store(None);
        let previous = TdpLoaded::from_config(&config(15, 20, false, 90));

        let state = set_tdp(
            &service,
            &store,
            SetTdpRequest {
                apply_power_limits: true,
                apply_temp_limit: false,
                sustained_w: 28,
                boost_w: 33,
                temp_limit_c: 70,
            },
            Some(previous),
        )
        .await;

        assert_eq!(*service.calls.lock().unwrap(), vec!["power 28/33"]);
        let TdpState::Loaded(loaded) = state else { panic!() };
        assert_eq!((loaded.sustained_w, loaded.boost_w, loaded.temp_limit_c), (28, 33, 90));
        assert!(loaded.power_limits_enabled);
        assert!(!loaded.temp_limit_enabled);
        let saved = store.load().hardware.tdp.unwrap();
        assert_eq!(saved.temp_limit_c, 0);
        assert!(!saved.temp_limit_enabled);
    }

    #[tokio::test]
    async fn clear_power_limits_keeps_previous_values_in_settings() {
        let service = fake(config(25, 35, true, 85));
        let store = store(None);
        let previous = TdpLoaded::from_config(&config(25, 35, true, 85));

        let state = clear_power_limits(&service, &store, Some(previous)).await;

        assert!(matches!(state, TdpState::Loaded(l) if !l.power_limits_enabled && l.sustained_w == 25));
        let saved = store.load().hardware.tdp.unwrap();
        assert!(!saved.power_limits_enabled);
        assert_eq!(saved.sustained_w, 25);
        assert!(saved.temp_limit_enabled);
    }

    #[tokio::test]
    async fn select_profile_switches_mode_and_reloads() {
        let service = fake(config(25, 35, true, 0));
        let store = store(None);

        let state = select_profile(&service, &store, 2).await;

        assert!(matches!(state, TdpState::Loaded(l) if l.active_performance_mode == 2));
        assert_eq!(store.load().hardware.performance_mode, Some(2));
    }

    #[tokio::test]
    async fn unsupported_hardware() {
        let service = fake(TdpConfig {
            is_supported: false,
            ..TdpConfig::default()
        });

        assert_eq!(load(&service, &store(None)).await, TdpState::Unsupported);
    }
}
