//! Production implementations of the feature service traits on top of the D-Bus
//! backend and `rust_core` sysfs readers.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use anyhow::{anyhow, Context, Result};
use async_trait::async_trait;

use super::battery::BatteryService;
use super::cpu_boost::{self, CpuBoostMode, CpuBoostService, CpuBoostStatus};
use super::fan::{FanCurveData, FanCurvePoint, FanEngineConfig, FanId, FanInfo, FanService, FanTemperatures, PerfProfile};
use super::gpu::{self, GpuConfiguration, GpuMode, GpuService, GpuStatus};
use super::performance::PerformanceService;
use super::tdp::{TdpConfig, TdpService};
use super::updates::{PackageUpdateBackend, PackageUpdateService, PackageUpdateStatus};
use super::xg_diag::{custom_curve, XgFanCurvePort, XgPlatformProfilePort};
use crate::backend::Backend;
use crate::logging::trace;

pub struct Services {
    backend: Backend,
    fan_engine_started: AtomicBool,
    fan_engine_known_running: AtomicBool,
}

impl Services {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            backend: Backend::new(),
            fan_engine_started: AtomicBool::new(false),
            fan_engine_known_running: AtomicBool::new(false),
        })
    }

    pub fn backend(&self) -> &Backend {
        &self.backend
    }
}

#[async_trait]
impl PerformanceService for Services {
    async fn get_performance_mode(&self) -> Result<i32> {
        self.backend
            .asusd
            .get_performance_mode()
            .await
            .map_err(|e| anyhow!("Failed to get performance mode from asusd: {e:#}. Is asusd running?"))
    }

    async fn set_performance_mode(&self, mode: i32) -> Result<()> {
        self.backend
            .asusd
            .set_performance_mode(mode)
            .await
            .map_err(|e| anyhow!("Failed to set performance mode via asusd: {e:#}"))
    }
}

#[async_trait]
impl CpuBoostService for Services {
    async fn get_cpu_boost_status(&self) -> Result<CpuBoostStatus> {
        let payload = self
            .backend
            .daemon
            .get_cpu_boost_status()
            .await
            .map_err(|e| anyhow!("Failed to get CPU boost status via daemon: {e:#}"))?;
        cpu_boost::parse_status(&payload).map_err(|e| anyhow!("Failed to get CPU boost status via daemon: {e:#}"))
    }

    async fn set_cpu_boost_mode(&self, mode: CpuBoostMode) -> Result<()> {
        self.backend
            .daemon
            .set_cpu_boost_mode(mode.daemon_id())
            .await
            .map_err(|e| anyhow!("Failed to set CPU boost mode via daemon: {e:#}"))
    }
}

#[async_trait]
impl TdpService for Services {
    /// asusd limits + saved temperature limit from our daemon + active profile.
    async fn get_tdp_config(&self) -> Result<TdpConfig> {
        let asusd = self
            .backend
            .asusd
            .get_tdp_config()
            .await
            .map_err(|e| anyhow!("Failed to get TDP config from asusd: {e:#}. Is asusd running?"))?;
        // Non-critical: daemon down means "no saved limit", unknown profile means Balanced.
        let temp_limit_c = self.backend.daemon.get_temp_limit().await.unwrap_or(0);
        let active_performance_mode = self.backend.asusd.get_performance_mode().await.unwrap_or(1);
        Ok(TdpConfig {
            is_supported: true,
            sustained_w: asusd.sustained_w,
            boost_w: asusd.boost_w,
            temp_limit_c,
            active_performance_mode,
            min_sustained_w: asusd.min_sustained_w,
            max_sustained_w: asusd.max_sustained_w,
            min_boost_w: asusd.min_boost_w,
            max_boost_w: asusd.max_boost_w,
            power_limits_enabled: asusd.power_limits_enabled,
        })
    }

    async fn set_performance_mode(&self, mode: i32) -> Result<()> {
        PerformanceService::set_performance_mode(self, mode).await
    }

    async fn set_power_limits(&self, sustained_w: i32, boost_w: i32) -> Result<()> {
        async {
            self.backend.asusd.set_tdp_config(sustained_w, boost_w).await?;
            self.backend.asusd.set_power_limits_enabled(true).await
        }
        .await
        .map_err(|e: anyhow::Error| anyhow!("Failed to set TDP via asusd: {e:#}"))
    }

    async fn clear_power_limits(&self) -> Result<()> {
        async {
            self.backend.asusd.reset_power_limits_to_profile_defaults().await?;
            self.backend.asusd.set_power_limits_enabled(false).await
        }
        .await
        .map_err(|e: anyhow::Error| anyhow!("Failed to clear TDP power limits: {e:#}"))
    }

