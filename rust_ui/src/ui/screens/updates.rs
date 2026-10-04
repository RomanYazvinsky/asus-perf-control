//! Native package updates (former `PackageUpdatesScreen`).

use std::time::{Duration, Instant};

use egui::{RichText, Ui};

use crate::features::updates::{self, PackageUpdateRunState, PackageUpdateState};
use crate::runtime::{take_ready, Task};
use crate::ui::{widgets, ScreenCtx};

const POLL_INTERVAL: Duration = Duration::from_secs(2);

pub struct UpdatesScreen {
    state: PackageUpdateState,
    task: Option<Task<PackageUpdateState>>,
    last_poll: Instant,
}

fn status_icon(state: &PackageUpdateState) -> &'static str {
    if state.daemon_unavailable {
        return "☁";
    }
    match state.status.as_ref().map(|s| s.state) {
        Some(PackageUpdateRunState::Available) => "⬇",
        Some(PackageUpdateRunState::Running) => "⟳",
        Some(PackageUpdateRunState::Succeeded) => "✔",
        Some(PackageUpdateRunState::Failed | PackageUpdateRunState::Unsupported) => "⚠",
        Some(PackageUpdateRunState::UpToDate) => "✅",
        _ => "ℹ",
    }
}

impl UpdatesScreen {
    pub fn new(cx: &mut ScreenCtx) -> Self {
        let services = cx.services.clone();
        Self {
            state: PackageUpdateState {
                is_checking: true,
                ..Default::default()
            },
            task: Some(cx.rt.spawn(async move { updates::load(&*services).await })),
            last_poll: Instant::now(),
        }
    }

    pub fn poll(&mut self, cx: &mut ScreenCtx) {
        if let Some(state) = take_ready(&mut self.task) {
            self.state = state;
            self.last_poll = Instant::now();
        }
        // Follow a running update until it finishes.
        if self.task.is_none() && self.state.is_update_running() && self.state.update_id.is_some() {
            let elapsed = self.last_poll.elapsed();
            if elapsed >= POLL_INTERVAL {
                let (services, current) = (cx.services.clone(), self.state.clone());
                self.task = Some(cx.rt.spawn(async move { updates::poll(&*services, current).await }));
            } else {
                cx.repaint_after(POLL_INTERVAL - elapsed);
            }
        }
    }

    pub fn ui(&mut self, ui: &mut Ui, cx: &mut ScreenCtx) {
        widgets::screen_title(ui, "Package Updates");
        let state = &self.state;
        widgets::card(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new(status_icon(state)).size(20.0));
                widgets::title(ui, &state.summary());
            });
            if state.is_checking {
                ui.add(egui::ProgressBar::new(0.0).animate(true).desired_height(4.0));
            }
            ui.add_space(8.0);
            ui.label("Backend");
            ui.label(state.backend_label());
            if let Some(status) = state.status.as_ref().filter(|s| !s.log_tail.is_empty()) {
                ui.add_space(8.0);
                for line in &status.log_tail {
                    ui.add(egui::Label::new(RichText::new(line).monospace()).selectable(true));
                }
            }
        });
        ui.add_space(16.0);
        widgets::card(ui, |ui| {
            widgets::title(ui, "Versions");
            ui.add_space(8.0);
            let status = state.status.as_ref();
            egui::Grid::new("versions").num_columns(2).min_col_width(96.0).show(ui, |ui| {
                ui.label("Installed");
                ui.label(status.and_then(|s| s.installed_version.as_deref()).unwrap_or("unknown"));
                ui.end_row();
                ui.label("Available");
                ui.label(status.and_then(|s| s.available_version.as_deref()).unwrap_or("none"));
                ui.end_row();
            });
        });
        ui.add_space(16.0);

        let busy = self.task.is_some();
        ui.horizontal_wrapped(|ui| {
            if widgets::outlined_button(ui, "⟳ Refresh", !self.state.is_checking && !busy).clicked() {
                self.state.is_checking = true;
                self.state.daemon_unavailable = false;
                let (services, current) = (cx.services.clone(), self.state.clone());
                self.task = Some(cx.rt.spawn(async move { updates::check(&*services, current).await }));
            }
            if widgets::primary_button(ui, "⬇ Install Update", self.state.can_start_update() && !busy).clicked() {
                let (services, current) = (cx.services.clone(), self.state.clone());
                self.task = Some(cx.rt.spawn(async move { updates::start(&*services, current).await }));
            }
            if self.state.is_update_running() && widgets::outlined_button(ui, "✖ Cancel", !busy).clicked() {
                let (services, current) = (cx.services.clone(), self.state.clone());
                self.task = Some(cx.rt.spawn(async move { updates::cancel(&*services, current).await }));
            }
        });
    }
}
