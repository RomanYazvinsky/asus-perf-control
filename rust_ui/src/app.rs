//! Application shell: navigation rail, screens, alerts, single-instance activation.

use std::sync::Arc;

use egui::{Align, Align2, FontId, Layout, Vec2};

use crate::features::services::Services;
use crate::input::{AppCommands, InputBridge};
use crate::logging::log_event;
use crate::runtime::Runtime;
use crate::settings::{update_settings, AppSettingsStore, FileAppSettingsStore, FileHardwareProfileStore, ProfiledAppSettingsStore};
use crate::single_instance::PrimaryInstance;
use crate::ui::alert::{show_alerts, Alert};
use crate::ui::screens::{
    battery::BatteryScreen, fan::FanScreen, gpu::GpuScreen, home::HomeScreen, profiles::ProfilesScreen, tdp::TdpScreen,
    updates::UpdatesScreen, xg_diag::XgDiagScreen,
};
use crate::ui::{theme, widgets, AppStore, ScreenCtx};

const DESTINATIONS: [(&str, &str); 8] = [
    ("⏱", "Performance"),
    ("🔌", "TDP"),
    ("🖥", "GPU"),
    ("🌀", "Fans"),
    ("🔋", "Battery"),
    ("🔗", "XG Diag"),
    ("⬇", "Updates"),
    ("👤", "Profiles"),
];

/// Screens that read hardware through the active profile; rebuilt when a profile is applied.
struct HardwareScreens {
    home: HomeScreen,
    tdp: TdpScreen,
    gpu: GpuScreen,
    fan: FanScreen,
    battery: BatteryScreen,
}

impl HardwareScreens {
    fn new(cx: &mut ScreenCtx) -> Self {
        Self {
            home: HomeScreen::new(cx),
            tdp: TdpScreen::new(cx),
            gpu: GpuScreen::new(cx),
            fan: FanScreen::new(cx),
            battery: BatteryScreen::new(cx),
        }
    }
}

pub struct AsusApp {
    rt: Runtime,
    services: Arc<Services>,
    store: Arc<AppStore>,
    alerts: Vec<Alert>,
    selected: usize,
    hardware: HardwareScreens,
    xg_diag: XgDiagScreen,
    updates: UpdatesScreen,
    profiles: ProfilesScreen,
    input: InputBridge,
    commands: AppCommands,
    instance: Option<PrimaryInstance>,
}

macro_rules! screen_ctx {
    ($self:ident) => {
        ScreenCtx {
            rt: &$self.rt,
            services: &$self.services,
            store: &$self.store,
            alerts: &mut $self.alerts,
        }
    };
}

impl AsusApp {
    pub fn new(
        cc: &eframe::CreationContext<'_>,
        instance: Option<PrimaryInstance>,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        theme::apply(&cc.egui_ctx);
        if let Some(instance) = &instance {
            instance.set_context(cc.egui_ctx.clone());
        }
        let rt = Runtime::new(cc.egui_ctx.clone())?;
        let services = Services::new();
        let settings_file = FileAppSettingsStore::default_settings_file();
        let profiles = FileHardwareProfileStore::for_settings_file(&settings_file);
        let store = Arc::new(ProfiledAppSettingsStore::new(FileAppSettingsStore::new(settings_file), profiles));

        let saved_index = store.load().ui.selected_navigation_index;
        let selected = if (0..DESTINATIONS.len() as i64).contains(&saved_index) {
            saved_index as usize
        } else {
            log_event(
                "Settings.readMismatch",
                &[
                    ("key", "ui.selectedNavigationIndex".into()),
                    ("actual", saved_index.to_string()),
                    ("expected", format!("0..{}", DESTINATIONS.len() - 1)),
                ],
            );
            0
        };

        let mut alerts = Vec::new();
        let mut cx = ScreenCtx {
            rt: &rt,
            services: &services,
            store: &store,
            alerts: &mut alerts,
        };
        let hardware = HardwareScreens::new(&mut cx);
        let xg_diag = XgDiagScreen::new(&mut cx);
        let updates = UpdatesScreen::new(&mut cx);
        let profiles = ProfilesScreen::new(&mut cx);

        Ok(Self {
            rt,
            services,
            store,
            alerts,
            selected,
            hardware,
            xg_diag,
            updates,
            profiles,
            input: InputBridge::default(),
            commands: AppCommands::default(),
            instance,
        })
    }

    fn select(&mut self, index: usize) {
        if index == self.selected {
            return;
        }
        self.selected = index;
        let store = self.store.clone();
        self.rt.spawn_detached(async move {
            let result = update_settings(&*store, |mut settings| {
                settings.ui.selected_navigation_index = index as i64;
                settings
            });
            if let Err(e) = result {
                log::warn!("failed to save selected navigation index: {e:#}");
            }
        });
    }