    async fn set_cpu_temp_limit(&self, temp_limit_c: i32) -> Result<()> {
        self.backend
            .daemon
            .set_temp_limit(temp_limit_c)
            .await
            .map(|_| ())
            .map_err(|e| anyhow!("Failed to set temperature limit via daemon: {e:#}"))
    }
}

#[async_trait]
impl BatteryService for Services {
    async fn get_charge_limit(&self) -> Result<i32> {
        self.backend
            .asusd
            .get_charge_limit()
            .await
            .map_err(|e| anyhow!("Failed to get battery charge limit: {e:#}"))
    }

    async fn set_charge_limit(&self, limit: i32) -> Result<()> {
        self.backend
            .asusd
            .set_charge_limit(limit)
            .await
            .map_err(|e| anyhow!("Failed to set battery charge limit: {e:#}"))
    }

    async fn is_charge_limit_supported(&self) -> bool {
        self.backend.asusd.is_charge_limit_supported().await
    }
}

#[async_trait]
impl GpuService for Services {
    async fn get_capabilities(&self) -> Result<GpuConfiguration> {
        let supported = self.backend.supergfx.supported().await?;
        let vendor = self.backend.supergfx.vendor().await;
        Ok(GpuConfiguration::from_supergfx(&supported, &vendor))
    }

    async fn get_status(&self) -> Result<GpuStatus> {
        Ok(GpuStatus::from_supergfx(self.backend.supergfx.mode().await?))
    }

    async fn get_supported_modes(&self) -> Result<Vec<GpuMode>> {
        Ok(self
            .backend
            .supergfx
            .supported()
            .await?
            .into_iter()
            .map(GpuMode::from_supergfx)
            .collect())
    }

    async fn set_gpu_mode(&self, mode: GpuMode) -> Result<()> {
        let action = self
            .backend
            .supergfx
            .set_mode(mode.to_supergfx())
            .await
            .map_err(|e| anyhow!("Failed to set GPU mode: {e:#}"))?;
        match gpu::user_action_error(action) {
            None => Ok(()),
            Some(e) => Err(anyhow!("Failed to set GPU mode: {e}")),
        }
    }
}

#[async_trait]
impl PackageUpdateService for Services {
    async fn get_package_backend(&self) -> Result<PackageUpdateBackend> {
        PackageUpdateBackend::from_wire(&self.backend.daemon.get_package_backend().await?)
    }

    async fn check_for_updates(&self) -> Result<PackageUpdateStatus> {
        PackageUpdateStatus::from_json_str(&self.backend.daemon.check_for_updates().await?)
    }

    async fn start_update(&self) -> Result<String> {
        self.backend.daemon.start_update().await
    }

    async fn get_update_status(&self, update_id: &str) -> Result<PackageUpdateStatus> {
        PackageUpdateStatus::from_json_str(&self.backend.daemon.get_update_status(update_id).await?)
    }

    async fn cancel_update(&self, update_id: &str) -> Result<bool> {
        self.backend.daemon.cancel_update(update_id).await
    }
}

fn read_fan_rpms() -> Vec<(FanId, Option<u32>)> {
    asus_linux_control_core::get_fan_rpms()
        .into_iter()
        .filter_map(|r| FanId::from_name(&r.id).map(|id| (id, r.rpm)))
        .collect()
}

#[async_trait]
impl FanService for Services {
    async fn capabilities(&self) -> Result<Vec<FanInfo>> {
        trace("FanServiceImpl.getCapabilities", &[], async {
            Ok(asus_linux_control_core::get_fan_capabilities()
                .fans
                .into_iter()
                .filter_map(|f| FanId::from_name(&f.id).map(|id| FanInfo { id, label: f.label }))
                .collect())
        })
        .await
    }

    async fn fan_rpms(&self) -> Result<Vec<(FanId, Option<u32>)>> {
        Ok(read_fan_rpms())
    }

    async fn temperatures(&self) -> Result<FanTemperatures> {
        let readings = asus_linux_control_core::get_sensor_readings().map_err(|e| anyhow!(e))?;
        Ok(FanTemperatures {
            cpu_temp_c: readings.cpu_temp.map_or(0, |t| t as i32),
            gpu_temp_c: readings.gpu_temp.map_or(0, |t| t as i32),
        })
    }

    async fn set_active_profile(&self, profile: PerfProfile) -> Result<()> {
        self.backend.asusd.set_active_profile(profile).await
    }

    async fn asusd_curves(&self, profile: PerfProfile) -> Result<Vec<FanCurveData>> {
        self.backend.asusd.get_fan_curves(profile).await
    }

