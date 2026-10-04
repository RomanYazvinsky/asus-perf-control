//! XG Mobile fan diagnostic (former `XgMobileFanDiagnosticScreen`).

use std::sync::Arc;

use anyhow::Result;
use egui::{RichText, Ui};

use crate::features::services::AsusdXgPorts;
use crate::features::xg_diag::{
    FanHypothesisResult, SysfsXgFanHwmonProbe, XgFanDiagnosis, XgFanDiagnosticRun, XgFanDiagnosticSession,
    XgFanDiagnosticStore, XgMobileFanDiagnosticService,
};
use crate::runtime::{take_ready, Task};
use crate::ui::{theme, widgets, ScreenCtx};

type Service = XgMobileFanDiagnosticService<AsusdXgPorts, AsusdXgPorts, SysfsXgFanHwmonProbe>;

const CURRENT_STATE: &str = "Current State";
const CHANGED_STATE: &str = "Changed XG Mobile State";

enum Job {
    Sweep { changed: bool, run: Result<XgFanDiagnosticRun> },
    Hypothesis(Result<FanHypothesisResult>),
}

pub struct XgDiagScreen {
    service: Arc<Service>,
    store: XgFanDiagnosticStore,
    session: XgFanDiagnosticSession,
    hypothesis: Option<FanHypothesisResult>,
    error: Option<String>,
    running_label: Option<&'static str>,
    task: Option<Task<Job>>,
}

impl XgDiagScreen {
    pub fn new(cx: &mut ScreenCtx) -> Self {
        let service = XgMobileFanDiagnosticService::new(
            AsusdXgPorts(cx.services.clone()),
            AsusdXgPorts(cx.services.clone()),
            SysfsXgFanHwmonProbe,
        );
        let store = XgFanDiagnosticStore::new(XgFanDiagnosticStore::default_file());
        let (session, error) = match store.load() {
            Ok(session) => (session, None),
            Err(e) => (
                XgFanDiagnosticSession::default(),
                Some(format!("Failed to load saved diagnostic session: {e:#}")),
            ),
        };
        Self {
            service: Arc::new(service),
            store,
            session,
            hypothesis: None,
            error,
            running_label: None,
            task: None,
        }
    }

    pub fn poll(&mut self, _cx: &mut ScreenCtx) {
        let Some(job) = take_ready(&mut self.task) else {
            return;
        };
        self.running_label = None;
        match job {
            Job::Sweep { changed, run: Ok(run) } => {
                if changed {
                    self.session.changed_run = Some(run);
                } else {
                    self.session.current_run = Some(run);
                }
                if let Err(e) = self.store.save(&self.session) {
                    self.error = Some(format!("{e:#}"));
                }
            }
            Job::Hypothesis(Ok(result)) => self.hypothesis = Some(result),
            Job::Sweep { run: Err(e), .. } | Job::Hypothesis(Err(e)) => self.error = Some(format!("{e:#}")),
        }
    }

    fn start_sweep(&mut self, cx: &mut ScreenCtx, changed: bool) {
        let label = if changed { CHANGED_STATE } else { CURRENT_STATE };
        self.running_label = Some(label);
        self.error = None;
        let service = self.service.clone();
        self.task = Some(cx.rt.spawn(async move {
            Job::Sweep {
                changed,
                run: service.run_sweep(label).await,
            }
        }));
    }

    fn start_hypothesis(&mut self, cx: &mut ScreenCtx, label: &'static str) {
        self.running_label = Some(label);
        self.error = None;
        let service = self.service.clone();
        self.task = Some(cx.rt.spawn(async move {
            Job::Hypothesis(match label {
                "CPU/GPU Coupling" => service.run_cpu_gpu_coupling_test().await,
                "Profile Bounds" => service.run_profile_bounds_test().await,
                _ => service.run_curve_validation_test().await,
            })
        }));
    }

