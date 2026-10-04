//! Modal alert with selectable, copyable details (former `CopyableAlert`).

use egui::{Color32, RichText};

use super::theme;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlertKind {
    Error,
    Info,
    Success,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Alert {
    pub kind: AlertKind,
    pub title: String,
    pub message: String,
    pub details: Option<String>,
}

impl Alert {
    pub fn error(title: impl Into<String>, message: impl Into<String>, details: impl Into<String>) -> Self {
        Self {
            kind: AlertKind::Error,
            title: title.into(),
            message: message.into(),
            details: Some(details.into()),
        }
    }

    pub fn info(title: impl Into<String>, message: impl Into<String>, details: Option<String>) -> Self {
        Self {
            kind: AlertKind::Info,
            title: title.into(),
            message: message.into(),
            details,
        }
    }

    fn icon(&self) -> (&'static str, Color32) {
        match self.kind {
            AlertKind::Error => ("⚠", theme::ERROR),
            AlertKind::Info => ("ℹ", theme::INFO),
            AlertKind::Success => ("✔", theme::SUCCESS),
        }
    }

    fn close_label(&self) -> &'static str {
        match self.kind {
            AlertKind::Error => "Закрыть",
            _ => "OK",
        }
    }
}

/// Shows the first queued alert; removes it when dismissed.
pub fn show_alerts(ctx: &egui::Context, alerts: &mut Vec<Alert>) {
    let Some(alert) = alerts.first().cloned() else {
        return;
    };
    let mut close = false;
    let modal = egui::Modal::new(egui::Id::new("alert_modal")).show(ctx, |ui| {
        ui.set_max_width(520.0);
        let (icon, color) = alert.icon();
        ui.horizontal(|ui| {
            ui.label(RichText::new(icon).size(26.0).color(color));
            ui.label(RichText::new(&alert.title).size(20.0).strong());
        });
        ui.add_space(8.0);
        ui.label(&alert.message);
        if let Some(details) = &alert.details {
            ui.add_space(8.0);
            egui::Frame::new()
                .fill(theme::SURFACE_CONTAINER_HIGHEST)
                .corner_radius(8)
                .inner_margin(10)
                .show(ui, |ui| {
                    egui::ScrollArea::vertical().max_height(220.0).show(ui, |ui| {
                        ui.add(egui::Label::new(RichText::new(details).monospace()).selectable(true));
                    });
                });
            if ui
                .button("📋 Копировать")
                .on_hover_text("Копировать детали")
                .clicked()
            {
                ui.ctx().copy_text(details.clone());
            }
        }
        ui.add_space(12.0);
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.button(alert.close_label()).clicked() {
                close = true;
            }
        });
    });
    if close || modal.should_close() {
        alerts.remove(0);
    }
}
