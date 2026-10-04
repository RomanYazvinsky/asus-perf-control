//! Performance mode (Silent / Balanced / Turbo) — former `PerformanceBloc`.

use anyhow::Result;
use async_trait::async_trait;

use crate::logging::log_hardware_mismatch;
use crate::settings::{update_hardware, AppSettingsStore};

#[async_trait]
pub trait PerformanceService: Send + Sync {
    async fn get_performance_mode(&self) -> Result<i32>;
    async fn set_performance_mode(&self, mode: i32) -> Result<()>;
}

#[derive(Debug, Clone, PartialEq)]
pub enum PerformanceState {
    Initial,
    Loading,
    Loaded(i32),
    Error(String),
}

pub fn mode_name(mode: i32) -> &'static str {
    match mode {
        0 => "Silent",
        1 => "Balanced",
        2 => "Turbo",
        _ => "Unknown",
    }
}

/// Reads the hardware mode and re-applies the saved mode when they differ.
/// With no saved mode, the hardware mode becomes the saved one.
pub async fn load_current_mode(service: &dyn PerformanceService, store: &dyn AppSettingsStore) -> PerformanceState {
    let result: Result<i32> = async {
        let mut current = service.get_performance_mode().await?;
        match store.load().hardware.performance_mode {
            None => {
                update_hardware(store, |hw| hw.performance_mode = Some(i64::from(current)))?;
            }
            Some(desired) if desired != i64::from(current) => {
                log_hardware_mismatch("performanceMode", desired, current);
                service.set_performance_mode(desired as i32).await?;
                current = desired as i32;
            }
            Some(_) => {}
        }
        Ok(current)
    }
    .await;
    match result {
        Ok(mode) => PerformanceState::Loaded(mode),
        Err(e) => PerformanceState::Error(format!("{e:#}")),
    }
}

pub async fn set_mode(service: &dyn PerformanceService, store: &dyn AppSettingsStore, mode: i32) -> PerformanceState {
    let result: Result<()> = async {
        service.set_performance_mode(mode).await?;
        update_hardware(store, |hw| hw.performance_mode = Some(i64::from(mode)))?;
        Ok(())
    }
    .await;
    match result {
        Ok(()) => PerformanceState::Loaded(mode),
        Err(e) => PerformanceState::Error(format!("{e:#}")),
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use super::*;
    use crate::settings::{AppSettings, HardwareSettings, MemorySettingsStore};

    struct FakeService {
        mode: Mutex<i32>,
        writes: Mutex<Vec<i32>>,
        fail: bool,
    }

    impl FakeService {
        fn new(mode: i32) -> Self {
            Self {
                mode: Mutex::new(mode),
                writes: Mutex::default(),
                fail: false,
            }
        }
    }

    #[async_trait]
    impl PerformanceService for FakeService {
        async fn get_performance_mode(&self) -> Result<i32> {
            if self.fail {
                anyhow::bail!("asusd unavailable");
            }
            Ok(*self.mode.lock().unwrap())
        }
        async fn set_performance_mode(&self, mode: i32) -> Result<()> {
            if self.fail {
                anyhow::bail!("asusd unavailable");
            }
            *self.mode.lock().unwrap() = mode;
            self.writes.lock().unwrap().push(mode);
            Ok(())
        }
    }

    fn store_with_mode(mode: Option<i64>) -> MemorySettingsStore {
        MemorySettingsStore::new(AppSettings {
            hardware: HardwareSettings {
                performance_mode: mode,
                ..Default::default()
            },
            ..Default::default()
        })
    }

    #[tokio::test]
    async fn saves_hardware_mode_when_nothing_is_saved() {
        let service = FakeService::new(2);
        let store = store_with_mode(None);

        let state = load_current_mode(&service, &store).await;

        assert_eq!(state, PerformanceState::Loaded(2));
        assert_eq!(store.load().hardware.performance_mode, Some(2));
        assert!(service.writes.lock().unwrap().is_empty());
    }

    #[tokio::test]
    async fn restores_saved_mode_when_hardware_differs() {
        let service = FakeService::new(1);
        let store = store_with_mode(Some(0));

        let state = load_current_mode(&service, &store).await;

        assert_eq!(state, PerformanceState::Loaded(0));
        assert_eq!(*service.writes.lock().unwrap(), vec![0]);
    }

    #[tokio::test]
    async fn set_mode_persists_choice() {
        let service = FakeService::new(1);
        let store = store_with_mode(Some(1));

        let state = set_mode(&service, &store, 2).await;

        assert_eq!(state, PerformanceState::Loaded(2));
        assert_eq!(store.load().hardware.performance_mode, Some(2));
    }

    #[tokio::test]
    async fn reports_errors() {
        let service = FakeService {
            fail: true,
            ..FakeService::new(1)
        };
        let store = store_with_mode(None);

        let state = load_current_mode(&service, &store).await;

        assert!(matches!(state, PerformanceState::Error(msg) if msg.contains("asusd unavailable")));
    }

    #[test]
    fn names_modes() {
        assert_eq!(mode_name(0), "Silent");
        assert_eq!(mode_name(1), "Balanced");
        assert_eq!(mode_name(2), "Turbo");
        assert_eq!(mode_name(9), "Unknown");
    }
}
