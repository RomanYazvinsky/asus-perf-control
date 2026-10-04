//! 8-point fan curve editor (former `FanCurveEditor`).
//!
//! Mouse: drag a point. Keyboard / gamepad while focused: `[` `]` or PageUp/PageDown select the
//! previous/next point, ←/→ move its temperature, ↑/↓ its speed. Apply sends the curve.

use egui::{pos2, vec2, Align2, Color32, FontId, Pos2, Rect, RichText, Sense, Shape, Stroke, Ui};

use super::{theme, widgets};
use crate::features::fan::{update_point_position, FanCurvePoint};
use crate::input::UiInputCommand;

const MIN_X: f64 = 20.0;
const MAX_X: f64 = 100.0;
const MIN_Y: f64 = 0.0;
const MAX_Y: f64 = 100.0;
const LEFT_AXIS: f32 = 40.0;
const BOTTOM_AXIS: f32 = 30.0;
const CHART_HEIGHT: f32 = 300.0;
/// Squared distance in chart units within which a press grabs a point.
const GRAB_DISTANCE_SQ: f64 = 200.0;

#[derive(Default)]
pub struct CurveEditor {
    points: Vec<FanCurvePoint>,
    source: Vec<FanCurvePoint>,
    dragged: Option<usize>,
    selected: Option<usize>,
}

impl CurveEditor {
    /// Takes a new curve from the state unless a drag is in progress.
    fn sync(&mut self, curve: &[FanCurvePoint]) {
        if self.dragged.is_none() && self.source != curve {
            self.source = curve.to_vec();
            self.points = curve.to_vec();
        }
    }

    pub fn selected_label(&self) -> String {
        match self.selected.and_then(|i| self.points.get(i).map(|p| (i, p))) {
            Some((i, p)) => format!(
                "Point {}: {}°C / {}%",
                i + 1,
                p.temp_c.round(),
                p.speed_percent.round()
            ),
            None => "No point selected".into(),
        }
    }

    /// Gamepad / keyboard curve commands.
    pub fn handle_command(&mut self, command: UiInputCommand) {
        if self.points.is_empty() {
            return;
        }
        let last = self.points.len() - 1;
        let current = self.selected.unwrap_or(0);
        match command {
            UiInputCommand::CurvePreviousPoint => self.selected = Some(current.saturating_sub(1)),
            UiInputCommand::CurveNextPoint => self.selected = Some((current + 1).min(last)),
            UiInputCommand::CurveSpeedUp => self.nudge(0.0, 1.0),
            UiInputCommand::CurveSpeedDown => self.nudge(0.0, -1.0),
            UiInputCommand::CurveTempLeft => self.nudge(-1.0, 0.0),
            UiInputCommand::CurveTempRight => self.nudge(1.0, 0.0),
            _ => {}
        }
    }

    fn nudge(&mut self, delta_temp: f64, delta_speed: f64) {
        let index = self.selected.unwrap_or(0);
        self.selected = Some(index);
        let point = self.points[index];
        update_point_position(&mut self.points, index, point.temp_c + delta_temp, point.speed_percent + delta_speed);
    }

