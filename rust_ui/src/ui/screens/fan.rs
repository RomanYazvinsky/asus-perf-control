//! Fan control (former `FanScreen`, `FanContent`, `FanTabContent`).

use std::collections::BTreeMap;
use std::time::{Duration, Instant};

use anyhow::Result;
use egui::{RichText, Ui};

use crate::features::fan::{self, FanControlMode, FanId, FanLoaded, FanRefresh, FanState, PerfProfile};
use crate::input::UiInputCommand;
use crate::runtime::{take_ready, Task};
use crate::ui::alert::Alert;
use crate::ui::curve_editor::CurveEditor;
use crate::ui::{theme, widgets, ScreenCtx};

const REFRESH_INTERVAL: Duration = Duration::from_secs(2);
const RESULT_AUTO_RESTORE: Duration = Duration::from_secs(3);

type MaxSpeedResult = Result<FanState, Box<(String, FanLoaded)>>;

pub struct FanScreen {
    state: FanState,
    task: Option<Task<FanState>>,
    max_speed_task: Option<Task<MaxSpeedResult>>,
    refresh_task: Option<Task<Result<FanRefresh>>>,
    last_refresh: Instant,
    result_shown_at: Option<Instant>,
    editors: BTreeMap<FanId, CurveEditor>,
}

/// What the user asked for during this frame.
enum Action {
    ToggleEngine(bool),
    SelectProfile(PerfProfile),
    ApplyCurves(Vec<(FanId, Vec<fan::FanCurvePoint>)>),
    Reset,
    MaxSpeed(FanId),
}

impl FanScreen {
    pub fn new(cx: &mut ScreenCtx) -> Self {
        let mut screen = Self {
            state: FanState::Initial,
            task: None,
            max_speed_task: None,
            refresh_task: None,
            last_refresh: Instant::now(),
            result_shown_at: None,
            editors: BTreeMap::new(),
        };
        screen.load(cx);
        screen
    }

    fn load(&mut self, cx: &mut ScreenCtx) {
        self.state = FanState::Loading;
        self.result_shown_at = None;
        let (services, store) = (cx.services.clone(), cx.store.clone());
        self.task = Some(cx.rt.spawn(async move { fan::load(&*services, &*store).await }));
    }

    fn set_state(&mut self, state: FanState, cx: &mut ScreenCtx) {
        if let FanState::Error(message) = &state {
            cx.alert(Alert::error(
                "Fan Control Error",
                "An error occurred while controlling fans",
                message.clone(),
            ));
        }
        if matches!(state, FanState::MaxSpeedResult { .. }) {
            self.result_shown_at = Some(Instant::now());
        }
        self.state = state;
    }

    pub fn poll(&mut self, cx: &mut ScreenCtx) {
        if let Some(state) = take_ready(&mut self.task) {
            self.last_refresh = Instant::now();
            self.set_state(state, cx);
        }
        if let Some(result) = take_ready(&mut self.max_speed_task) {
            match result {
                Ok(state) => self.set_state(state, cx),
                Err(failed) => {
                    let (message, previous) = *failed;
                    cx.alert(Alert::error(
                        "Fan Control Error",
                        "An error occurred while controlling fans",
                        message,
                    ));
                    self.state = FanState::Loaded(previous);
                }
            }
        }
        // Refresh failures are ignored, like the former periodic poll.
        if let Some(Ok(refresh)) = take_ready(&mut self.refresh_task) {
            if let FanState::Loaded(loaded) = &mut self.state {
                fan::apply_refresh(loaded, refresh);
            }
        }

        if let (FanState::MaxSpeedResult { previous, .. }, Some(shown)) = (&self.state, self.result_shown_at) {
            if shown.elapsed() >= RESULT_AUTO_RESTORE {
                self.state = FanState::Loaded(previous.clone());
                self.result_shown_at = None;
            } else {
                cx.repaint_after(RESULT_AUTO_RESTORE - shown.elapsed());
            }
        }

        if matches!(self.state, FanState::Loaded(_)) && self.task.is_none() && self.refresh_task.is_none() {
            let elapsed = self.last_refresh.elapsed();
            if elapsed >= REFRESH_INTERVAL {
                self.last_refresh = Instant::now();
                let services = cx.services.clone();
                self.refresh_task = Some(cx.rt.spawn(async move { fan::refresh(&*services).await }));
            } else {
                cx.repaint_after(REFRESH_INTERVAL - elapsed);
            }
        }
    }

