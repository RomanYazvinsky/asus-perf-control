//! Shared widgets mirroring the former Material 3 components.

use egui::{Align, Color32, CornerRadius, FontId, Frame, Layout, Margin, Rect, Response, RichText, Sense, Stroke, Ui, UiBuilder, Vec2};

use super::theme;

/// Bootstrap container breakpoints: (minimum available width, container width).
const CONTAINER_BREAKPOINTS: [(f32, f32); 5] = [(1400.0, 1320.0), (1200.0, 1140.0), (992.0, 960.0), (768.0, 720.0), (576.0, 540.0)];

/// Width of a Bootstrap-like 12-column container for the given available width.
pub fn container_width(available: f32) -> f32 {
    CONTAINER_BREAKPOINTS
        .iter()
        .find(|(min, _)| available >= *min)
        .map_or(available, |(_, width)| *width)
}

/// Horizontally centered, fixed-width content column (Bootstrap `.container` with a `.col-12` row).
pub fn container<R>(ui: &mut Ui, add_contents: impl FnOnce(&mut Ui) -> R) -> R {
    let available = ui.available_width();
    let width = container_width(available);
    let cursor = ui.cursor().min;
    let left = cursor.x + ((available - width) / 2.0).max(0.0);
    let rect = Rect::from_min_size(egui::pos2(left, cursor.y), Vec2::new(width, f32::INFINITY));
    ui.scope_builder(UiBuilder::new().max_rect(rect).layout(Layout::top_down(Align::Min)), |ui| {
        ui.set_width(width);
        add_contents(ui)
    })
    .inner
}

/// Material-style card.
pub fn card<R>(ui: &mut Ui, add_contents: impl FnOnce(&mut Ui) -> R) -> R {
    card_with_fill(ui, theme::SURFACE_CONTAINER, add_contents)
}

pub fn card_with_fill<R>(ui: &mut Ui, fill: Color32, add_contents: impl FnOnce(&mut Ui) -> R) -> R {
    Frame::new()
        .fill(fill)
        .corner_radius(CornerRadius::same(12))
        .inner_margin(Margin::same(16))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            add_contents(ui)
        })
        .inner
}

pub fn screen_title(ui: &mut Ui, title: &str) {
    ui.add_space(4.0);
    ui.label(RichText::new(title).size(22.0).strong());
    ui.add_space(12.0);
}

pub fn heading(ui: &mut Ui, text: &str) {
    ui.label(RichText::new(text).size(20.0).strong());
}

pub fn title(ui: &mut Ui, text: &str) {
    ui.label(RichText::new(text).size(16.0).strong());
}

pub fn muted(ui: &mut Ui, text: impl Into<String>) {
    ui.label(RichText::new(text.into()).color(theme::ON_SURFACE_VARIANT));
}

pub fn small(ui: &mut Ui, text: impl Into<String>) {
    ui.label(RichText::new(text.into()).small().color(theme::ON_SURFACE_VARIANT));
}

/// Centered spinner filling the available space.
pub fn loading(ui: &mut Ui) {
    ui.centered_and_justified(|ui| {
        ui.spinner();
    });
}

/// Centered message with an icon and an optional button; returns true when the button is clicked.
pub fn centered_message(ui: &mut Ui, icon: &str, color: Color32, message: &str, button: Option<&str>) -> bool {
    let mut clicked = false;
    ui.vertical_centered(|ui| {
        ui.add_space((ui.available_height() / 3.0).max(16.0));
        ui.label(RichText::new(icon).size(48.0).color(color));
        ui.add_space(12.0);
        ui.label(RichText::new(message).size(16.0));
        if let Some(text) = button {
            ui.add_space(12.0);
            clicked = ui.button(text).clicked();
        }
    });
    clicked
}

