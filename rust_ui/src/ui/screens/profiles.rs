//! Hardware profiles (former `ProfilesScreen`). Profile files are small, so store calls run
//! on the UI thread.

use egui::{Color32, RichText, Ui};

use crate::settings::{update_settings, AppSettingsStore, HardwareProfile, HardwareProfileStore};
use crate::ui::alert::Alert;
use crate::ui::{widgets, ScreenCtx};

#[derive(Default)]
pub struct ProfilesScreen {
    profiles: Vec<HardwareProfile>,
    active_profile_id: Option<String>,
    adding: bool,
    focus_name: bool,
    name: String,
}

impl ProfilesScreen {
    pub fn new(cx: &mut ScreenCtx) -> Self {
        let mut screen = Self::default();
        screen.reload(cx);
        screen
    }

    fn reload(&mut self, cx: &ScreenCtx) {
        self.active_profile_id = cx.store.load().ui.active_hardware_profile_id;
        self.profiles = cx.store.profiles().list();
    }

    fn set_active_profile(cx: &mut ScreenCtx, id: &str) -> bool {
        let result = update_settings(&**cx.store, |mut settings| {
            settings.ui.active_hardware_profile_id = Some(id.to_string());
            settings
        });
        match result {
            Ok(_) => true,
            Err(e) => {
                cx.alert(Alert::error("Profile Error", "Failed to activate profile", format!("{e:#}")));
                false
            }
        }
    }

    fn save_profile(&mut self, cx: &mut ScreenCtx) -> bool {
        let name = self.name.trim().to_string();
        if name.is_empty() {
            return false;
        }
        let hardware = cx.store.load().hardware;
        let applied = match cx.store.profiles().create(&name, hardware) {
            Ok(profile) => Self::set_active_profile(cx, &profile.id),
            Err(e) => {
                cx.alert(Alert::error("Profile Error", "Failed to save profile", format!("{e:#}")));
                false
            }
        };
        self.adding = false;
        self.name.clear();
        self.reload(cx);
        applied
    }

    /// Returns true when a profile was applied, so hardware screens must reload.
    pub fn ui(&mut self, ui: &mut Ui, cx: &mut ScreenCtx) -> bool {
        widgets::screen_title(ui, "Profiles");
        let mut applied = false;

        if self.adding {
            ui.horizontal(|ui| {
                let response = ui.add(
                    egui::TextEdit::singleline(&mut self.name)
                        .hint_text("Profile name")
                        .desired_width(360.0),
                );
                if std::mem::take(&mut self.focus_name) {
                    response.request_focus();
                }
                let submitted = response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
                if submitted || widgets::primary_button(ui, "Save", true).clicked() {
                    applied |= self.save_profile(cx);
                }
                if ui.button("Cancel").clicked() {
                    self.adding = false;
                    self.name.clear();
                }
            });
        } else if widgets::primary_button(ui, "➕ Add", true).clicked() {
            self.adding = true;
            self.focus_name = true;
        }
        ui.add_space(16.0);

        if self.profiles.is_empty() {
            ui.label("No profiles");
            return applied;
        }

        let mut apply = None;
        let mut delete = None;
        egui::Grid::new("profiles")
            .num_columns(3)
            .striped(false)
            .min_col_width(72.0)
            .spacing([16.0, 6.0])
            .show(ui, |ui| {
                ui.label(RichText::new("Profile").strong());
                ui.label(RichText::new("Apply").strong());
                ui.label(RichText::new("Delete").strong());
                ui.end_row();
                for profile in &self.profiles {
                    let active = self.active_profile_id.as_deref() == Some(profile.id.as_str());
                    let name = if active {
                        RichText::new(format!("● {}", profile.name)).color(Color32::from_rgb(0x81, 0xC7, 0x84))
                    } else {
                        RichText::new(&profile.name)
                    };
                    ui.label(name);
                    if ui.add_enabled(!active, egui::Button::new("Apply")).clicked() {
                        apply = Some(profile.id.clone());
                    }
                    if ui.add_enabled(!active, egui::Button::new("Delete")).clicked() {
                        delete = Some(profile.id.clone());
                    }
                    ui.end_row();
                }
            });

        if let Some(id) = apply {
            applied |= Self::set_active_profile(cx, &id);
            self.reload(cx);
        }
        if let Some(id) = delete {
            if self.active_profile_id.as_deref() != Some(id.as_str()) {
                if let Err(e) = cx.store.profiles().delete(&id) {
                    cx.alert(Alert::error("Profile Error", "Failed to delete profile", format!("{e:#}")));
                }
                self.reload(cx);
            }
        }
        applied
    }
}