    fn reload_hardware_screens(&mut self) {
        self.hardware = HardwareScreens::new(&mut screen_ctx!(self));
    }

    fn navigation_rail(&mut self, ui: &mut egui::Ui) {
        let mut clicked = None;
        ui.add_space(12.0);
        for (index, (icon, label)) in DESTINATIONS.iter().enumerate() {
            if rail_item(ui, icon, label, index == self.selected).clicked() {
                clicked = Some(index);
            }
            ui.add_space(4.0);
        }
        if let Some(index) = clicked {
            self.select(index);
        }
    }
}

/// Navigation rail destination: large icon above a single-line label.
fn rail_item(ui: &mut egui::Ui, icon: &str, label: &str, selected: bool) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 64.0), egui::Sense::click());
    let response = response.on_hover_cursor(egui::CursorIcon::PointingHand);
    if ui.is_rect_visible(rect) {
        let painter = ui.painter();
        let fill = if selected {
            theme::PRIMARY_CONTAINER
        } else if response.hovered() {
            theme::SURFACE_CONTAINER_HIGH
        } else {
            egui::Color32::TRANSPARENT
        };
        painter.rect_filled(rect, 16, fill);
        let color = if selected { theme::ON_SURFACE } else { theme::ON_SURFACE_VARIANT };
        let x = rect.center().x;
        painter.text(egui::pos2(x, rect.top() + 22.0), Align2::CENTER_CENTER, icon, FontId::proportional(26.0), color);
        painter.text(egui::pos2(x, rect.bottom() - 14.0), Align2::CENTER_CENTER, label, FontId::proportional(12.0), color);
        if response.has_focus() {
            painter.rect_stroke(rect, 16, egui::Stroke::new(2.0, theme::PRIMARY), egui::StrokeKind::Inside);
        }
    }
    response
}

impl eframe::App for AsusApp {
    fn raw_input_hook(&mut self, _ctx: &egui::Context, raw_input: &mut egui::RawInput) {
        let commands = self.input.pump(raw_input);
        self.commands.next_tab |= commands.next_tab;
        self.commands.previous_tab |= commands.previous_tab;
        self.commands.curve.extend(commands.curve);
    }

    fn logic(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.instance.as_ref().is_some_and(PrimaryInstance::take_activation) {
            ctx.send_viewport_cmd(egui::ViewportCommand::Visible(true));
            ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(false));
            ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
        }

        let mut cx = screen_ctx!(self);
        self.hardware.home.poll(&mut cx);
        self.hardware.tdp.poll(&mut cx);
        self.hardware.gpu.poll(&mut cx);
        self.hardware.fan.poll(&mut cx);
        self.hardware.battery.poll(&mut cx);
        self.xg_diag.poll(&mut cx);
        self.updates.poll(&mut cx);

        let count = DESTINATIONS.len();
        if std::mem::take(&mut self.commands.next_tab) {
            self.select((self.selected + 1) % count);
        }
        if std::mem::take(&mut self.commands.previous_tab) {
            self.select((self.selected + count - 1) % count);
        }
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::Panel::left("navigation")
            .exact_size(104.0)
            .resizable(false)
            .frame(egui::Frame::new().fill(theme::SURFACE).inner_margin(8))
            .show(ui, |ui| {
                ui.with_layout(Layout::top_down(Align::Center), |ui| self.navigation_rail(ui));
            });

        let curve_commands = std::mem::take(&mut self.commands.curve);
        let mut profile_applied = false;
        egui::CentralPanel::default()
            .frame(egui::Frame::new().fill(theme::SURFACE).inner_margin(egui::Margin::symmetric(20, 12)))
            .show(ui, |ui| {
                egui::ScrollArea::vertical()
                    .id_salt(("screen", self.selected))
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        widgets::container(ui, |ui| {
                            let mut cx = screen_ctx!(self);
                            match self.selected {
                                0 => self.hardware.home.ui(ui, &mut cx),
                                1 => self.hardware.tdp.ui(ui, &mut cx),
                                2 => self.hardware.gpu.ui(ui, &mut cx),
                                3 => self.hardware.fan.ui(ui, &mut cx, &curve_commands),
                                4 => self.hardware.battery.ui(ui, &mut cx),
                                5 => self.xg_diag.ui(ui, &mut cx),
                                6 => self.updates.ui(ui, &mut cx),
                                _ => profile_applied = self.profiles.ui(ui, &mut cx),
                            }
                        });
                    });
            });
        if profile_applied {
            self.reload_hardware_screens();
        }

        show_alerts(ui.ctx(), &mut self.alerts);
    }
}