/// Full-width option button: icon, title, subtitle and a check mark when selected.
pub fn choice_button(ui: &mut Ui, icon: &str, title: &str, subtitle: &str, selected: bool, enabled: bool) -> Response {
    let fill = if selected {
        theme::PRIMARY_CONTAINER
    } else {
        theme::SURFACE_CONTAINER_HIGH
    };
    let frame = Frame::new()
        .fill(fill)
        .corner_radius(CornerRadius::same(20))
        .inner_margin(Margin::symmetric(16, 12));
    let inner = ui.add_enabled_ui(enabled, |ui| {
        frame
            .show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.horizontal(|ui| {
                    ui.label(RichText::new(icon).size(26.0));
                    ui.add_space(12.0);
                    ui.vertical(|ui| {
                        let mut text = RichText::new(title).size(16.0);
                        if selected {
                            text = text.strong();
                        }
                        ui.label(text);
                        small(ui, subtitle);
                    });
                    if selected {
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            ui.label(RichText::new("✔").size(20.0).color(theme::SUCCESS));
                        });
                    }
                });
            })
            .response
    });
    let response = ui.interact(inner.inner.rect, inner.inner.id.with("choice"), if enabled { Sense::click() } else { Sense::hover() });
    if response.hovered() && enabled {
        ui.painter().rect_stroke(
            inner.inner.rect,
            CornerRadius::same(20),
            Stroke::new(1.0, theme::PRIMARY),
            egui::StrokeKind::Inside,
        );
    }
    if response.has_focus() {
        ui.painter().rect_stroke(
            inner.inner.rect,
            CornerRadius::same(20),
            Stroke::new(2.0, theme::PRIMARY),
            egui::StrokeKind::Inside,
        );
    }
    response.on_hover_cursor(egui::CursorIcon::PointingHand)
}

/// Card with a labelled integer slider (former `TdpSliderCard`). Returns true when changed.
#[allow(clippy::too_many_arguments)]
pub fn slider_card(
    ui: &mut Ui,
    label: &str,
    value: &mut f64,
    min: f64,
    max: f64,
    unit: &str,
    description: &str,
    enabled: bool,
) -> bool {
    let mut changed = false;
    card(ui, |ui| {
        ui.horizontal(|ui| {
            title(ui, label);
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                ui.label(
                    RichText::new(format!("{}{unit}", value.round()))
                        .size(18.0)
                        .strong()
                        .color(theme::PRIMARY),
                );
            });
        });
        let color = if enabled {
            theme::ON_SURFACE_VARIANT
        } else {
            theme::ON_SURFACE_VARIANT.gamma_multiply(0.5)
        };
        ui.label(RichText::new(description).small().color(color));
        ui.add_space(4.0);
        ui.spacing_mut().slider_width = (ui.available_width() - 16.0).max(80.0);
        let slider = egui::Slider::new(value, min..=max.max(min))
            .step_by(1.0)
            .show_value(false);
        changed = ui.add_enabled(enabled, slider).changed();
        ui.horizontal(|ui| {
            small(ui, format!("{}{unit}", min.round()));
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                small(ui, format!("{}{unit}", max.round()));
            });
        });
    });
    changed
}

/// Card with a switch row (former `SwitchListTile`). Returns true when toggled.
pub fn switch_card(ui: &mut Ui, label: &str, subtitle: &str, value: &mut bool) -> bool {
    card(ui, |ui| {
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.label(RichText::new(label).size(16.0));
                small(ui, subtitle);
            });
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| toggle(ui, value).changed())
                .inner
        })
        .inner
    })
}

/// iOS/Material-like toggle switch.
pub fn toggle(ui: &mut Ui, on: &mut bool) -> Response {
    let size = Vec2::new(44.0, 24.0);
    let (rect, mut response) = ui.allocate_exact_size(size, Sense::click());
    if response.clicked() {
        *on = !*on;
        response.mark_changed();
    }
    if ui.is_rect_visible(rect) {
        let t = ui.ctx().animate_bool_responsive(response.id, *on);
        let radius = rect.height() / 2.0;
        let track = if *on { theme::PRIMARY } else { theme::SURFACE_CONTAINER_HIGHEST };
        ui.painter().rect_filled(rect, radius, track);
        if response.has_focus() {
            ui.painter()
                .rect_stroke(rect.expand(2.0), radius + 2.0, Stroke::new(2.0, theme::PRIMARY), egui::StrokeKind::Outside);
        }
        let x = egui::lerp((rect.left() + radius)..=(rect.right() - radius), t);
        let knob = if *on { theme::ON_PRIMARY } else { theme::ON_SURFACE_VARIANT };
        ui.painter().circle_filled(egui::pos2(x, rect.center().y), radius - 4.0, knob);
    }
    response.on_hover_cursor(egui::CursorIcon::PointingHand)
}

/// Coloured banner (info / error) with an icon.
pub fn banner(ui: &mut Ui, icon: &str, text: &str, fill: Color32, fg: Color32) {
    card_with_fill(ui, fill, |ui| {
        ui.horizontal_wrapped(|ui| {
            ui.label(RichText::new(icon).color(fg));
            ui.label(RichText::new(text).color(fg));
        });
    });
}

/// Label / value row.
pub fn status_row(ui: &mut Ui, label: &str, value: &str) {
    ui.horizontal(|ui| {
        ui.label(RichText::new(label).strong());
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            ui.label(value);
        });
    });
}