    /// Draws the editor. Returns the edited curve when Apply is pressed.
    pub fn show(
        &mut self,
        ui: &mut Ui,
        curve: &[FanCurvePoint],
        read_only: bool,
        current_speed: Option<f64>,
        commands: &[UiInputCommand],
    ) -> Option<Vec<FanCurvePoint>> {
        self.sync(curve);
        let mut applied = None;
        widgets::card(ui, |ui| {
            ui.horizontal(|ui| {
                widgets::heading(ui, "Fan Curve (8 points)");
                if !read_only {
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("Apply").clicked() {
                            applied = Some(self.points.clone());
                        }
                    });
                }
            });
            ui.add_space(12.0);
            self.chart(ui, read_only, current_speed, commands);
            ui.add_space(12.0);
            if self.selected.is_some() {
                ui.label(self.selected_label());
            }
            widgets::small(ui, "Temperature (°C) → Fan Speed (%)");
        });
        applied
    }

    fn chart(&mut self, ui: &mut Ui, read_only: bool, current_speed: Option<f64>, commands: &[UiInputCommand]) {
        let size = vec2(ui.available_width(), CHART_HEIGHT);
        let sense = if read_only { Sense::hover() } else { Sense::click_and_drag() };
        let (outer, response) = ui.allocate_exact_size(size, sense);
        let plot = Rect::from_min_max(pos2(outer.left() + LEFT_AXIS, outer.top()), pos2(outer.right(), outer.bottom() - BOTTOM_AXIS));

        let to_screen = |p: &FanCurvePoint| {
            pos2(
                plot.left() + ((p.temp_c - MIN_X) / (MAX_X - MIN_X)) as f32 * plot.width(),
                plot.bottom() - ((p.speed_percent - MIN_Y) / (MAX_Y - MIN_Y)) as f32 * plot.height(),
            )
        };
        let to_chart = |pos: Pos2| {
            (
                MIN_X + ((pos.x - plot.left()) / plot.width()) as f64 * (MAX_X - MIN_X),
                MAX_Y - ((pos.y - plot.top()) / plot.height()) as f64 * (MAX_Y - MIN_Y),
            )
        };

        if !read_only {
            if response.has_focus() {
                ui.memory_mut(|m| {
                    m.set_focus_lock_filter(
                        response.id,
                        egui::EventFilter {
                            horizontal_arrows: true,
                            vertical_arrows: true,
                            ..Default::default()
                        },
                    )
                });
                if self.selected.is_none() && !self.points.is_empty() {
                    self.selected = Some(0);
                }
                for command in keyboard_commands(ui).into_iter().chain(commands.iter().copied()) {
                    self.handle_command(command);
                }
            }
            if response.drag_started() || response.clicked() {
                response.request_focus();
                if let Some(pos) = response.interact_pointer_pos() {
                    let (x, y) = to_chart(pos);
                    self.dragged = nearest_point(&self.points, x, y);
                    if self.dragged.is_some() {
                        self.selected = self.dragged;
                    }
                }
            }
            if response.dragged() {
                if let (Some(index), Some(pos)) = (self.dragged, response.interact_pointer_pos()) {
                    let (x, y) = to_chart(pos);
                    update_point_position(&mut self.points, index, x, y);
                }
            }
            if response.drag_stopped() || (!response.dragged() && !response.drag_started()) {
                self.dragged = None;
            }
        }

        let painter = ui.painter_at(outer);
        let grid = Stroke::new(1.0, theme::OUTLINE.gamma_multiply(0.2));
        let label_font = FontId::proportional(11.0);
        for t in (20..=100).step_by(10) {
            let x = to_screen(&FanCurvePoint::new(t as f64, 0.0)).x;
            painter.line_segment([pos2(x, plot.top()), pos2(x, plot.bottom())], grid);
            painter.text(pos2(x, plot.bottom() + 6.0), Align2::CENTER_TOP, format!("{t}°C"), label_font.clone(), theme::ON_SURFACE_VARIANT);
        }
        for s in (0..=100).step_by(10) {
            let y = to_screen(&FanCurvePoint::new(MIN_X, s as f64)).y;
            painter.line_segment([pos2(plot.left(), y), pos2(plot.right(), y)], grid);
            painter.text(pos2(plot.left() - 6.0, y), Align2::RIGHT_CENTER, format!("{s}%"), label_font.clone(), theme::ON_SURFACE_VARIANT);
        }
        painter.rect_stroke(plot, 0.0, Stroke::new(1.0, theme::OUTLINE.gamma_multiply(0.3)), egui::StrokeKind::Inside);

        if let Some(speed) = current_speed.filter(|s| (MIN_Y..=MAX_Y).contains(s)) {
            let y = to_screen(&FanCurvePoint::new(MIN_X, speed)).y;
            painter.add(Shape::dashed_line(&[pos2(plot.left(), y), pos2(plot.right(), y)], Stroke::new(2.0, theme::WARNING), 5.0, 5.0));
        }

        let screen: Vec<Pos2> = self.points.iter().map(to_screen).collect();
        painter.add(Shape::line(screen.clone(), Stroke::new(4.0, theme::PRIMARY)));
        for (i, pos) in screen.iter().enumerate() {
            let color: Color32 = if self.dragged == Some(i) {
                theme::ON_SECONDARY_CONTAINER
            } else if self.selected == Some(i) {
                Color32::from_rgb(0xD6, 0xBE, 0xE4)
            } else {
                theme::PRIMARY
            };
            let stroke_width = if self.selected == Some(i) { 4.0 } else { 2.0 };
            painter.circle(*pos, 8.0, color, Stroke::new(stroke_width, theme::SURFACE));
        }
        if response.has_focus() {
            painter.rect_stroke(outer, 4.0, Stroke::new(2.0, theme::PRIMARY), egui::StrokeKind::Inside);
        }
        if read_only {
            ui.label(RichText::new("Read only").small());
        }
    }
}

