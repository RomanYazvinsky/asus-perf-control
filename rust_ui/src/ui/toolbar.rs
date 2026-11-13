//! Top status toolbar, refreshed every second. Two rows per device column:
//!
//! ```text
//! CPU: Ryzen 9 5980HS  63°C   │ GPU1: Radeon Vega Series  58°C │ GPU2: GeForce GTX 1650 Mobile  off │ Profile
//! ⏱ 3.21 GHz  🌀 2300 RPM    │ ⏱ 2.01 GHz  🌀 2300 RPM       │ ⏱ --  🌀 2100 RPM                 │ Balanced
//! ```

use std::time::{Duration, Instant};

use egui::{Color32, RichText, Ui};

use crate::features::telemetry::{self, DeviceColumn, Telemetry, TelemetryService};
use crate::runtime::{take_ready, Task};
use crate::ui::{theme, ScreenCtx};

const TICK: Duration = Duration::from_secs(1);
const HOT_COLOR: Color32 = Color32::from_rgb(0xFF, 0x3B, 0x30);

/// Height of the toolbar panel (two rows of text plus padding).
pub const HEIGHT: f32 = 56.0;

#[derive(Default)]
pub struct Toolbar {
    data: Telemetry,
    task: Option<Task<Telemetry>>,
    last_sample: Option<Instant>,
}

impl Toolbar {
    pub fn poll(&mut self, cx: &mut ScreenCtx) {
        if let Some(data) = take_ready(&mut self.task) {
            self.data = data;
        }
        let due = self.last_sample.is_none_or(|at| at.elapsed() >= TICK);
        if due && self.task.is_none() {
            self.last_sample = Some(Instant::now());
            let services = cx.services.clone();
            self.task = Some(cx.rt.spawn(async move { services.sample_telemetry().await }));
        }
        let since = self.last_sample.map_or(Duration::ZERO, |at| at.elapsed());
        cx.repaint_after(TICK.saturating_sub(since));
    }

    pub fn ui(&self, ui: &mut Ui) {
        let columns = telemetry::device_columns(&self.data);
        ui.horizontal_centered(|ui| {
            for column in &columns {
                device_column(ui, column);
                separator(ui);
            }
            ui.vertical(|ui| {
                ui.spacing_mut().item_spacing.y = 2.0;
                ui.label(RichText::new("Profile").color(theme::ON_SURFACE_VARIANT));
                ui.label(RichText::new(telemetry::format_profile(self.data.performance_mode)).strong());
            });
        });
    }
}

fn device_column(ui: &mut Ui, column: &DeviceColumn) {
    let temp_color = if column.hot { HOT_COLOR } else { theme::ON_SURFACE };
    let response = ui
        .vertical(|ui| {
            ui.spacing_mut().item_spacing.y = 2.0;
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 6.0;
                ui.label(RichText::new(format!("{}:", column.title)).color(theme::ON_SURFACE_VARIANT));
                ui.label(&column.name);
                ui.label(RichText::new(&column.temp).color(temp_color).strong());
            });
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 12.0;
                ui.label(RichText::new(format!("⏱ {}", column.clock)).color(theme::ON_SURFACE_VARIANT));
                ui.label(RichText::new(format!("🌀 {}", column.fan)).color(theme::ON_SURFACE_VARIANT));
            });
        })
        .response;
    response.on_hover_text(&column.hover);
}

fn separator(ui: &mut Ui) {
    ui.add_space(4.0);
    ui.separator();
    ui.add_space(4.0);
}
