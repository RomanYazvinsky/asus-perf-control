//! Battery charge limit (former `BatteryScreen`).

use egui::{RichText, Ui};

use crate::features::battery::{self, BatteryState};
use crate::runtime::{take_ready, Task};
use crate::ui::alert::Alert;
use crate::ui::{theme, widgets, ScreenCtx};

pub struct BatteryScreen {
    state: BatteryState,
    task: Option<Task<BatteryState>>,
    charge_limit: f64,
    has_loaded_data: bool,
    has_shown_unsupported: bool,
}

impl BatteryScreen {
    pub fn new(cx: &mut ScreenCtx) -> Self {
        let mut screen = Self {
            state: BatteryState::Initial,
            task: None,
            charge_limit: 80.0,
            has_loaded_data: false,
            has_shown_unsupported: false,
        };
        screen.load(cx);
        screen
    }

    fn load(&mut self, cx: &mut ScreenCtx) {
        self.state = BatteryState::Loading;
        let (services, store) = (cx.services.clone(), cx.store.clone());
        self.task = Some(
            cx.rt
                .spawn(async move { battery::check_support_and_load(&*services, &*store).await }),
        );
    }

    pub fn poll(&mut self, cx: &mut ScreenCtx) {
        let Some(state) = take_ready(&mut self.task) else {
            return;
        };
        match &state {
            BatteryState::Error(message) => cx.alert(Alert::error(
                "Battery Error",
                if self.has_loaded_data {
                    "Failed to set charge limit"
                } else {
                    "Failed to load battery status"
                },
                message.clone(),
            )),
            BatteryState::Unsupported if !self.has_shown_unsupported => {
                self.has_shown_unsupported = true;
                cx.alert(Alert::info(
                    "Battery Charge Limit Not Supported",
                    "Your hardware or kernel does not support charge limit control.",
                    Some("Requires Linux kernel 5.4+ with charge_control_end_threshold sysfs attribute.".into()),
                ));
            }
            BatteryState::Loaded { charge_limit } => {
                self.charge_limit = f64::from(*charge_limit);
                self.has_loaded_data = true;
            }
            _ => {}
        }
        self.state = state;
    }

    pub fn ui(&mut self, ui: &mut Ui, cx: &mut ScreenCtx) {
        widgets::screen_title(ui, "Battery Charge Limit");
        match &self.state {
            BatteryState::Unsupported => {
                widgets::centered_message(
                    ui,
                    "🔋",
                    theme::WARNING,
                    "Battery charge limit is not supported on this system",
                    None,
                );
                return;
            }
            BatteryState::Error(_) if !self.has_loaded_data => {
                if widgets::centered_message(ui, "⚠", theme::ERROR, "Failed to load battery status", Some("Retry")) {
                    self.load(cx);
                }
                return;
            }
            BatteryState::Loading | BatteryState::Initial if !self.has_loaded_data => return widgets::loading(ui),
            _ => {}
        }

        widgets::card(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.label(RichText::new("ℹ").color(theme::PRIMARY).size(20.0));
                ui.label(
                    "Setting a charge limit helps extend battery lifespan by preventing overcharging. \
                     Recommended: 80% for daily use, 100% when you need full capacity.",
                );
            });
        });
        ui.add_space(16.0);

        widgets::card(ui, |ui| {
            ui.horizontal(|ui| {
                widgets::title(ui, "Charge Limit");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    egui::Frame::new()
                        .fill(theme::PRIMARY_CONTAINER)
                        .corner_radius(16)
                        .inner_margin(egui::Margin::symmetric(12, 6))
                        .show(ui, |ui| {
                            ui.label(RichText::new(format!("{}%", self.charge_limit.round())).strong());
                        });
                });
            });
            widgets::small(ui, "Battery will stop charging at this percentage");
            ui.add_space(8.0);
            ui.spacing_mut().slider_width = (ui.available_width() - 16.0).max(80.0);
            ui.add(egui::Slider::new(&mut self.charge_limit, 50.0..=100.0).step_by(1.0).show_value(false));
            ui.horizontal(|ui| {
                widgets::small(ui, "50%");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| widgets::small(ui, "100%"));
            });
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                for preset in [80.0, 90.0, 100.0] {
                    let selected = self.charge_limit.round() == preset;
                    let button = egui::Button::new(format!("{preset}%")).selected(selected);
                    if ui.add(button).clicked() {
                        self.charge_limit = preset;
                    }
                }
            });
        });
        ui.add_space(16.0);

        let busy = self.task.is_some();
        let label = if busy { "Applying..." } else { "💾 Apply" };
        if widgets::primary_button(ui, label, !busy).clicked() {
            let limit = self.charge_limit.round() as i32;
            self.state = BatteryState::Loading;
            let (services, store) = (cx.services.clone(), cx.store.clone());
            self.task = Some(cx.rt.spawn(async move { battery::set_limit(&*services, &*store, limit).await }));
        }
        if let BatteryState::Loaded { charge_limit } = self.state {
            ui.add_space(12.0);
            ui.vertical_centered(|ui| widgets::muted(ui, format!("Current limit: {charge_limit}%")));
        }
    }
}