fn nearest_point(points: &[FanCurvePoint], x: f64, y: f64) -> Option<usize> {
    points
        .iter()
        .enumerate()
        .map(|(i, p)| (i, (x - p.temp_c).powi(2) + (y - p.speed_percent).powi(2)))
        .min_by(|a, b| a.1.total_cmp(&b.1))
        .filter(|(_, d)| *d < GRAB_DISTANCE_SQ)
        .map(|(i, _)| i)
}

fn keyboard_commands(ui: &Ui) -> Vec<UiInputCommand> {
    ui.input(|i| {
        let mut commands = Vec::new();
        let pairs = [
            (egui::Key::OpenBracket, UiInputCommand::CurvePreviousPoint),
            (egui::Key::PageUp, UiInputCommand::CurvePreviousPoint),
            (egui::Key::CloseBracket, UiInputCommand::CurveNextPoint),
            (egui::Key::PageDown, UiInputCommand::CurveNextPoint),
            (egui::Key::ArrowUp, UiInputCommand::CurveSpeedUp),
            (egui::Key::ArrowDown, UiInputCommand::CurveSpeedDown),
            (egui::Key::ArrowLeft, UiInputCommand::CurveTempLeft),
            (egui::Key::ArrowRight, UiInputCommand::CurveTempRight),
        ];
        for (key, command) in pairs {
            if i.key_pressed(key) {
                commands.push(command);
            }
        }
        commands
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::fan::default_fan_curve;

    #[test]
    fn grabs_only_nearby_points() {
        let points = default_fan_curve();
        assert_eq!(nearest_point(&points, 41.0, 51.0), Some(2));
        assert_eq!(nearest_point(&points, 45.0, 95.0), None);
    }

    #[test]
    fn commands_move_selection_and_nudge_points() {
        let mut editor = CurveEditor::default();
        editor.sync(&default_fan_curve());

        editor.handle_command(UiInputCommand::CurveNextPoint);
        editor.handle_command(UiInputCommand::CurveSpeedUp);
        editor.handle_command(UiInputCommand::CurveTempRight);

        assert_eq!(editor.selected, Some(1));
        assert_eq!(editor.points[1].temp_c, 31.0);
        assert_eq!(editor.points[1].speed_percent.round(), 41.0);
        assert_eq!(editor.selected_label(), "Point 2: 31°C / 41%");

        editor.handle_command(UiInputCommand::CurvePreviousPoint);
        editor.handle_command(UiInputCommand::CurvePreviousPoint);
        assert_eq!(editor.selected, Some(0));
    }

    #[test]
    fn keeps_local_edits_until_the_source_curve_changes() {
        let mut editor = CurveEditor::default();
        let curve = default_fan_curve();
        editor.sync(&curve);
        editor.handle_command(UiInputCommand::CurveSpeedUp);
        editor.sync(&curve);
        assert_ne!(editor.points, curve);

        let mut changed = curve.clone();
        changed[0].speed_percent = 10.0;
        editor.sync(&changed);
        assert_eq!(editor.points, changed);
    }
}
