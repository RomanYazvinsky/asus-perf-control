//! Client for our privileged daemon (`com.asuslinuxcontrol.Daemon`).
//! Write methods are authorised by the daemon through polkit.

use std::sync::Arc;

use anyhow::{anyhow, bail, Result};

use super::bus::SystemBus;
use crate::logging::trace;

pub const BUS_NAME: &str = "com.asuslinuxcontrol.Daemon";
pub const OBJECT_PATH: &str = "/com/asuslinuxcontrol/Daemon";
pub const INTERFACE: &str = "com.asuslinuxcontrol.Daemon";

pub struct DaemonClient {
    bus: Arc<SystemBus>,
}

impl DaemonClient {
    pub fn new(bus: Arc<SystemBus>) -> Self {
        Self { bus }
    }

    async fn call<B, R>(&self, method: &str, body: &B) -> Result<R>
    where
        B: serde::Serialize + zbus::zvariant::DynamicType,
        R: serde::de::DeserializeOwned + zbus::zvariant::Type,
    {
        self.bus.call(BUS_NAME, OBJECT_PATH, INTERFACE, method, body).await
    }

    // ----------------------------------------------------------- temp limit

    /// Saved CPU temperature limit of a platform profile ("Quiet", "Balanced",
    /// "Performance"); 0 = no saved limit.
    pub async fn get_profile_temp_limit(&self, profile: &str) -> Result<i32> {
        trace(
            "DbusClient.getProfileTempLimit",
            &[("profile", profile.to_string())],
            self.call("GetProfileTempLimit", &(profile,)),
        )
        .await
    }

    /// Saves a profile's limit (0 clears it); applied only while that profile is active.
    pub async fn set_profile_temp_limit(&self, profile: &str, temp_limit_c: i32) -> Result<bool> {
        trace(
            "DbusClient.setProfileTempLimit",
            &[("profile", profile.to_string()), ("tempLimitC", temp_limit_c.to_string())],
            self.call("SetProfileTempLimit", &(profile, temp_limit_c)),
        )
        .await
    }

    // ------------------------------------------------------------ CPU boost

    /// JSON status payload, parsed by `features::cpu_boost`.
    pub async fn get_cpu_boost_status(&self) -> Result<String> {
        trace(
            "CpuBoostDaemonClient.getCpuBoostStatus",
            &[],
            self.call("GetCpuBoostStatus", &()),
        )
        .await
    }

    pub async fn set_cpu_boost_mode(&self, mode_id: &str) -> Result<()> {
        trace(
            "CpuBoostDaemonClient.setCpuBoostMode",
            &[("mode", mode_id.to_string())],
            async {
                let applied: bool = self.call("SetCpuBoostMode", &(mode_id,)).await?;
                if !applied {
                    bail!("CPU boost mode was not applied");
                }
                Ok(())
            },
        )
        .await
    }

    // ----------------------------------------------------------- fan engine

    async fn checked(&self, method: &str, ok: bool) -> Result<()> {
        if ok {
            Ok(())
        } else {
            Err(anyhow!("{method} returned false"))
        }
    }

    pub async fn start_fan_engine(&self, config_json: &str) -> Result<()> {
        trace("FanEngineDaemonClient.startFanEngine", &[], async {
            let ok: bool = self.call("StartFanEngine", &(config_json,)).await?;
            self.checked("StartFanEngine", ok)
                .await
                .map_err(|_| anyhow!("Failed to start fan engine"))
        })
        .await
    }

    pub async fn stop_fan_engine(&self) -> Result<()> {
        trace("FanEngineDaemonClient.stopFanEngine", &[], async {
            let ok: bool = self.call("StopFanEngine", &()).await?;
            self.checked("StopFanEngine", ok)
                .await
                .map_err(|_| anyhow!("Failed to stop fan engine"))
        })
        .await
    }

    pub async fn update_fan_engine_curve(&self, fan_id: &str, curve_json: &str) -> Result<()> {
        trace(
            "FanEngineDaemonClient.updateFanCurve",
            &[("fanId", fan_id.to_string())],
            async {
                let ok: bool = self.call("UpdateFanEngineCurve", &(fan_id, curve_json)).await?;
                self.checked("UpdateFanEngineCurve", ok)
                    .await
                    .map_err(|_| anyhow!("Failed to update fan curve"))
            },
        )
        .await
    }

    pub async fn get_fan_engine_running(&self) -> Result<bool> {
        trace(
            "FanEngineDaemonClient.getRunning",
            &[],
            self.call("GetFanEngineRunning", &()),
        )
        .await
    }

    // ------------------------------------------------------ package updates

    async fn package_call<B, R>(&self, method: &str, body: &B) -> Result<R>
    where
        B: serde::Serialize + zbus::zvariant::DynamicType,
        R: serde::de::DeserializeOwned + zbus::zvariant::Type,
    {
        trace(&format!("DbusPackageUpdateService.{method}"), &[], async {
            self.call(method, body)
                .await
                .map_err(|e| anyhow!("Package update daemon unavailable: {e:#}"))
        })
        .await
    }

    pub async fn get_package_backend(&self) -> Result<String> {
        self.package_call("GetPackageBackend", &()).await
    }

    pub async fn check_for_updates(&self) -> Result<String> {
        self.package_call("CheckForUpdates", &()).await
    }

    pub async fn start_update(&self) -> Result<String> {
        self.package_call("StartUpdate", &()).await
    }

    pub async fn get_update_status(&self, update_id: &str) -> Result<String> {
        self.package_call("GetUpdateStatus", &(update_id,)).await
    }

    pub async fn cancel_update(&self, update_id: &str) -> Result<bool> {
        self.package_call("CancelUpdate", &(update_id,)).await
    }
}
