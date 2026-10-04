//! Client for asusd (`xyz.ljones.Asusd`): platform profile, PPT power limits,
//! battery charge limit and fan curves.

use std::sync::Arc;

use anyhow::{anyhow, bail, Context, Result};
use zbus::zvariant::Value;

use super::bus::{value_as_bool, value_as_i64, SystemBus};
use crate::features::fan::model::{FanCurveData, PerfProfile};
use crate::logging::trace;

const BUS_NAME: &str = "xyz.ljones.Asusd";
const ARMOURY_IFACE: &str = "xyz.ljones.AsusArmoury";
const PLATFORM_IFACE: &str = "xyz.ljones.Platform";
const FAN_CURVES_IFACE: &str = "xyz.ljones.FanCurves";
const PLATFORM_PATH: &str = "/xyz/ljones";
const PL1_PATH: &str = "/xyz/ljones/asus_armoury/ppt_pl1_spl";
const PL2_PATH: &str = "/xyz/ljones/asus_armoury/ppt_pl2_sppt";
const CHARGE_LIMIT_PROP: &str = "ChargeControlEndThreshold";
const ENABLE_PPT_GROUP_PROP: &str = "EnablePptGroup";

/// asusd PlatformProfile → UI mode. Balanced=0→1, Performance=1→2, Quiet=2→0, LowPower=3→0.
pub fn mode_from_platform_profile(value: u32) -> Result<i32> {
    match value {
        0 => Ok(1),
        1 => Ok(2),
        2 | 3 => Ok(0),
        other => bail!("Unknown PlatformProfile value: {other}"),
    }
}

