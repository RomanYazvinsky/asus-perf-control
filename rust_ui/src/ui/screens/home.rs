//! Performance mode and CPU boost (former `HomeScreen`).

use egui::{RichText, Ui};

use crate::features::cpu_boost::{self, CpuBoostState, BOOST_PRESETS};
use crate::features::performance::{self, mode_name, PerformanceState};
use crate::runtime::{take_ready, Task};
use crate::ui::alert::Alert;
use crate::ui::{theme, widgets, ScreenCtx};

pub struct HomeScreen {
    perf: PerformanceState,
    perf_task: Option<Task<PerformanceState>>,
    boost: CpuBoostState,
    boost_task: Option<Task<CpuBoostState>>,
}

const MODES: [(i32, &str, &str, &str); 3] = [
    (0, "🔇", "Silent", "Quiet operation, lower performance"),
    (1, "⚖", "Balanced", "Optimal balance of performance and noise"),
    (2, "⏩", "Turbo", "Maximum performance, higher noise"),
];

impl HomeScreen {
    pub fn new(cx: &mut ScreenCtx) -> Self {
        let (services, store) = (cx.services.clone(), cx.store.clone());
        let perf_task = cx
            .rt
            .spawn(async move { performance::load_current_mode(&*services, &*store).await });
        let (services, store) = (cx.services.clone(), cx.store.clone());
        let boost_task = cx
            .rt
            .spawn(async move { cpu_boost::load_status(&*services, &*store).await });
        Self {
            perf: PerformanceState::Loading,
            perf_task: Some(perf_task),
            boost: CpuBoostState::Loading,
            boost_task: Some(boost_task),
        }
    }

    pub fn poll(&mut self, cx: &mut ScreenCtx) {
        if let Some(state) = take_ready(&mut self.perf_task) {
            if let PerformanceState::Error(message) = &state {
                cx.alert(Alert::error(
                    "Ошибка производительности",
                    "Не удалось выполнить операцию с режимом производительности",
                    message.clone(),
                ));
            }
            self.perf = state;
        }
        if let Some(state) = take_ready(&mut self.boost_task) {
            if let CpuBoostState::Error(message) = &state {
                cx.alert(Alert::error(
                    "Ошибка CPU Boost",
                    "Не удалось выполнить операцию с CPU Boost",
                    message.clone(),
                ));
            }
            self.boost = state;
        }
    }

    pub fn ui(&mut self, ui: &mut Ui, cx: &mut ScreenCtx) {
        widgets::screen_title(ui, "ASUS Linux Control");
        ui.label(RichText::new("ASUS Linux Control Tool").size(26.0));
        widgets::title(ui, "MVP: Performance Mode Control");
        ui.add_space(16.0);

        widgets::card(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new("ℹ").size(30.0));
                ui.add_space(8.0);
                ui.vertical(|ui| {
                    ui.label(RichText::new("Current Mode").strong());
                    match &self.perf {
                        PerformanceState::Loading => {
                            ui.label("Loading...");
                        }
                        PerformanceState::Loaded(mode) => {
                            ui.label(RichText::new(mode_name(*mode)).size(22.0).color(theme::PRIMARY));
                        }
                        PerformanceState::Error(_) => {
                            ui.label(RichText::new("Error").color(theme::ERROR));
                        }
                        PerformanceState::Initial => {
                            ui.label("Unknown");
                        }
                    }
                });
            });
        });
        ui.add_space(16.0);

        let current = match self.perf {
            PerformanceState::Loaded(mode) => mode,
            _ => 1,
        };
        let busy = self.perf_task.is_some();
        let mut requested = None;
        widgets::card(ui, |ui| {
            widgets::heading(ui, "Performance Mode");
            ui.add_space(8.0);
            for (mode, icon, title, subtitle) in MODES {
                if widgets::choice_button(ui, icon, title, subtitle, current == mode, !busy).clicked() {
                    requested = Some(mode);
                }
            }
        });
        if let Some(mode) = requested {
            self.perf = PerformanceState::Loading;
            let (services, store) = (cx.services.clone(), cx.store.clone());
            self.perf_task = Some(
                cx.rt
                    .spawn(async move { performance::set_mode(&*services, &*store, mode).await }),
            );
        }
        ui.add_space(16.0);
        self.boost_ui(ui, cx);
    }

    fn boost_ui(&mut self, ui: &mut Ui, cx: &mut ScreenCtx) {
        let mut requested = None;
        widgets::card(ui, |ui| {
            widgets::heading(ui, "CPU Boost Mode");
            ui.add_space(8.0);
            match &self.boost {
                CpuBoostState::Loading | CpuBoostState::Initial => {
                    ui.vertical_centered(|ui| ui.spinner());
                }
                CpuBoostState::Unsupported => {
                    ui.label("CPU boost controls are unavailable");
                }
                CpuBoostState::Loaded(status) if !status.is_supported => {
                    ui.label("CPU boost controls are unavailable");
                }
                state => {
                    let status = match state {
                        CpuBoostState::Loaded(status) => Some(status),
                        _ => None,
                    };
                    let available = status.map_or(cpu_boost::CpuBoostMode::ALL.to_vec(), |s| s.available_modes.clone());
                    for preset in &BOOST_PRESETS {
                        let Some(mode) = preset.resolve_mode(&available) else {
                            continue;
                        };
                        let selected = status.is_some_and(|s| preset.includes(s.current_mode));
                        if widgets::choice_button(ui, preset.icon, preset.label, preset.description, selected, true).clicked() {
                            requested = Some(mode);
                        }
                    }
                }
            }
        });
        if let Some(mode) = requested {
            self.boost = CpuBoostState::Loading;
            let (services, store) = (cx.services.clone(), cx.store.clone());
            self.boost_task = Some(
                cx.rt
                    .spawn(async move { cpu_boost::set_mode(&*services, &*store, mode).await }),
            );
        }
    }
}
