//! GPU modes (former `GpuScreen`, `GpuStatusCard`, `GpuModeSelector`).

use egui::{RichText, Ui};

use crate::features::gpu::{self, GpuLoaded, GpuMode, GpuState, SetGpuModeOutcome};
use crate::runtime::{take_ready, Task};
use crate::ui::alert::Alert;
use crate::ui::{theme, widgets, ScreenCtx};

pub struct GpuScreen {
    state: GpuState,
    load_task: Option<Task<GpuState>>,
    set_task: Option<Task<SetGpuModeOutcome>>,
}

fn mode_subtitle(mode: GpuMode, loaded: &GpuLoaded) -> &'static str {
    match mode {
        GpuMode::Integrated => "Battery saving - dGPU disabled",
        GpuMode::Hybrid => "Balanced - dGPU on demand",
        GpuMode::Ultimate => "Maximum performance - dGPU primary",
        GpuMode::AsusMuxDgpu => "Direct dGPU output (requires reboot)",
        GpuMode::AsusEgpu if loaded.capabilities.external_gpu_connected => "External GPU (ASUS XG Mobile)",
        GpuMode::AsusEgpu => "Connect XG Mobile to enable",
        GpuMode::Vfio => "GPU passthrough for VMs",
    }
}

impl GpuScreen {
    pub fn new(cx: &mut ScreenCtx) -> Self {
        let mut screen = Self {
            state: GpuState::Initial,
            load_task: None,
            set_task: None,
        };
        screen.load(cx);
        screen
    }

    fn load(&mut self, cx: &mut ScreenCtx) {
        self.state = GpuState::Loading;
        let (services, store) = (cx.services.clone(), cx.store.clone());
        self.load_task = Some(
            cx.rt
                .spawn(async move { gpu::load_capabilities(&*services, &*store).await }),
        );
    }

    pub fn poll(&mut self, cx: &mut ScreenCtx) {
        if let Some(state) = take_ready(&mut self.load_task) {
            self.apply(state, cx);
        }
        if let Some(outcome) = take_ready(&mut self.set_task) {
            if let Some(error) = outcome.error {
                cx.alert(Alert::error(
                    "GPU Error",
                    "An error occurred while performing a GPU operation.",
                    error,
                ));
            }
            self.apply(outcome.state, cx);
        }
    }

    fn apply(&mut self, state: GpuState, cx: &mut ScreenCtx) {
        match &state {
            GpuState::RequiresReboot(message) => cx.alert(Alert::info("Reboot Required", message.clone(), None)),
            GpuState::RequiresLogout(message) => cx.alert(Alert::info("Logout Required", message.clone(), None)),
            GpuState::Unsupported(message) => cx.alert(Alert::info(
                "GPU Not Supported",
                "Discrete GPU hardware not detected on this system.",
                Some(message.clone()),
            )),
            GpuState::Error(message) => cx.alert(Alert::error(
                "GPU Error",
                "An error occurred while performing a GPU operation.",
                message.clone(),
            )),
            _ => {}
        }
        self.state = state;
    }

    pub fn ui(&mut self, ui: &mut Ui, cx: &mut ScreenCtx) {
        widgets::screen_title(ui, "GPU Modes");
        let loaded = match &self.state {
            GpuState::Loading | GpuState::Initial => return widgets::loading(ui),
            GpuState::Unsupported(message) => {
                widgets::centered_message(ui, "⚠", theme::WARNING, message, None);
                return;
            }
            GpuState::Error(_) => {
                if widgets::centered_message(ui, "⚠", theme::ERROR, "GPU operation failed", Some("Retry")) {
                    self.load(cx);
                }
                return;
            }
            GpuState::RequiresReboot(message) | GpuState::RequiresLogout(message) => {
                if widgets::centered_message(ui, "⟲", theme::WARNING, message, Some("Reload")) {
                    self.load(cx);
                }
                return;
            }
            GpuState::Loaded(loaded) => loaded.clone(),
        };

        widgets::card(ui, |ui| {
            widgets::heading(ui, "🖥 GPU Status");
            ui.separator();
            let status = &loaded.status;
            widgets::status_row(ui, "Active Mode", status.active_mode.label());
            widgets::status_row(ui, "dGPU Enabled", if status.dgpu_enabled { "Yes" } else { "No" });
            if let Some(t) = status.temperature {
                widgets::status_row(ui, "Temperature", &format!("{t}°C"));
            }
            if let Some(u) = status.utilization {
                widgets::status_row(ui, "Utilization", &format!("{u}%"));
            }
            widgets::status_row(ui, "Power State", status.power_state.label());
        });
        ui.add_space(16.0);

        let busy = self.set_task.is_some();
        let mut requested = None;
        widgets::card(ui, |ui| {
            widgets::heading(ui, "⚙ GPU Mode");
            ui.separator();
            for mode in GpuMode::ALL.into_iter().filter(|m| loaded.supported_modes.contains(m)) {
                let enabled = loaded.is_selectable(mode) && !busy;
                let selected = loaded.status.active_mode == mode;
                ui.add_enabled_ui(enabled, |ui| {
                    if ui.radio(selected, RichText::new(mode.label()).size(15.0)).clicked() && !selected {
                        requested = Some(mode);
                    }
                    let subtitle = mode_subtitle(mode, &loaded);
                    let color = if mode == GpuMode::AsusEgpu && !loaded.capabilities.external_gpu_connected {
                        theme::WARNING
                    } else {
                        theme::ON_SURFACE_VARIANT
                    };
                    ui.indent(mode.name(), |ui| ui.label(RichText::new(subtitle).small().color(color)));
                });
            }
            if busy {
                ui.spinner();
            }
        });
        if let Some(mode) = requested {
            let (services, store) = (cx.services.clone(), cx.store.clone());
            self.set_task = Some(
                cx.rt
                    .spawn(async move { gpu::set_mode(&*services, &*store, loaded, mode).await }),
            );
        }
    }
}
