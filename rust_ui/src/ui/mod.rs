//! egui presentation layer.

pub mod alert;
pub mod curve_editor;
pub mod screens;
pub mod theme;
pub mod widgets;

use std::sync::Arc;

use crate::features::services::Services;
use crate::runtime::Runtime;
use crate::settings::{FileAppSettingsStore, FileHardwareProfileStore, ProfiledAppSettingsStore};
use alert::Alert;

/// Settings store used by the app: UI in `settings.json`, hardware in the active profile.
pub type AppStore = ProfiledAppSettingsStore<FileAppSettingsStore, FileHardwareProfileStore>;

/// What every screen needs to start jobs and report problems.
pub struct ScreenCtx<'a> {
    pub rt: &'a Runtime,
    pub services: &'a Arc<Services>,
    pub store: &'a Arc<AppStore>,
    pub alerts: &'a mut Vec<Alert>,
}

impl ScreenCtx<'_> {
    pub fn alert(&mut self, alert: Alert) {
        self.alerts.push(alert);
    }

    /// Wake the UI later (timers such as the fan RPM refresh).
    pub fn repaint_after(&self, delay: std::time::Duration) {
        self.rt.repaint_after(delay);
    }
}