/// UI mode → asusd PlatformProfile. Silent is written as Quiet because some machines list
/// LowPower but reject `platform_profile=low-power`.
pub fn platform_profile_from_mode(mode: i32) -> Result<u32> {
    match mode {
        0 => Ok(2),
        1 => Ok(0),
        2 => Ok(1),
        other => bail!("Invalid mode: {other}. Must be 0, 1, or 2"),
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AsusdTdpConfig {
    pub sustained_w: i32,
    pub boost_w: i32,
    pub min_sustained_w: i32,
    pub max_sustained_w: i32,
    pub min_boost_w: i32,
    pub max_boost_w: i32,
    pub power_limits_enabled: bool,
}

type Curve8 = (u8, u8, u8, u8, u8, u8, u8, u8);
type CurveStruct = (String, Curve8, Curve8, bool);

fn curve8_to_vec(c: Curve8) -> Vec<u8> {
    vec![c.0, c.1, c.2, c.3, c.4, c.5, c.6, c.7]
}

fn vec_to_curve8(v: &[u8]) -> Result<Curve8> {
    match v {
        [a, b, c, d, e, f, g, h] => Ok((*a, *b, *c, *d, *e, *f, *g, *h)),
        _ => Err(anyhow!("fan curve requires exactly 8 points, got {}", v.len())),
    }
}

pub struct AsusdClient {
    bus: Arc<SystemBus>,
}

impl AsusdClient {
    pub fn new(bus: Arc<SystemBus>) -> Self {
        Self { bus }
    }

    async fn platform_prop(&self, prop: &str) -> Result<zbus::zvariant::OwnedValue> {
        self.bus.get_property(BUS_NAME, PLATFORM_PATH, PLATFORM_IFACE, prop).await
    }

    async fn set_platform_prop(&self, prop: &str, value: Value<'_>) -> Result<()> {
        self.bus
            .set_property(BUS_NAME, PLATFORM_PATH, PLATFORM_IFACE, prop, value)
            .await
    }

    // ---------------------------------------------------------------- profile

    pub async fn get_platform_profile(&self) -> Result<u32> {
        trace("AsusdClient.getPlatformProfile", &[], async {
            let value = self.platform_prop("PlatformProfile").await?;
            u32::try_from(value_as_i64(&value)?).context("PlatformProfile out of range")
        })
        .await
    }

    pub async fn set_platform_profile(&self, profile: u32) -> Result<()> {
        trace(
            "AsusdClient.setPlatformProfile",
            &[("profile", profile.to_string())],
            self.set_platform_prop("PlatformProfile", Value::U32(profile)),
        )
        .await
    }

    /// Current performance mode (0 Silent, 1 Balanced, 2 Turbo).
    pub async fn get_performance_mode(&self) -> Result<i32> {
        mode_from_platform_profile(self.get_platform_profile().await?)
    }

    pub async fn set_performance_mode(&self, mode: i32) -> Result<()> {
        self.set_platform_profile(platform_profile_from_mode(mode)?).await
    }

    // -------------------------------------------------------------- PPT / TDP

    async fn armoury_int(&self, path: &str, prop: &str) -> Result<i32> {
        let value = self.bus.get_property(BUS_NAME, path, ARMOURY_IFACE, prop).await?;
        i32::try_from(value_as_i64(&value)?).context("value out of range")
    }

    /// Real min/max limits are model-specific; boost (`ppt_pl2_sppt`) is optional.
    pub async fn get_tdp_config(&self) -> Result<AsusdTdpConfig> {
        trace("AsusdClient.getTdpConfig", &[], async {
            let result: Result<AsusdTdpConfig> = async {
                let sustained_w = self.armoury_int(PL1_PATH, "CurrentValue").await?;
                let min_sustained_w = self.armoury_int(PL1_PATH, "MinValue").await?;
                let max_sustained_w = self.armoury_int(PL1_PATH, "MaxValue").await?;
                let power_limits_enabled = self.get_power_limits_enabled().await.unwrap_or(false);
                let (boost_w, min_boost_w, max_boost_w) = if self.bus.object_exists(BUS_NAME, PL2_PATH).await {
                    (
                        self.armoury_int(PL2_PATH, "CurrentValue").await?,
                        self.armoury_int(PL2_PATH, "MinValue").await?,
                        self.armoury_int(PL2_PATH, "MaxValue").await?,
                    )
                } else {
                    (0, 0, 0)
                };
                Ok(AsusdTdpConfig {
                    sustained_w,
                    boost_w,
                    min_sustained_w,
                    max_sustained_w,
                    min_boost_w,
                    max_boost_w,
                    power_limits_enabled,
                })
            }
            .await;
            result.map_err(|e| {
                let text = format!("{e:#}");
                if ["UnknownInterface", "No such interface", "UnknownMethod", "InvalidArgs"]
                    .iter()
                    .any(|needle| text.contains(needle))
                {
                    anyhow!(
                        "asusd TDP interface not found. Ensure asusd is running (systemctl status asusd) \
                         and kernel supports armoury class (5.17+). Original error: {text}"
                    )
                } else {
                    anyhow!("Failed to communicate with asusd. Is asusd running? (systemctl status asusd). Error: {text}")
                }
            })
        })
        .await
    }

    pub async fn set_tdp_config(&self, sustained_w: i32, boost_w: i32) -> Result<()> {
        trace(
            "AsusdClient.setTdpConfig",
            &[("sustainedW", sustained_w.to_string()), ("boostW", boost_w.to_string())],
            async {
                self.bus
                    .set_property(BUS_NAME, PL1_PATH, ARMOURY_IFACE, "CurrentValue", Value::I32(sustained_w))
                    .await
                    .map_err(|e| anyhow!("PL1/SPL set to {sustained_w}W failed: {e:#}"))?;
                if self.bus.object_exists(BUS_NAME, PL2_PATH).await {
                    self.bus
                        .set_property(BUS_NAME, PL2_PATH, ARMOURY_IFACE, "CurrentValue", Value::I32(boost_w))
                        .await
                        .map_err(|e| anyhow!("PL2/SPPT set to {boost_w}W failed: {e:#}"))?;
                }
                Ok(())
            },
        )
        .await
    }

    pub async fn reset_power_limits_to_profile_defaults(&self) -> Result<()> {
        trace("AsusdClient.resetPowerLimitsToProfileDefaults", &[], async {
            self.bus
                .call_void(BUS_NAME, PL1_PATH, ARMOURY_IFACE, "RestoreDefault", &())
                .await?;
            if self.bus.object_exists(BUS_NAME, PL2_PATH).await {
                self.bus
                    .call_void(BUS_NAME, PL2_PATH, ARMOURY_IFACE, "RestoreDefault", &())
                    .await?;
            }
            Ok(())
        })
        .await
    }

    pub async fn get_power_limits_enabled(&self) -> Result<bool> {
        value_as_bool(&self.platform_prop(ENABLE_PPT_GROUP_PROP).await?)
    }

    pub async fn set_power_limits_enabled(&self, enabled: bool) -> Result<()> {
        trace(
            "AsusdClient.setPowerLimitsEnabled",
            &[("enabled", enabled.to_string())],
            self.set_platform_prop(ENABLE_PPT_GROUP_PROP, Value::Bool(enabled)),
        )
        .await
    }

    // ---------------------------------------------------------- charge limit

    pub async fn is_charge_limit_supported(&self) -> bool {
        self.platform_prop(CHARGE_LIMIT_PROP).await.is_ok()
    }

    /// 0–100, 0 = no limit. asusd exposes this as a byte.
    pub async fn get_charge_limit(&self) -> Result<i32> {
        trace("AsusdClient.getChargeLimit", &[], async {
            let value = self
                .platform_prop(CHARGE_LIMIT_PROP)
                .await
                .context("Failed to get battery charge limit from asusd")?;
            Ok(value_as_i64(&value)? as i32)
        })
        .await
    }

    pub async fn set_charge_limit(&self, limit: i32) -> Result<()> {
        trace("AsusdClient.setChargeLimit", &[("limit", limit.to_string())], async {
            if !(0..=100).contains(&limit) {
                bail!("Charge limit must be 0-100%, got {limit}");
            }
            self.set_platform_prop(CHARGE_LIMIT_PROP, Value::U8(limit as u8))
                .await
                .context("Failed to set battery charge limit via asusd")
        })
        .await
    }

    // ------------------------------------------------------------ fan curves

    pub async fn set_active_profile(&self, profile: PerfProfile) -> Result<()> {
        self.set_platform_profile(profile.asusd_id()).await
    }

    pub async fn get_fan_curves(&self, profile: PerfProfile) -> Result<Vec<FanCurveData>> {
        trace(
            "AsusdFanClient.getFanCurves",
            &[("profile", profile.name().to_string())],
            async {
                let curves: Vec<CurveStruct> = self
                    .bus
                    .call(BUS_NAME, PLATFORM_PATH, FAN_CURVES_IFACE, "FanCurveData", &(profile.asusd_id(),))
                    .await?;
                Ok(curves
                    .into_iter()
                    .map(|(name, pwm, temp, enabled)| FanCurveData {
                        name,
                        pwm_values: curve8_to_vec(pwm),
                        temp_values: curve8_to_vec(temp),
                        enabled,
                    })
                    .collect())
            },
        )
        .await
    }

    pub async fn set_fan_curve(&self, profile: PerfProfile, data: &FanCurveData) -> Result<()> {
        trace(
            "AsusdFanClient.setFanCurve",
            &[
                ("asusdProfile", profile.asusd_id().to_string()),
                ("fan", data.name.clone()),
                ("enabled", data.enabled.to_string()),
            ],
            async {
                let curve: CurveStruct = (
                    data.name.clone(),
                    vec_to_curve8(&data.pwm_values)?,
                    vec_to_curve8(&data.temp_values)?,
                    data.enabled,
                );
                self.bus
                    .call_void(BUS_NAME, PLATFORM_PATH, FAN_CURVES_IFACE, "SetFanCurve", &(profile.asusd_id(), curve))
                    .await
            },
        )
        .await
    }

    /// `false` returns the fan to factory auto mode.
    pub async fn set_fan_curve_enabled(&self, profile: PerfProfile, fan_name: &str, enabled: bool) -> Result<()> {
        trace(
            "AsusdFanClient.setFanCurveEnabled",
            &[
                ("asusdProfile", profile.asusd_id().to_string()),
                ("fan", fan_name.to_string()),
                ("enabled", enabled.to_string()),
            ],
            self.bus.call_void(
                BUS_NAME,
                PLATFORM_PATH,
                FAN_CURVES_IFACE,
                "SetProfileFanCurveEnabled",
                &(profile.asusd_id(), fan_name, enabled),
            ),
        )
        .await
    }

    pub async fn reset_curves_to_defaults(&self, profile: PerfProfile) -> Result<()> {
        trace(
            "AsusdFanClient.resetCurvesToDefaults",
            &[("asusdProfile", profile.asusd_id().to_string())],
            self.bus.call_void(
                BUS_NAME,
                PLATFORM_PATH,
                FAN_CURVES_IFACE,
                "SetCurvesToDefaults",
                &(profile.asusd_id(),),
            ),
        )
        .await
    }

    /// Writes the curve and enables it.
    pub async fn apply_fan_curve(&self, profile: PerfProfile, data: &FanCurveData) -> Result<()> {
        self.set_fan_curve(profile, data).await?;
        self.set_fan_curve_enabled(profile, &data.name, true).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_platform_profiles_to_modes() {
        assert_eq!(mode_from_platform_profile(0).unwrap(), 1);
        assert_eq!(mode_from_platform_profile(1).unwrap(), 2);
        assert_eq!(mode_from_platform_profile(2).unwrap(), 0);
        assert_eq!(mode_from_platform_profile(3).unwrap(), 0);
        assert!(mode_from_platform_profile(4).is_err());
    }

    #[test]
    fn writes_silent_as_quiet() {
        assert_eq!(platform_profile_from_mode(0).unwrap(), 2);
        assert_eq!(platform_profile_from_mode(1).unwrap(), 0);
        assert_eq!(platform_profile_from_mode(2).unwrap(), 1);
        assert!(platform_profile_from_mode(3).is_err());
    }

    #[test]
    fn curve_struct_signature_matches_asusd() {
        use zbus::zvariant::Type;
        assert_eq!(
            <Vec<CurveStruct> as Type>::signature().to_string(),
            "a(s(yyyyyyyy)(yyyyyyyy)b)"
        );
    }
}
