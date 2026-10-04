//! CPU power and temperature limits (former `TdpScreen` + `TdpControls`).

use egui::{RichText, Ui};

use crate::features::tdp::{self, SetTdpRequest, TdpLoaded, TdpState};
use crate::runtime::{take_ready, Task};
use crate::ui::alert::Alert;
use crate::ui::{theme, widgets, ScreenCtx};

pub struct TdpScreen {
    state: TdpState,
    task: Option<Task<TdpState>>,
    // Local (unapplied) edits.
    sustained_w: f64,
    boost_w: f64,
    temp_limit_c: f64,
    min_w: f64,
    max_w: f64,
    min_boost_w: f64,
    max_boost_w: f64,
    has_loaded_data: bool,
    power_limits_enabled: bool,
    temp_limit_enabled: bool,
}

fn profile_label(mode: i32) -> &'static str {
    match mode {
        0 => "Silent",
        2 => "Turbo",
        _ => "Balanced",
    }
}

impl TdpScreen {
    pub fn new(cx: &mut ScreenCtx) -> Self {
        let mut screen = Self {
            state: TdpState::Initial,
            task: None,
            sustained_w: 0.0,
            boost_w: 0.0,
            temp_limit_c: 90.0,
            min_w: 5.0,
            max_w: 54.0,
            min_boost_w: 5.0,
            max_boost_w: 54.0,
            has_loaded_data: false,
            power_limits_enabled: false,
            temp_limit_enabled: false,
        };
        screen.load(cx);
        screen
    }

    fn load(&mut self, cx: &mut ScreenCtx) {
        let (services, store) = (cx.services.clone(), cx.store.clone());
        self.start(cx, async move { tdp::load(&*services, &*store).await });
    }

    fn start(&mut self, cx: &mut ScreenCtx, job: impl std::future::Future<Output = TdpState> + Send + 'static) {
        self.state = TdpState::Loading;
        self.task = Some(cx.rt.spawn(job));
    }

    fn loaded(&self) -> Option<TdpLoaded> {
        match &self.state {
            TdpState::Loaded(loaded) => Some(loaded.clone()),
            _ => None,
        }
    }

    pub fn poll(&mut self, cx: &mut ScreenCtx) {
        let Some(state) = take_ready(&mut self.task) else {
            return;
        };
        match &state {
            TdpState::Error(message) => cx.alert(Alert::error(
                "TDP Error",
                if self.has_loaded_data {
                    "Failed to apply TDP settings"
                } else {
                    "Failed to load TDP config"
                },
                message.clone(),
            )),
            TdpState::Loaded(loaded) => self.take_values(loaded),
            _ => {}
        }
        self.state = state;
    }

    /// Hardware reports 0 when TDP was never configured; fall back to the range minimum.
    fn take_values(&mut self, loaded: &TdpLoaded) {
        self.min_w = f64::from(loaded.min_w);
        self.max_w = f64::from(loaded.max_w);
        self.min_boost_w = f64::from(loaded.min_boost_w);
        self.max_boost_w = f64::from(loaded.max_boost_w);
        self.sustained_w = if loaded.sustained_w > 0 {
            f64::from(loaded.sustained_w).clamp(self.min_w, self.max_w.max(self.min_w))
        } else {
            self.min_w
        };
        self.boost_w = if loaded.boost_w > 0 {
            f64::from(loaded.boost_w).clamp(self.min_boost_w, self.max_boost_w.max(self.min_boost_w))
        } else {
            self.min_boost_w
        };
        if loaded.temp_limit_c > 0 {
            self.temp_limit_c = f64::from(loaded.temp_limit_c).clamp(60.0, 100.0);
        }
        self.has_loaded_data = true;
        self.power_limits_enabled = loaded.power_limits_enabled;
        self.temp_limit_enabled = loaded.temp_limit_enabled;
    }

    pub fn ui(&mut self, ui: &mut Ui, cx: &mut ScreenCtx) {
        widgets::screen_title(ui, "CPU Power & Temperature Limits");
        match &self.state {
            TdpState::Unsupported => return unsupported_card(ui),
            TdpState::Error(_) if !self.has_loaded_data => {
                if widgets::centered_message(ui, "⚠", theme::ERROR, "Failed to load TDP config", Some("Retry")) {
                    self.load(cx);
                }
                return;
            }
            TdpState::Loading | TdpState::Initial => return widgets::loading(ui),
            _ if !self.has_loaded_data => return widgets::loading(ui),
            _ => {}
        }
        self.controls(ui, cx);
    }

