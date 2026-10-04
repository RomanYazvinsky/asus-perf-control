//! GPU modes via supergfxd — former `GpuBloc`, `SupergfxGpuService` and GPU models.

use anyhow::{anyhow, Result};
use async_trait::async_trait;

use crate::backend::supergfx::SupergfxMode;
use crate::logging::log_hardware_mismatch;
use crate::settings::{update_hardware, AppSettingsStore};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GpuMode {
    Integrated,
    Hybrid,
    Ultimate,
    AsusMuxDgpu,
    AsusEgpu,
    Vfio,
}

impl GpuMode {
    /// Display order of the mode selector.
    pub const ALL: [GpuMode; 6] = [
        GpuMode::Integrated,
        GpuMode::Hybrid,
        GpuMode::Ultimate,
        GpuMode::AsusMuxDgpu,
        GpuMode::AsusEgpu,
        GpuMode::Vfio,
    ];

    /// Name persisted in settings (`hardware.gpuMode`).
    pub fn name(self) -> &'static str {
        match self {
            GpuMode::Integrated => "integrated",
            GpuMode::Hybrid => "hybrid",
            GpuMode::Ultimate => "ultimate",
            GpuMode::AsusMuxDgpu => "asusMuxDgpu",
            GpuMode::AsusEgpu => "asusEgpu",
            GpuMode::Vfio => "vfio",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            GpuMode::Integrated => "Integrated",
            GpuMode::Hybrid => "Hybrid",
            GpuMode::Ultimate => "Ultimate",
            GpuMode::AsusMuxDgpu => "MUX dGPU",
            GpuMode::AsusEgpu => "eGPU",
            GpuMode::Vfio => "VFIO",
        }
    }

    pub fn from_supergfx(mode: SupergfxMode) -> Self {
        match mode {
            SupergfxMode::Integrated => GpuMode::Integrated,
            SupergfxMode::Vfio => GpuMode::Vfio,
            SupergfxMode::AsusMuxDgpu => GpuMode::AsusMuxDgpu,
            SupergfxMode::AsusEgpu => GpuMode::AsusEgpu,
            _ => GpuMode::Hybrid,
        }
    }

    /// Ultimate maps to Hybrid in supergfxctl.
    pub fn to_supergfx(self) -> SupergfxMode {
        match self {
            GpuMode::Integrated => SupergfxMode::Integrated,
            GpuMode::Hybrid | GpuMode::Ultimate => SupergfxMode::Hybrid,
            GpuMode::AsusMuxDgpu => SupergfxMode::AsusMuxDgpu,
            GpuMode::AsusEgpu => SupergfxMode::AsusEgpu,
            GpuMode::Vfio => SupergfxMode::Vfio,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GpuVendor {
    Nvidia,
    Amd,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GpuPowerState {
    Active,
    Idle,
    Suspended,
    Unknown,
}

impl GpuPowerState {
    pub fn label(self) -> &'static str {
        match self {
            GpuPowerState::Active => "Active",
            GpuPowerState::Idle => "Idle",
            GpuPowerState::Suspended => "Suspended",
            GpuPowerState::Unknown => "Unknown",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GpuConfiguration {
    pub has_dgpu: bool,
    pub dgpu_vendor: Option<GpuVendor>,
    pub has_mux_switch: bool,
    pub supports_egpu: bool,
    pub external_gpu_connected: bool,
    pub vfio_available: bool,
}

impl GpuConfiguration {
    pub fn from_supergfx(supported: &[SupergfxMode], vendor: &str) -> Self {
        let vendor = vendor.to_lowercase();
        Self {
            has_dgpu: vendor != "unknown",
            dgpu_vendor: match vendor.as_str() {
                "nvidia" => Some(GpuVendor::Nvidia),
                "amd" => Some(GpuVendor::Amd),
                _ => None,
            },
            has_mux_switch: supported.contains(&SupergfxMode::AsusMuxDgpu),
            supports_egpu: supported.contains(&SupergfxMode::AsusEgpu),
            external_gpu_connected: supported.contains(&SupergfxMode::AsusEgpu),
            vfio_available: supported.contains(&SupergfxMode::Vfio),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GpuStatus {
    pub active_mode: GpuMode,
    pub dgpu_enabled: bool,
    pub temperature: Option<f64>,
    pub utilization: Option<f64>,
    pub power_state: GpuPowerState,
}

impl GpuStatus {
    pub fn from_supergfx(mode: SupergfxMode) -> Self {
        Self {
            active_mode: GpuMode::from_supergfx(mode),
            dgpu_enabled: mode != SupergfxMode::Integrated,
            temperature: None,
            utilization: None,
            power_state: if mode == SupergfxMode::Integrated {
                GpuPowerState::Suspended
            } else {
                GpuPowerState::Active
            },
        }
    }
}

#[async_trait]
pub trait GpuService: Send + Sync {
    async fn get_capabilities(&self) -> Result<GpuConfiguration>;
    async fn get_status(&self) -> Result<GpuStatus>;
    async fn get_supported_modes(&self) -> Result<Vec<GpuMode>>;
    /// Fails with a message containing `reboot`/`logout` when the user must act.
    async fn set_gpu_mode(&self, mode: GpuMode) -> Result<()>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct GpuLoaded {
    pub capabilities: GpuConfiguration,
    pub status: GpuStatus,
    pub supported_modes: Vec<GpuMode>,
}

impl GpuLoaded {
    /// eGPU can only be chosen while XG Mobile is connected (or already active).
    pub fn is_selectable(&self, mode: GpuMode) -> bool {
        mode != GpuMode::AsusEgpu
            || self.capabilities.external_gpu_connected
            || self.status.active_mode == GpuMode::AsusEgpu
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum GpuState {
    Initial,
    Loading,
    Loaded(GpuLoaded),
    Unsupported(String),
    Error(String),
    RequiresReboot(String),
    RequiresLogout(String),
}

pub async fn load_capabilities(service: &dyn GpuService, store: &dyn AppSettingsStore) -> GpuState {
    let result: Result<GpuState> = async {
        let capabilities = service.get_capabilities().await?;
        if !capabilities.has_dgpu {
            return Ok(GpuState::Unsupported(
                "GPU hardware not detected. Check that discrete GPU is present.".into(),
            ));
        }
        let status = service.get_status().await?;
        let supported_modes = service.get_supported_modes().await?;
        log_saved_mode_mismatch(store, &status)?;
        Ok(GpuState::Loaded(GpuLoaded {
            capabilities,
            status,
            supported_modes,
        }))
    }
    .await;
    result.unwrap_or_else(|e| GpuState::Error(format!("Failed to load GPU capabilities: {e:#}")))
}

/// GPU mode is never re-applied automatically (it can need a reboot); a mismatch is only logged.
fn log_saved_mode_mismatch(store: &dyn AppSettingsStore, status: &GpuStatus) -> Result<()> {
    let active = status.active_mode.name();
    match store.load().hardware.gpu_mode {
        None => {
            update_hardware(store, |hw| hw.gpu_mode = Some(active.to_string()))?;
        }
        Some(desired) if desired != active => log_hardware_mismatch("gpuMode", desired, active),
        Some(_) => {}
    }
    Ok(())
}

/// Result of a mode change: the next state plus an optional error to show while keeping it.
pub struct SetGpuModeOutcome {
    pub state: GpuState,
    pub error: Option<String>,
}

pub async fn set_mode(
    service: &dyn GpuService,
    store: &dyn AppSettingsStore,
    current: GpuLoaded,
    mode: GpuMode,
) -> SetGpuModeOutcome {
    if !current.supported_modes.contains(&mode) {
        let supported: Vec<&str> = current.supported_modes.iter().map(|m| m.name()).collect();
        return SetGpuModeOutcome {
            error: Some(format!(
                "GPU mode \"{}\" is not supported on this laptop model. Supported modes: {}",
                mode.name(),
                supported.join(", ")
            )),
            state: GpuState::Loaded(current),
        };
    }

    let result: Result<GpuStatus> = async {
        service.set_gpu_mode(mode).await?;
        update_hardware(store, |hw| hw.gpu_mode = Some(mode.name().to_string()))?;
        service.get_status().await
    }
    .await;

    let state = match result {
        Ok(status) => GpuState::Loaded(GpuLoaded { status, ..current }),
        Err(e) => classify_set_mode_error(&format!("{e:#}")),
    };
    SetGpuModeOutcome { state, error: None }
}

fn classify_set_mode_error(error: &str) -> GpuState {
    if error.contains("reboot") || error.contains("RebootRequired") {
        GpuState::RequiresReboot(
            "Switching to MUX dGPU mode requires a system reboot. The MUX switch will physically route \
             display output directly to the dGPU for maximum performance."
                .into(),
        )
    } else if error.contains("logout") || error.contains("LogoutRequired") {
        GpuState::RequiresLogout(
            "GPU mode changed. Please save your work and log out, then log back in to finish applying the change."
                .into(),
        )
    } else if error.contains("eGPU not connected") || error.contains("EgpuNotConnected") {
        GpuState::Error("External GPU not connected. Connect XG Mobile before enabling eGPU mode.".into())
    } else if error.contains("VFIO") || error.contains("VfioNotAvailable") {
        GpuState::Error("VFIO kernel modules not available. Install vfio-pci module for GPU passthrough.".into())
    } else {
        GpuState::Error(format!("Failed to set GPU mode: {error}"))
    }
}

/// supergfxctl user action → error text understood by [`classify_set_mode_error`].
pub fn user_action_error(action: crate::backend::supergfx::SupergfxUserAction) -> Option<anyhow::Error> {
    use crate::backend::supergfx::SupergfxUserAction as A;
    match action {
        A::Nothing => None,
        A::Reboot => Some(anyhow!("Reboot required. Please save your work and reboot.")),
        A::Logout => Some(anyhow!("Logout required. Please save your work and logout.")),
        other => Some(anyhow!("Action required: {}", other.name())),
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use super::*;
    use crate::settings::{AppSettings, HardwareSettings, MemorySettingsStore};

    struct Fake {
        mode: Mutex<GpuMode>,
        set_error: Option<&'static str>,
        vendor_known: bool,
    }

    #[async_trait]
    impl GpuService for Fake {
        async fn get_capabilities(&self) -> Result<GpuConfiguration> {
            let vendor = if self.vendor_known { "Nvidia" } else { "Unknown" };
            Ok(GpuConfiguration::from_supergfx(
                &[SupergfxMode::Hybrid, SupergfxMode::Integrated, SupergfxMode::AsusMuxDgpu],
                vendor,
            ))
        }
        async fn get_status(&self) -> Result<GpuStatus> {
            Ok(GpuStatus::from_supergfx(self.mode.lock().unwrap().to_supergfx()))
        }
        async fn get_supported_modes(&self) -> Result<Vec<GpuMode>> {
            Ok(vec![GpuMode::Hybrid, GpuMode::Integrated, GpuMode::AsusMuxDgpu])
        }
        async fn set_gpu_mode(&self, mode: GpuMode) -> Result<()> {
            if let Some(error) = self.set_error {
                anyhow::bail!("{error}");
            }
            *self.mode.lock().unwrap() = mode;
            Ok(())
        }
    }

    fn fake(set_error: Option<&'static str>) -> Fake {
        Fake {
            mode: Mutex::new(GpuMode::Hybrid),
            set_error,
            vendor_known: true,
        }
    }

    fn store() -> MemorySettingsStore {
        MemorySettingsStore::new(AppSettings {
            hardware: HardwareSettings::default(),
            ..Default::default()
        })
    }

    async fn loaded(service: &Fake, store: &MemorySettingsStore) -> GpuLoaded {
        match load_capabilities(service, store).await {
            GpuState::Loaded(loaded) => loaded,
            other => panic!("unexpected {other:?}"),
        }
    }

    #[tokio::test]
    async fn loads_and_saves_active_mode() {
        let service = fake(None);
        let store = store();

        let loaded = loaded(&service, &store).await;

        assert_eq!(loaded.status.active_mode, GpuMode::Hybrid);
        assert!(loaded.capabilities.has_mux_switch);
        assert_eq!(store.load().hardware.gpu_mode.as_deref(), Some("hybrid"));
    }

    #[tokio::test]
    async fn unknown_vendor_is_unsupported() {
        let service = Fake {
            vendor_known: false,
            ..fake(None)
        };

        assert!(matches!(load_capabilities(&service, &store()).await, GpuState::Unsupported(_)));
    }

    #[tokio::test]
    async fn rejects_unsupported_mode_but_keeps_loaded_state() {
        let service = fake(None);
        let store = store();
        let current = loaded(&service, &store).await;

        let outcome = set_mode(&service, &store, current.clone(), GpuMode::Vfio).await;

        assert_eq!(outcome.state, GpuState::Loaded(current));
        assert!(outcome.error.unwrap().contains("not supported"));
    }

    #[tokio::test]
    async fn switches_mode_and_saves_it() {
        let service = fake(None);
        let store = store();
        let current = loaded(&service, &store).await;

        let outcome = set_mode(&service, &store, current, GpuMode::Integrated).await;

        assert!(matches!(outcome.state, GpuState::Loaded(l) if l.status.active_mode == GpuMode::Integrated));
        assert_eq!(store.load().hardware.gpu_mode.as_deref(), Some("integrated"));
    }

    #[tokio::test]
    async fn maps_reboot_and_logout_actions() {
        let store = store();
        let reboot = fake(Some("Reboot required. Please save your work and reboot."));
        let current = loaded(&reboot, &store).await;
        let outcome = set_mode(&reboot, &store, current.clone(), GpuMode::AsusMuxDgpu).await;
        assert!(matches!(outcome.state, GpuState::RequiresReboot(_)));

        let logout = fake(Some("Logout required. Please save your work and logout."));
        let outcome = set_mode(&logout, &store, current, GpuMode::Integrated).await;
        assert!(matches!(outcome.state, GpuState::RequiresLogout(_)));
    }

    #[test]
    fn egpu_selectable_only_when_connected() {
        let mut loaded = GpuLoaded {
            capabilities: GpuConfiguration::from_supergfx(&[SupergfxMode::Hybrid], "Nvidia"),
            status: GpuStatus::from_supergfx(SupergfxMode::Hybrid),
            supported_modes: vec![GpuMode::Hybrid, GpuMode::AsusEgpu],
        };
        assert!(!loaded.is_selectable(GpuMode::AsusEgpu));
        assert!(loaded.is_selectable(GpuMode::Hybrid));
        loaded.capabilities.external_gpu_connected = true;
        assert!(loaded.is_selectable(GpuMode::AsusEgpu));
    }
}
