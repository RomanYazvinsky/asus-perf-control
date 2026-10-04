//! CPU boost mode — former `CpuBoostBloc` and `BoostModeSelector` presets.

use anyhow::{anyhow, Result};
use async_trait::async_trait;
use serde_json::Value;

use crate::logging::{log_hardware_mismatch, log_read_mismatch};
use crate::settings::{update_hardware, AppSettingsStore};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CpuBoostMode {
    Disabled,
    Efficient,
    EfficientEnabled,
    Enabled,
    EfficientAggressive,
    Aggressive,
}

impl CpuBoostMode {
    pub const ALL: [CpuBoostMode; 6] = [
        CpuBoostMode::Disabled,
        CpuBoostMode::Efficient,
        CpuBoostMode::EfficientEnabled,
        CpuBoostMode::Enabled,
        CpuBoostMode::EfficientAggressive,
        CpuBoostMode::Aggressive,
    ];

    pub fn daemon_id(self) -> &'static str {
        match self {
            CpuBoostMode::Disabled => "disabled",
            CpuBoostMode::Efficient => "efficient",
            CpuBoostMode::EfficientEnabled => "efficient_enabled",
            CpuBoostMode::Enabled => "enabled",
            CpuBoostMode::EfficientAggressive => "efficient_aggressive",
            CpuBoostMode::Aggressive => "aggressive",
        }
    }

    pub fn try_from_daemon_id(id: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|m| m.daemon_id() == id)
    }

    /// Unknown ids fall back to `Enabled`, like the daemon contract expects.
    pub fn from_daemon_id(id: &str) -> Self {
        Self::try_from_daemon_id(id).unwrap_or(CpuBoostMode::Enabled)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CpuBoostStatus {
    pub is_supported: bool,
    pub current_mode: CpuBoostMode,
    pub boost_enabled: bool,
    pub epp: Option<String>,
    pub epp_writable: bool,
    pub available_modes: Vec<CpuBoostMode>,
}

/// Parses the daemon's `GetCpuBoostStatus` JSON payload.
pub fn parse_status(payload: &str) -> Result<CpuBoostStatus> {
    let Value::Object(json) = serde_json::from_str::<Value>(payload)? else {
        return Err(anyhow!("Invalid CPU boost status payload"));
    };
    let available_modes = match json.get("availableModes") {
        Some(Value::Array(ids)) => ids
            .iter()
            .filter_map(Value::as_str)
            .map(CpuBoostMode::from_daemon_id)
            .collect(),
        _ => CpuBoostMode::ALL.to_vec(),
    };
    Ok(CpuBoostStatus {
        is_supported: json.get("isSupported") == Some(&Value::Bool(true)),
        current_mode: CpuBoostMode::from_daemon_id(
            json.get("currentMode").and_then(Value::as_str).unwrap_or("enabled"),
        ),
        boost_enabled: json.get("boostEnabled") == Some(&Value::Bool(true)),
        epp: json.get("epp").and_then(Value::as_str).map(str::to_string),
        epp_writable: json.get("eppWritable") != Some(&Value::Bool(false)),
        available_modes,
    })
}

#[async_trait]
pub trait CpuBoostService: Send + Sync {
    async fn get_cpu_boost_status(&self) -> Result<CpuBoostStatus>;
    async fn set_cpu_boost_mode(&self, mode: CpuBoostMode) -> Result<()>;
}

#[derive(Debug, Clone, PartialEq)]
pub enum CpuBoostState {
    Initial,
    Loading,
    Loaded(CpuBoostStatus),
    Unsupported,
    Error(String),
}

pub async fn load_status(service: &dyn CpuBoostService, store: &dyn AppSettingsStore) -> CpuBoostState {
    let result: Result<CpuBoostState> = async {
        let mut status = service.get_cpu_boost_status().await?;
        if !status.is_supported {
            return Ok(CpuBoostState::Unsupported);
        }
        let desired_id = store.load().hardware.cpu_boost_mode;
        match desired_id {
            None => {
                let id = status.current_mode.daemon_id().to_string();
                update_hardware(store, |hw| hw.cpu_boost_mode = Some(id.clone()))?;
            }
            Some(id) => match CpuBoostMode::try_from_daemon_id(&id) {
                None => log_read_mismatch("hardware.cpuBoostMode", &id),
                Some(desired) if desired != status.current_mode => {
                    log_hardware_mismatch("cpuBoostMode", desired.daemon_id(), status.current_mode.daemon_id());
                    if status.available_modes.contains(&desired) {
                        service.set_cpu_boost_mode(desired).await?;
                        status = service.get_cpu_boost_status().await?;
                    }
                }
                Some(_) => {}
            },
        }
        Ok(CpuBoostState::Loaded(status))
    }
    .await;
    result.unwrap_or_else(|e| CpuBoostState::Error(format!("{e:#}")))
}

pub async fn set_mode(service: &dyn CpuBoostService, store: &dyn AppSettingsStore, mode: CpuBoostMode) -> CpuBoostState {
    let result: Result<CpuBoostState> = async {
        service.set_cpu_boost_mode(mode).await?;
        update_hardware(store, |hw| hw.cpu_boost_mode = Some(mode.daemon_id().to_string()))?;
        let status = service.get_cpu_boost_status().await?;
        Ok(if status.is_supported {
            CpuBoostState::Loaded(status)
        } else {
            CpuBoostState::Unsupported
        })
    }
    .await;
    result.unwrap_or_else(|e| CpuBoostState::Error(format!("{e:#}")))
}

/// User-facing preset; resolves to the first available daemon mode.
pub struct BoostPreset {
    pub id: &'static str,
    pub label: &'static str,
    pub description: &'static str,
    pub icon: &'static str,
    pub modes: &'static [CpuBoostMode],
}

pub const BOOST_PRESETS: [BoostPreset; 4] = [
    BoostPreset {
        id: "off",
        label: "Off",
        description: "Disable CPU boost",
        icon: "⛔",
        modes: &[CpuBoostMode::Disabled],
    },
    BoostPreset {
        id: "efficient",
        label: "Efficient",
        description: "Boost with lower power bias",
        icon: "🍃",
        modes: &[CpuBoostMode::EfficientEnabled, CpuBoostMode::Efficient],
    },
    BoostPreset {
        id: "balanced",
        label: "Balanced",
        description: "Responsive boost for normal use",
        icon: "📈",
        modes: &[CpuBoostMode::EfficientAggressive, CpuBoostMode::Enabled],
    },
    BoostPreset {
        id: "aggressive",
        label: "Aggressive",
        description: "Maximum boost response",
        icon: "🚀",
        modes: &[CpuBoostMode::Aggressive],
    },
];

impl BoostPreset {
    pub fn includes(&self, mode: CpuBoostMode) -> bool {
        self.modes.contains(&mode)
    }

    pub fn resolve_mode(&self, available: &[CpuBoostMode]) -> Option<CpuBoostMode> {
        self.modes.iter().copied().find(|m| available.contains(m))
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use super::*;
    use crate::settings::{AppSettings, HardwareSettings, MemorySettingsStore};

    fn status(current: CpuBoostMode) -> CpuBoostStatus {
        CpuBoostStatus {
            is_supported: true,
            current_mode: current,
            boost_enabled: true,
            epp: None,
            epp_writable: true,
            available_modes: CpuBoostMode::ALL.to_vec(),
        }
    }

    struct Fake {
        status: Mutex<CpuBoostStatus>,
        writes: Mutex<Vec<CpuBoostMode>>,
    }

    #[async_trait]
    impl CpuBoostService for Fake {
        async fn get_cpu_boost_status(&self) -> Result<CpuBoostStatus> {
            Ok(self.status.lock().unwrap().clone())
        }
        async fn set_cpu_boost_mode(&self, mode: CpuBoostMode) -> Result<()> {
            self.status.lock().unwrap().current_mode = mode;
            self.writes.lock().unwrap().push(mode);
            Ok(())
        }
    }

    fn fake(current: CpuBoostMode) -> Fake {
        Fake {
            status: Mutex::new(status(current)),
            writes: Mutex::default(),
        }
    }

    fn store(saved: Option<&str>) -> MemorySettingsStore {
        MemorySettingsStore::new(AppSettings {
            hardware: HardwareSettings {
                cpu_boost_mode: saved.map(Into::into),
                ..Default::default()
            },
            ..Default::default()
        })
    }

    #[test]
    fn parses_daemon_payload() {
        let status = parse_status(
            r#"{"isSupported":true,"currentMode":"efficient_enabled","boostEnabled":true,
                "epp":"balance_power","eppWritable":false,"availableModes":["disabled","aggressive"]}"#,
        )
        .unwrap();

        assert!(status.is_supported);
        assert_eq!(status.current_mode, CpuBoostMode::EfficientEnabled);
        assert_eq!(status.epp.as_deref(), Some("balance_power"));
        assert!(!status.epp_writable);
        assert_eq!(status.available_modes, vec![CpuBoostMode::Disabled, CpuBoostMode::Aggressive]);
        assert!(parse_status("[]").is_err());
    }

    #[test]
    fn missing_fields_use_defaults() {
        let status = parse_status("{}").unwrap();
        assert!(!status.is_supported);
        assert_eq!(status.current_mode, CpuBoostMode::Enabled);
        assert!(status.epp_writable);
        assert_eq!(status.available_modes.len(), 6);
    }

    #[tokio::test]
    async fn restores_saved_mode() {
        let service = fake(CpuBoostMode::Enabled);
        let store = store(Some("aggressive"));

        let state = load_status(&service, &store).await;

        assert_eq!(*service.writes.lock().unwrap(), vec![CpuBoostMode::Aggressive]);
        assert!(matches!(state, CpuBoostState::Loaded(s) if s.current_mode == CpuBoostMode::Aggressive));
    }

    #[tokio::test]
    async fn saves_current_mode_when_nothing_saved() {
        let service = fake(CpuBoostMode::Efficient);
        let store = store(None);

        load_status(&service, &store).await;

        assert_eq!(store.load().hardware.cpu_boost_mode.as_deref(), Some("efficient"));
    }

    #[tokio::test]
    async fn ignores_unknown_saved_mode() {
        let service = fake(CpuBoostMode::Enabled);
        let store = store(Some("turbo-max"));

        let state = load_status(&service, &store).await;

        assert!(service.writes.lock().unwrap().is_empty());
        assert!(matches!(state, CpuBoostState::Loaded(_)));
    }

    #[tokio::test]
    async fn unsupported_status() {
        let service = fake(CpuBoostMode::Enabled);
        service.status.lock().unwrap().is_supported = false;

        assert_eq!(load_status(&service, &store(None)).await, CpuBoostState::Unsupported);
    }

    #[test]
    fn presets_resolve_to_first_available_mode() {
        let balanced = &BOOST_PRESETS[2];
        assert_eq!(
            balanced.resolve_mode(&[CpuBoostMode::Enabled, CpuBoostMode::Disabled]),
            Some(CpuBoostMode::Enabled)
        );
        assert_eq!(
            balanced.resolve_mode(&CpuBoostMode::ALL),
            Some(CpuBoostMode::EfficientAggressive)
        );
        assert_eq!(BOOST_PRESETS[3].resolve_mode(&[CpuBoostMode::Disabled]), None);
        assert!(BOOST_PRESETS[1].includes(CpuBoostMode::Efficient));
    }
}