    async fn set_asusd_curve(&self, profile: PerfProfile, fan: FanId, points: &[FanCurvePoint]) -> Result<()> {
        if points.len() != 8 {
            return Err(anyhow!("FanCurveData requires exactly 8 points"));
        }
        let data = FanCurveData::from_curve_points(fan.asusd_name(), points, true);
        self.backend.asusd.apply_fan_curve(profile, &data).await
    }

    async fn set_constant_speed(&self, profile: PerfProfile, fan: FanId, speed_percent: i64) -> Result<()> {
        let data = FanCurveData::flat(fan.asusd_name(), speed_percent);
        self.backend.asusd.apply_fan_curve(profile, &data).await
    }

    async fn reset_to_defaults(&self, profile: PerfProfile) -> Result<()> {
        self.backend.asusd.reset_curves_to_defaults(profile).await
    }

    async fn start_dynamic_engine(&self, config: &FanEngineConfig) -> Result<()> {
        let json = serde_json::Value::Object(config.to_json()).to_string();
        self.backend.daemon.start_fan_engine(&json).await?;
        self.fan_engine_started.store(true, Ordering::SeqCst);
        self.fan_engine_known_running.store(true, Ordering::SeqCst);
        Ok(())
    }

    async fn stop_dynamic_engine(&self) -> Result<()> {
        if !self.fan_engine_started.load(Ordering::SeqCst) && !self.fan_engine_known_running.load(Ordering::SeqCst) {
            return Ok(());
        }
        self.backend.daemon.stop_fan_engine().await?;
        self.fan_engine_started.store(false, Ordering::SeqCst);
        self.fan_engine_known_running.store(false, Ordering::SeqCst);
        Ok(())
    }

    async fn update_dynamic_curve(&self, fan: FanId, curve: &[FanCurvePoint]) -> Result<()> {
        let json = super::fan::curve_to_json_pairs(curve).to_string();
        self.backend.daemon.update_fan_engine_curve(fan.name(), &json).await
    }

    async fn is_dynamic_engine_running(&self) -> Result<bool> {
        let running = self.backend.daemon.get_fan_engine_running().await?;
        self.fan_engine_known_running.store(running, Ordering::SeqCst);
        Ok(running)
    }

    async fn run_max_speed_test(&self, profile: PerfProfile, fan: FanId) -> Result<u32> {
        let asusd = &self.backend.asusd;
        let name = fan.asusd_name();
        let current = asusd
            .get_fan_curves(profile)
            .await?
            .into_iter()
            .find(|c| c.name == name)
            .with_context(|| format!("No current curve found for {name}"))?;

        asusd.set_fan_curve(profile, &FanCurveData::flat(name, 100)).await?;
        let measured: Result<u32> = async {
            asusd.set_fan_curve_enabled(profile, name, true).await?;
            tokio::time::sleep(Duration::from_secs(2)).await;
            Ok(read_fan_rpms()
                .into_iter()
                .find(|(id, _)| *id == fan)
                .and_then(|(_, rpm)| rpm)
                .unwrap_or(0))
        }
        .await;
        let restored: Result<()> = async {
            asusd.set_fan_curve(profile, &current).await?;
            asusd.set_fan_curve_enabled(profile, name, current.enabled).await
        }
        .await;
        let rpm = measured?;
        restored?;
        Ok(rpm)
    }
}

/// asusd-backed ports for the XG Mobile diagnostic.
pub struct AsusdXgPorts(pub Arc<Services>);

#[async_trait]
impl XgFanCurvePort for AsusdXgPorts {
    async fn get_curves(&self, profile: PerfProfile) -> Result<Vec<FanCurveData>> {
        self.0.backend.asusd.get_fan_curves(profile).await
    }

    async fn set_custom_curve(&self, profile: PerfProfile, fan: &str, pwm: &[i64], temps: Option<&[i64]>) -> Result<()> {
        self.0
            .backend
            .asusd
            .apply_fan_curve(profile, &custom_curve(fan, pwm, temps))
            .await
    }

    async fn restore_defaults(&self, profile: PerfProfile) -> Result<()> {
        self.0.backend.asusd.reset_curves_to_defaults(profile).await
    }
}

#[async_trait]
impl XgPlatformProfilePort for AsusdXgPorts {
    async fn get_active_profile(&self) -> Result<PerfProfile> {
        let id = self.0.backend.asusd.get_platform_profile().await?;
        Ok(PerfProfile::from_asusd_id(id).unwrap_or(PerfProfile::Balanced))
    }

    async fn set_active_profile(&self, profile: PerfProfile) -> Result<()> {
        self.0.backend.asusd.set_active_profile(profile).await
    }
}
