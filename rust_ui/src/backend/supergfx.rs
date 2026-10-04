//! Client for supergfxd (`org.supergfxctl.Daemon`) GPU mode switching.

use std::sync::Arc;

use anyhow::Result;

use super::bus::SystemBus;
use crate::logging::trace;

const BUS_NAME: &str = "org.supergfxctl.Daemon";
const OBJECT_PATH: &str = "/org/supergfxctl/Gfx";
const INTERFACE: &str = "org.supergfxctl.Daemon";

/// supergfxctl GPU modes (D-Bus u32 values).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SupergfxMode {
    Hybrid,
    Integrated,
    NvidiaNoModeset,
    Vfio,
    AsusEgpu,
    AsusMuxDgpu,
    None,
}

impl SupergfxMode {
    pub fn from_value(value: u32) -> Self {
        match value {
            0 => Self::Hybrid,
            1 => Self::Integrated,
            2 => Self::NvidiaNoModeset,
            3 => Self::Vfio,
            4 => Self::AsusEgpu,
            5 => Self::AsusMuxDgpu,
            _ => Self::None,
        }
    }

    pub fn value(self) -> u32 {
        match self {
            Self::Hybrid => 0,
            Self::Integrated => 1,
            Self::NvidiaNoModeset => 2,
            Self::Vfio => 3,
            Self::AsusEgpu => 4,
            Self::AsusMuxDgpu => 5,
            Self::None => 6,
        }
    }
}

/// Action the user must take after a mode change.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SupergfxUserAction {
    Logout,
    Reboot,
    SwitchToIntegrated,
    AsusEgpuDisable,
    Nothing,
}

impl SupergfxUserAction {
    pub fn from_value(value: u32) -> Self {
        match value {
            0 => Self::Logout,
            1 => Self::Reboot,
            2 => Self::SwitchToIntegrated,
            3 => Self::AsusEgpuDisable,
            _ => Self::Nothing,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::Logout => "logout",
            Self::Reboot => "reboot",
            Self::SwitchToIntegrated => "switchToIntegrated",
            Self::AsusEgpuDisable => "asusEgpuDisable",
            Self::Nothing => "nothing",
        }
    }
}

pub struct SupergfxClient {
    bus: Arc<SystemBus>,
}

impl SupergfxClient {
    pub fn new(bus: Arc<SystemBus>) -> Self {
        Self { bus }
    }

    pub async fn mode(&self) -> Result<SupergfxMode> {
        trace("SupergfxClient.getMode", &[], async {
            let value: u32 = self.bus.call(BUS_NAME, OBJECT_PATH, INTERFACE, "Mode", &()).await?;
            Ok(SupergfxMode::from_value(value))
        })
        .await
    }

    pub async fn supported(&self) -> Result<Vec<SupergfxMode>> {
        trace("SupergfxClient.getSupportedModes", &[], async {
            let values: Vec<u32> = self
                .bus
                .call(BUS_NAME, OBJECT_PATH, INTERFACE, "Supported", &())
                .await?;
            Ok(values.into_iter().map(SupergfxMode::from_value).collect())
        })
        .await
    }

    /// `Nvidia`, `AMD` or `Unknown` (also on error).
    pub async fn vendor(&self) -> String {
        trace("SupergfxClient.getVendor", &[], async {
            self.bus
                .call::<_, String>(BUS_NAME, OBJECT_PATH, INTERFACE, "Vendor", &())
                .await
        })
        .await
        .unwrap_or_else(|_| "Unknown".to_string())
    }

    pub async fn set_mode(&self, mode: SupergfxMode) -> Result<SupergfxUserAction> {
        trace("SupergfxClient.setMode", &[("mode", format!("{mode:?}"))], async {
            let action: u32 = self
                .bus
                .call(BUS_NAME, OBJECT_PATH, INTERFACE, "SetMode", &(mode.value(),))
                .await?;
            Ok(SupergfxUserAction::from_value(action))
        })
        .await
    }
}
