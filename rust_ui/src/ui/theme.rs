//! Dark Material 3 palette (seed: blue), as used by the former Flutter theme.

use egui::{Color32, Context, CornerRadius, Stroke, Visuals};

pub const PRIMARY: Color32 = Color32::from_rgb(0xA8, 0xC7, 0xFA);
pub const ON_PRIMARY: Color32 = Color32::from_rgb(0x06, 0x2E, 0x6F);
pub const PRIMARY_CONTAINER: Color32 = Color32::from_rgb(0x1F, 0x3B, 0x6B);
pub const SURFACE: Color32 = Color32::from_rgb(0x11, 0x13, 0x18);
pub const SURFACE_CONTAINER: Color32 = Color32::from_rgb(0x1D, 0x20, 0x24);
pub const SURFACE_CONTAINER_HIGH: Color32 = Color32::from_rgb(0x28, 0x2A, 0x2F);
pub const SURFACE_CONTAINER_HIGHEST: Color32 = Color32::from_rgb(0x33, 0x35, 0x3A);
pub const ON_SURFACE: Color32 = Color32::from_rgb(0xE2, 0xE2, 0xE9);
pub const ON_SURFACE_VARIANT: Color32 = Color32::from_rgb(0xC4, 0xC6, 0xD0);
pub const OUTLINE: Color32 = Color32::from_rgb(0x8E, 0x90, 0x99);
pub const OUTLINE_VARIANT: Color32 = Color32::from_rgb(0x44, 0x47, 0x4F);
pub const SECONDARY_CONTAINER: Color32 = Color32::from_rgb(0x3E, 0x47, 0x59);
pub const ON_SECONDARY_CONTAINER: Color32 = Color32::from_rgb(0xDA, 0xE2, 0xF9);
pub const ERROR: Color32 = Color32::from_rgb(0xFF, 0xB4, 0xAB);
pub const ERROR_CONTAINER: Color32 = Color32::from_rgb(0x93, 0x00, 0x0A);
pub const ON_ERROR_CONTAINER: Color32 = Color32::from_rgb(0xFF, 0xDA, 0xD6);
pub const SUCCESS: Color32 = Color32::from_rgb(0x66, 0xBB, 0x6A);
pub const WARNING: Color32 = Color32::from_rgb(0xFF, 0x98, 0x00);
pub const INFO: Color32 = Color32::from_rgb(0x42, 0xA5, 0xF5);

pub fn apply(ctx: &Context) {
    let mut visuals = Visuals::dark();
    visuals.panel_fill = SURFACE;
    visuals.window_fill = SURFACE_CONTAINER_HIGH;
    visuals.extreme_bg_color = SURFACE_CONTAINER_HIGHEST;
    visuals.faint_bg_color = SURFACE_CONTAINER;
    visuals.override_text_color = Some(ON_SURFACE);
    visuals.hyperlink_color = PRIMARY;
    visuals.selection.bg_fill = PRIMARY_CONTAINER;
    visuals.selection.stroke = Stroke::new(1.0, PRIMARY);
    visuals.window_corner_radius = CornerRadius::same(24);
    visuals.widgets.inactive.corner_radius = CornerRadius::same(8);
    visuals.widgets.hovered.corner_radius = CornerRadius::same(8);
    visuals.widgets.active.corner_radius = CornerRadius::same(8);
    visuals.widgets.inactive.weak_bg_fill = SURFACE_CONTAINER_HIGH;
    visuals.widgets.hovered.weak_bg_fill = SURFACE_CONTAINER_HIGHEST;
    visuals.widgets.active.weak_bg_fill = PRIMARY_CONTAINER;
    visuals.widgets.inactive.bg_fill = SURFACE_CONTAINER_HIGHEST;
    ctx.set_visuals_of(egui::Theme::Dark, visuals);
    ctx.set_theme(egui::Theme::Dark);
    ctx.style_mut_of(egui::Theme::Dark, |style| {
        style.spacing.item_spacing = egui::vec2(8.0, 8.0);
        style.spacing.button_padding = egui::vec2(14.0, 6.0);
        style.spacing.interact_size.y = 28.0;
        for (text_style, font) in style.text_styles.iter_mut() {
            font.size = match text_style {
                egui::TextStyle::Small => 12.0,
                egui::TextStyle::Body | egui::TextStyle::Button => 14.5,
                egui::TextStyle::Heading => 22.0,
                egui::TextStyle::Monospace => 13.5,
                _ => font.size,
            };
        }
    });
}