/// Silent / Balanced / Turbo tabs (former `ProfileTabBar`). Returns the clicked mode.
pub fn profile_label(mode: i32) -> &'static str {
    match mode {
        0 => "Silent",
        2 => "Turbo",
        _ => "Balanced",
    }
}

/// Explains that the profile tabs choose what is edited, not what is active.
pub fn editing_profile_hint(ui: &mut Ui, editing_mode: i32, active_mode: Option<i32>) {
    let text = match active_mode {
        Some(active) if active == editing_mode => format!("Editing {} — the active profile", profile_label(editing_mode)),
        Some(active) => format!(
            "Editing {} — takes effect when that profile is active (now: {})",
            profile_label(editing_mode),
            profile_label(active)
        ),
        None => format!("Editing {}", profile_label(editing_mode)),
    };
    small(ui, &text);
}

pub fn profile_tabs(ui: &mut Ui, selected_mode: i32) -> Option<i32> {
    let selected_mode = if (0..=2).contains(&selected_mode) { selected_mode } else { 1 };
    let mut clicked = None;
    let tabs = [(0, "🔇", "Silent"), (1, "⚖", "Balanced"), (2, "⏩", "Turbo")];
    ui.columns(tabs.len(), |columns| {
        for (column, (mode, icon, label)) in columns.iter_mut().zip(tabs) {
            let selected = mode == selected_mode;
            let color = if selected { theme::PRIMARY } else { theme::ON_SURFACE_VARIANT };
            let response = column
                .vertical_centered(|ui| {
                    let (rect, response) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 56.0), Sense::click());
                    let response = response.on_hover_cursor(egui::CursorIcon::PointingHand);
                    if response.hovered() {
                        ui.painter().rect_filled(rect, 8, theme::SURFACE_CONTAINER_HIGH);
                    }
                    let x = rect.center().x;
                    let painter = ui.painter();
                    painter.text(egui::pos2(x, rect.top() + 19.0), egui::Align2::CENTER_CENTER, icon, FontId::proportional(24.0), color);
                    painter.text(egui::pos2(x, rect.bottom() - 11.0), egui::Align2::CENTER_CENTER, label, FontId::proportional(13.0), color);
                    let line = egui::Rect::from_min_size(
                        egui::pos2(response.rect.left(), response.rect.bottom()),
                        Vec2::new(response.rect.width(), 3.0),
                    );
                    if selected {
                        ui.painter().rect_filled(line, 1.0, theme::PRIMARY);
                    } else {
                        ui.painter().rect_filled(line.with_min_y(line.max.y - 1.0), 0.0, theme::OUTLINE_VARIANT);
                    }
                    response
                })
                .inner;
            if response.clicked() {
                clicked = Some(mode);
            }
        }
    });
    ui.add_space(6.0);
    clicked
}

/// Chip with icon and label.
pub fn chip(ui: &mut Ui, icon: &str, label: &str) {
    Frame::new()
        .stroke(Stroke::new(1.0, theme::OUTLINE_VARIANT))
        .corner_radius(CornerRadius::same(8))
        .inner_margin(Margin::symmetric(10, 4))
        .show(ui, |ui| {
            ui.label(format!("{icon} {label}"));
        });
}

/// Filled primary button.
pub fn primary_button(ui: &mut Ui, text: &str, enabled: bool) -> Response {
    ui.add_enabled(
        enabled,
        egui::Button::new(RichText::new(text).color(theme::ON_PRIMARY).strong())
            .fill(theme::PRIMARY)
            .corner_radius(CornerRadius::same(20))
            .min_size(Vec2::new(0.0, 36.0)),
    )
}

/// Outlined secondary button.
pub fn outlined_button(ui: &mut Ui, text: &str, enabled: bool) -> Response {
    ui.add_enabled(
        enabled,
        egui::Button::new(RichText::new(text).color(theme::PRIMARY))
            .fill(Color32::TRANSPARENT)
            .stroke(Stroke::new(1.0, theme::OUTLINE))
            .corner_radius(CornerRadius::same(20))
            .min_size(Vec2::new(0.0, 36.0)),
    )
}

#[cfg(test)]
mod tests {
    use super::container_width;

    #[test]
    fn container_width_snaps_to_bootstrap_breakpoints() {
        assert_eq!(container_width(400.0), 400.0);
        assert_eq!(container_width(600.0), 540.0);
        assert_eq!(container_width(800.0), 720.0);
        assert_eq!(container_width(1000.0), 960.0);
        assert_eq!(container_width(1300.0), 1140.0);
        assert_eq!(container_width(1800.0), 1320.0);
    }
}