    fn controls(&mut self, ui: &mut Ui, cx: &mut ScreenCtx) {
        let active_mode = self.loaded().map_or(1, |l| l.active_performance_mode);
        let busy = self.task.is_some();
        let can_apply = self.power_limits_enabled || self.temp_limit_enabled;

        if let Some(mode) = widgets::profile_tabs(ui, active_mode) {
            let (services, store) = (cx.services.clone(), cx.store.clone());
            self.start(cx, async move { tdp::select_profile(&*services, &*store, mode).await });
            return;
        }
        ui.add_space(8.0);
        ui.label(RichText::new("CPU Power Limits").size(24.0));
        ui.label("Configure sustained and boost TDP limits. Values are persistent — asusd saves them to its profile.");
        ui.add_space(8.0);

        let power_subtitle = if self.power_limits_enabled {
            "Custom sustained and boost limits are active"
        } else {
            "Power limits will not be changed"
        };
        if widgets::switch_card(ui, "Enable Power Limits", power_subtitle, &mut self.power_limits_enabled)
            && !self.power_limits_enabled
        {
            let previous = self.loaded();
            let (services, store) = (cx.services.clone(), cx.store.clone());
            self.start(cx, async move { tdp::clear_power_limits(&*services, &*store, previous).await });
        }
        ui.add_space(8.0);
        widgets::slider_card(
            ui,
            "Sustained TDP (PL1/SPL)",
            &mut self.sustained_w,
            self.min_w,
            self.max_w,
            "W",
            "Long-term power limit under sustained load",
            self.power_limits_enabled,
        );
        // Boost exists only when asusd exposes ppt_pl2_sppt.
        if self.max_boost_w > 0.0 {
            ui.add_space(8.0);
            self.boost_w = self.boost_w.clamp(self.min_boost_w, self.max_boost_w.max(self.min_boost_w));
            widgets::slider_card(
                ui,
                "Boost TDP (PL2/SPPT)",
                &mut self.boost_w,
                self.min_boost_w,
                self.max_boost_w,
                "W",
                "Short-burst power limit (PL2/SPPT)",
                self.power_limits_enabled,
            );
        }

        ui.add_space(16.0);
        ui.label(RichText::new("CPU Temperature Limit").size(24.0));
        ui.horizontal(|ui| {
            ui.label("Active profile");
            widgets::chip(ui, "⏱", profile_label(active_mode));
        });
        ui.label("Configure the CPU temperature ceiling independently from power limits.");
        ui.add_space(8.0);
        let temp_subtitle = if self.temp_limit_enabled {
            "Custom CPU temperature ceiling is active"
        } else {
            "Temperature limit will not be changed"
        };
        widgets::switch_card(ui, "Enable CPU Temperature Limit", temp_subtitle, &mut self.temp_limit_enabled);
        ui.add_space(8.0);
        widgets::slider_card(
            ui,
            "CPU Temperature Limit",
            &mut self.temp_limit_c,
            60.0,
            100.0,
            "°C",
            "CPU throttles when this temperature is reached",
            self.temp_limit_enabled,
        );
        widgets::small(ui, "Persists across reboots (saved to disk)");
        ui.add_space(16.0);

        ui.horizontal(|ui| {
            if widgets::primary_button(ui, "✔ Apply", !busy && can_apply).clicked() {
                let request = SetTdpRequest {
                    apply_power_limits: self.power_limits_enabled,
                    apply_temp_limit: self.temp_limit_enabled,
                    sustained_w: self.sustained_w.round() as i32,
                    boost_w: self.boost_w.round() as i32,
                    temp_limit_c: self.temp_limit_c.round() as i32,
                };
                let previous = self.loaded();
                let (services, store) = (cx.services.clone(), cx.store.clone());
                self.start(cx, async move { tdp::set_tdp(&*services, &*store, request, previous).await });
            }
            if widgets::outlined_button(ui, "⟲ Reload from Hardware", !busy).clicked() {
                let services = cx.services.clone();
                self.start(cx, async move { tdp::reload(&*services).await });
            }
            if busy {
                ui.spinner();
            }
        });

        if !can_apply {
            ui.add_space(12.0);
            widgets::banner(
                ui,
                "ℹ",
                "Enable at least one limit to apply changes",
                theme::SECONDARY_CONTAINER,
                theme::ON_SECONDARY_CONTAINER,
            );
        }
    }
}

fn unsupported_card(ui: &mut Ui) {
    ui.vertical_centered(|ui| {
        ui.add_space(24.0);
        widgets::card(ui, |ui| {
            ui.vertical_centered(|ui| {
                ui.label(RichText::new("🔧").size(56.0).color(theme::ON_SURFACE_VARIANT));
                ui.add_space(8.0);
                widgets::heading(ui, "TDP Control Not Available");
                ui.label(
                    "CPU TDP control requires Linux kernel 5.17+ with the asus-nb-wmi module and an ASUS laptop with PPT support.",
                );
                ui.add_space(8.0);
                ui.label(
                    RichText::new("Required sysfs path:\n/sys/devices/platform/asus-nb-wmi/ppt_pl1_spl")
                        .monospace()
                        .color(theme::ON_SURFACE_VARIANT),
                );
            });
        });
    });
}