    pub fn ui(&mut self, ui: &mut Ui, cx: &mut ScreenCtx, curve_commands: &[UiInputCommand]) {
        widgets::screen_title(ui, "Fan Control");
        let loaded = match &self.state {
            FanState::Loading | FanState::Initial => return widgets::loading(ui),
            FanState::Unsupported(message) => {
                widgets::centered_message(ui, "⚠", theme::WARNING, message, None);
                return;
            }
            FanState::Error(_) => {
                if widgets::centered_message(ui, "⚠", theme::ERROR, "Fan control failed", Some("Retry")) {
                    self.load(cx);
                }
                return;
            }
            FanState::MaxSpeedTesting { .. } => {
                ui.vertical_centered(|ui| {
                    ui.add_space(ui.available_height() / 3.0);
                    ui.spinner();
                    ui.add_space(16.0);
                    widgets::heading(ui, "Max Speed Test in Progress");
                    ui.label("Current RPM: 0");
                    ui.label("Elapsed: 0s");
                });
                return;
            }
            FanState::MaxSpeedResult { fan, max_rpm, .. } => {
                let (fan, max_rpm) = (*fan, *max_rpm);
                if max_speed_result(ui, fan, max_rpm) {
                    self.load(cx);
                }
                return;
            }
            FanState::Loaded(loaded) => loaded.clone(),
        };

        if let Some(action) = self.content(ui, &loaded, curve_commands) {
            self.run(action, loaded, cx);
        }
    }

