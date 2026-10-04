//! Battery charge limit — former `BatteryBloc`.

use anyhow::Result;
use async_trait::async_trait;

use crate::logging::log_hardware_mismatch;
use crate::settings::{update_hardware, AppSettingsStore};

#[async_trait]
pub trait BatteryService: Send + Sync {
    /// 0–100, 0 = no limit.
    async fn get_charge_limit(&self) -> Result<i32>;
    async fn set_charge_limit(&self, limit: i32) -> Result<()>;
    async fn is_charge_limit_supported(&self) -> bool;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BatteryState {
    Initial,
    Loading,
    Loaded { charge_limit: i32 },
    Unsupported,
    Error(String),
}

/// Checks support, then loads the limit (restoring the saved one if it differs).
pub async fn check_support_and_load(service: &dyn BatteryService, store: &dyn AppSettingsStore) -> BatteryState {
    if !service.is_charge_limit_supported().await {
        return BatteryState::Unsupported;
    }
    load(service, store).await
}

pub async fn load(service: &dyn BatteryService, store: &dyn AppSettingsStore) -> BatteryState {
    let result: Result<i32> = async {
        let current = service.get_charge_limit().await?;
        match store.load().hardware.battery_charge_limit {
            None => {
                update_hardware(store, |hw| hw.battery_charge_limit = Some(i64::from(current)))?;
                Ok(current)
            }
            Some(desired) if desired == i64::from(current) => Ok(current),
            Some(desired) => {
                log_hardware_mismatch("batteryChargeLimit", desired, current);
                service.set_charge_limit(desired as i32).await?;
                Ok(desired as i32)
            }
        }
    }
    .await;
    match result {
        Ok(charge_limit) => BatteryState::Loaded { charge_limit },
        Err(e) => BatteryState::Error(format!("{e:#}")),
    }
}

pub async fn set_limit(service: &dyn BatteryService, store: &dyn AppSettingsStore, limit: i32) -> BatteryState {
    let result: Result<()> = async {
        service.set_charge_limit(limit).await?;
        update_hardware(store, |hw| hw.battery_charge_limit = Some(i64::from(limit)))?;
        Ok(())
    }
    .await;
    match result {
        Ok(()) => BatteryState::Loaded { charge_limit: limit },
        Err(e) => BatteryState::Error(format!("{e:#}")),
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use super::*;
    use crate::settings::{AppSettings, HardwareSettings, MemorySettingsStore};

    struct Fake {
        limit: Mutex<i32>,
        supported: bool,
        writes: Mutex<Vec<i32>>,
    }

    #[async_trait]
    impl BatteryService for Fake {
        async fn get_charge_limit(&self) -> Result<i32> {
            Ok(*self.limit.lock().unwrap())
        }
        async fn set_charge_limit(&self, limit: i32) -> Result<()> {
            *self.limit.lock().unwrap() = limit;
            self.writes.lock().unwrap().push(limit);
            Ok(())
        }
        async fn is_charge_limit_supported(&self) -> bool {
            self.supported
        }
    }

    fn fake(limit: i32, supported: bool) -> Fake {
        Fake {
            limit: Mutex::new(limit),
            supported,
            writes: Mutex::default(),
        }
    }

    fn store(saved: Option<i64>) -> MemorySettingsStore {
        MemorySettingsStore::new(AppSettings {
            hardware: HardwareSettings {
                battery_charge_limit: saved,
                ..Default::default()
            },
            ..Default::default()
        })
    }

    #[tokio::test]
    async fn unsupported_hardware() {
        assert_eq!(
            check_support_and_load(&fake(80, false), &store(None)).await,
            BatteryState::Unsupported
        );
    }

    #[tokio::test]
    async fn restores_saved_limit() {
        let service = fake(100, true);

        let state = check_support_and_load(&service, &store(Some(80))).await;

        assert_eq!(state, BatteryState::Loaded { charge_limit: 80 });
        assert_eq!(*service.writes.lock().unwrap(), vec![80]);
    }

    #[tokio::test]
    async fn saves_hardware_limit_when_nothing_saved() {
        let service = fake(90, true);
        let store = store(None);

        check_support_and_load(&service, &store).await;

        assert_eq!(store.load().hardware.battery_charge_limit, Some(90));
        assert!(service.writes.lock().unwrap().is_empty());
    }

    #[tokio::test]
    async fn set_limit_persists() {
        let service = fake(100, true);
        let store = store(Some(100));

        assert_eq!(set_limit(&service, &store, 60).await, BatteryState::Loaded { charge_limit: 60 });
        assert_eq!(store.load().hardware.battery_charge_limit, Some(60));
    }
}