    pub fn ui(&mut self, ui: &mut Ui, cx: &mut ScreenCtx) {
        widgets::screen_title(ui, "XG Mobile Diagnostic");
        let running = self.running_label.is_some();

        widgets::card(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                if running {
                    ui.spinner();
                } else {
                    ui.label("ℹ");
                }
                ui.label(match self.running_label {
                    Some(label) => format!(
                        "Running {label}. The app will restore balanced defaults when the sweep finishes."
                    ),
                    None => "Run the current state, reboot after changing XG Mobile state, then return here. \
                             The first run is saved for comparison after restart."
                        .to_string(),
                });
            });
        });
        ui.add_space(8.0);
        if let Some(error) = &self.error {
            widgets::banner(ui, "⚠", error, theme::ERROR_CONTAINER, theme::ON_ERROR_CONTAINER);
            ui.add_space(8.0);
        }

        let mut sweep = None;
        ui.columns(2, |columns| {
            let slots = [
                (CURRENT_STATE, &self.session.current_run, "Run Current State Sweep", false),
                (CHANGED_STATE, &self.session.changed_run, "Run Changed State Sweep", true),
            ];
            for (column, (title, run, button, changed)) in columns.iter_mut().zip(slots) {
                widgets::card(column, |ui| {
                    widgets::title(ui, title);
                    ui.horizontal(|ui| {
                        if widgets::primary_button(ui, &format!("🔬 {button}"), !running).clicked() {
                            sweep = Some(changed);
                        }
                        if self.running_label == Some(title) {
                            ui.spinner();
                        }
                    });
                    ui.add_space(8.0);
                    match run {
                        None => {
                            ui.label("No run captured");
                        }
                        Some(run) => run_summary(ui, run),
                    }
                });
            }
        });
        if let Some(changed) = sweep {
            self.start_sweep(cx, changed);
        }
        ui.add_space(8.0);

        let mut hypothesis = None;
        widgets::card(ui, |ui| {
            widgets::title(ui, "Fan Hypotheses");
            ui.horizontal_wrapped(|ui| {
                for (icon, label) in [("⇄", "CPU/GPU Coupling"), ("⏩", "Profile Bounds"), ("📏", "Curve Rule Discovery")] {
                    if widgets::primary_button(ui, &format!("{icon} Run {label}"), !running).clicked() {
                        hypothesis = Some(label);
                    }
                }
            });
            if let Some(result) = &self.hypothesis {
                ui.add_space(8.0);
                ui.label(RichText::new(&result.name).strong());
                ui.label(result.diagnosis.kind.title());
                ui.label(&result.diagnosis.summary);
                ui.label(&result.diagnosis.next_action);
                ui.add_space(4.0);
                for step in &result.steps {
                    ui.add(egui::Label::new(RichText::new(step.describe()).monospace()).selectable(true));
                }
            }
        });
        if let Some(label) = hypothesis {
            self.start_hypothesis(cx, label);
        }

        if let Some(diagnosis) = XgFanDiagnosis::for_session(&self.session) {
            ui.add_space(8.0);
            widgets::card(ui, |ui| {
                widgets::title(ui, diagnosis.kind.title());
                ui.label(diagnosis.summary);
                ui.label(diagnosis.next_action);
            });
        }
    }
}

fn rpm(value: Option<i64>) -> String {
    value.map_or("n/a".into(), |v| v.to_string())
}

fn run_summary(ui: &mut Ui, run: &XgFanDiagnosticRun) {
    ui.label(format!("Baseline fan2: {}", rpm(run.baseline_snapshot.fan2_rpm)));
    egui::Grid::new(("xg_run", &run.label)).num_columns(5).show(ui, |ui| {
        for header in ["PWM", "Min", "Max", "Last", "Result"] {
            ui.label(RichText::new(header).strong());
        }
        ui.end_row();
        for step in &run.steps {
            ui.label(step.requested_pwm.to_string());
            ui.label(rpm(step.fan2_min_rpm()));
            ui.label(rpm(step.fan2_max_rpm()));
            ui.label(rpm(step.fan2_last_rpm()));
            ui.label(step.classification().label());
            ui.end_row();
        }
    });
}