    fn content(&mut self, ui: &mut Ui, state: &FanLoaded, curve_commands: &[UiInputCommand]) -> Option<Action> {
        let busy = self.task.is_some() || self.max_speed_task.is_some();
        let mut action = None;

        ui.columns(2, |columns| {
            for (column, (label, temp)) in columns.iter_mut().zip([("CPU", state.cpu_temp_c), ("GPU", state.gpu_temp_c)]) {
                column.vertical_centered(|ui| {
                    ui.label(RichText::new(label).small());
                    ui.label(RichText::new(format!("{temp}°C")).size(20.0).strong());
                });
            }
        });
        ui.add_space(8.0);
        ui.horizontal(|ui| {
            ui.label("Dynamic Engine");
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let mut running = state.dynamic_engine_running;
                ui.add_enabled_ui(!busy, |ui| {
                    if widgets::toggle(ui, &mut running).changed() {
                        action = Some(Action::ToggleEngine(running));
                    }
                });
                if busy {
                    ui.spinner();
                }
            });
        });
        if let Some(mode) = widgets::profile_tabs(ui, state.selected_profile.to_mode()) {
            action = Some(Action::SelectProfile(PerfProfile::from_mode(mode)));
        }
        widgets::editing_profile_hint(
            ui,
            state.selected_profile.to_mode(),
            state.active_profile.map(PerfProfile::to_mode),
        );

        ui.add_space(8.0);
        // Fans side by side so both curves are visible at once.
        let fan_ids = state.fan_ids();
        ui.columns(fan_ids.len().max(1), |columns| {
            for (column, fan_id) in columns.iter_mut().zip(fan_ids) {
                if let Some(fan_action) = self.fan_section(column, state, fan_id, busy, curve_commands) {
                    action = Some(fan_action);
                }
            }
        });
        action
    }

    /// Whether the sticky action bar is shown.
    pub fn has_actions(&self) -> bool {
        matches!(self.state, FanState::Loaded(_))
    }

    /// Apply / reset for the edited profile, drawn in the sticky bar below the scroll area.
    pub fn actions(&mut self, ui: &mut Ui, cx: &mut ScreenCtx) {
        let FanState::Loaded(state) = &self.state else {
            return;
        };
        let state = state.clone();
        let busy = self.task.is_some() || self.max_speed_task.is_some();
        let edits: Vec<_> = state
            .fan_ids()
            .into_iter()
            .filter_map(|id| self.editors.get(&id)?.edited_points().map(|points| (id, points)))
            .collect();
        let mut action = None;
        ui.horizontal(|ui| {
            if widgets::primary_button(ui, "✔ Apply", !busy && !edits.is_empty()).clicked() {
                action = Some(Action::ApplyCurves(edits.clone()));
            }
            if widgets::outlined_button(ui, "⟲ Reset to Default", !busy).clicked() {
                action = Some(Action::Reset);
            }
            if busy {
                ui.spinner();
            } else if !edits.is_empty() {
                widgets::muted(ui, "Unsaved curve changes");
            }
        });
        if let Some(action) = action {
            self.run(action, state, cx);
        }
    }

    fn fan_section(
        &mut self,
        ui: &mut Ui,
        state: &FanLoaded,
        fan_id: FanId,
        busy: bool,
        curve_commands: &[UiInputCommand],
    ) -> Option<Action> {
        let mode = state.control_mode(fan_id);
        let rpm = state.fan_rpms.get(&fan_id).copied().unwrap_or(0);
        let mut action = None;

        widgets::card(ui, |ui| {
            widgets::heading(ui, fan_id.label());
            ui.label(RichText::new(format!("🌀 {rpm} RPM")).size(20.0));
            ui.label(format!("Mode: {}", mode.label()));
        });
        ui.add_space(8.0);

        match mode {
            FanControlMode::ConstantSpeed => {
                widgets::card(ui, |ui| {
                    widgets::title(ui, "Constant Speed");
                    ui.label("Constant speed control is not available in this build.");
                });
            }
            FanControlMode::AsusctlCurve | FanControlMode::DynamicEngine => {
                let curve = state.editor_curve(fan_id);
                // Each fan follows its own sensor; 0 means the reading failed.
                let temp_c = match fan_id {
                    FanId::Cpu => state.cpu_temp_c,
                    FanId::Gpu => state.gpu_temp_c,
                };
                let current_temp = (temp_c > 0).then_some(f64::from(temp_c));
                let editor = self.editors.entry(fan_id).or_default();
                editor.show(ui, &curve, false, current_temp, curve_commands);
                if mode == FanControlMode::DynamicEngine {
                    widgets::small(ui, "Apply sends this curve to the running Dynamic Engine.");
                }
            }
        }
        ui.add_space(8.0);
        if ui.add_enabled(!busy, egui::Button::new("⏩ Test Max Speed")).clicked() {
            action = Some(Action::MaxSpeed(fan_id));
        }
        action
    }

    fn run(&mut self, action: Action, state: FanLoaded, cx: &mut ScreenCtx) {
        let (services, store) = (cx.services.clone(), cx.store.clone());
        let job: std::pin::Pin<Box<dyn std::future::Future<Output = FanState> + Send>> = match action {
            Action::ToggleEngine(true) => {
                let config = state.build_engine_config();
                Box::pin(async move { fan::start_dynamic_engine(&*services, &*store, state, config).await })
            }
            Action::ToggleEngine(false) => Box::pin(async move { fan::stop_dynamic_engine(&*services, &*store, state).await }),
            Action::SelectProfile(profile) => {
                Box::pin(async move { fan::select_profile(&*store, state, profile) })
            }
            Action::ApplyCurves(curves) => {
                Box::pin(async move { fan::apply_curves(&*services, &*store, state, curves).await })
            }
            Action::Reset => {
                let profile = state.selected_profile;
                Box::pin(async move { fan::reset_to_defaults(&*services, &*store, state, profile).await })
            }
            Action::MaxSpeed(fan_id) => {
                self.state = FanState::MaxSpeedTesting { fan: fan_id };
                self.max_speed_task = Some(cx.rt.spawn(async move { fan::max_speed_test(&*services, state, fan_id).await }));
                return;
            }
        };
        self.task = Some(cx.rt.spawn(job));
    }
}

/// Returns true when OK is pressed.
fn max_speed_result(ui: &mut Ui, fan: FanId, max_rpm: u32) -> bool {
    let mut ok = false;
    ui.vertical_centered(|ui| {
        ui.add_space(ui.available_height() / 4.0);
        ui.set_max_width(360.0);
        widgets::card(ui, |ui| {
            ui.vertical_centered(|ui| {
                ui.label(RichText::new("✔").size(56.0).color(theme::SUCCESS));
                widgets::heading(ui, "Max Speed Test Complete");
                ui.add_space(8.0);
                ui.label(format!("{} Fan Max RPM", fan.asusd_name()));
                ui.label(RichText::new(format!("{max_rpm} RPM")).size(26.0));
                ui.add_space(12.0);
                ok = ui.button("OK").clicked();
            });
        });
    });
    ok
}
