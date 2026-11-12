//! The look: Project ALM v4.5.0 Insanux's dark tool window — a navy wall, panels a shade lighter,
//! one amber accent for what acts and a green one for what is fine.

use egui::{Color32, RichText, Style, Visuals};

pub const BG: Color32 = Color32::from_rgb(15, 18, 24);
pub const PANEL: Color32 = Color32::from_rgb(22, 27, 35);
pub const PANEL_HI: Color32 = Color32::from_rgb(30, 36, 46);
pub const LINE: Color32 = Color32::from_rgb(42, 50, 62);
pub const TEXT: Color32 = Color32::from_rgb(222, 228, 238);
pub const DIM: Color32 = Color32::from_rgb(138, 149, 165);
pub const AMBER: Color32 = Color32::from_rgb(240, 165, 60);
pub const GREEN: Color32 = Color32::from_rgb(96, 200, 140);
pub const RED: Color32 = Color32::from_rgb(230, 100, 100);

/// Applies the window's look to a context.
pub fn apply(ctx: &egui::Context) {
    let mut style: Style = (*ctx.style()).clone();
    let mut visuals = Visuals::dark();
    visuals.panel_fill = BG;
    visuals.window_fill = PANEL;
    visuals.extreme_bg_color = Color32::from_rgb(12, 15, 20);
    visuals.faint_bg_color = PANEL_HI;
    visuals.widgets.noninteractive.bg_fill = PANEL;
    visuals.widgets.noninteractive.fg_stroke.color = TEXT;
    visuals.widgets.inactive.bg_fill = PANEL_HI;
    visuals.widgets.inactive.fg_stroke.color = TEXT;
    visuals.widgets.hovered.bg_fill = Color32::from_rgb(40, 48, 60);
    visuals.widgets.hovered.fg_stroke.color = AMBER;
    visuals.widgets.active.bg_fill = Color32::from_rgb(48, 57, 70);
    visuals.widgets.active.fg_stroke.color = AMBER;
    visuals.selection.bg_fill = AMBER.linear_multiply(0.35);
    visuals.selection.stroke.color = AMBER;
    visuals.window_stroke.color = LINE;
    style.visuals = visuals;
    style.spacing.item_spacing = egui::vec2(10.0, 8.0);
    style.spacing.button_padding = egui::vec2(14.0, 8.0);
    ctx.set_style(style);
}

/// The heading of a page.
pub fn title(text: &str) -> RichText {
    RichText::new(text).size(22.0).color(TEXT).strong()
}

/// A caption above a field or a list.
pub fn label(text: impl Into<String>) -> RichText {
    RichText::new(text).size(12.5).color(DIM)
}

/// A number a player reads.
pub fn value(text: impl Into<String>) -> RichText {
    RichText::new(text.into()).size(15.0).color(TEXT).monospace()
}

/// A note the tool made.
pub fn note(text: impl Into<String>, ok: bool) -> RichText {
    RichText::new(text.into())
        .size(12.5)
        .color(if ok { GREEN } else { RED })
}

/// The accent used by what acts.
pub fn action(text: &str) -> RichText {
    RichText::new(text).size(13.5).color(AMBER).strong()
}

/// The icons the pages use: thin, one colour, drawn from lines and circles — the Material way, and
/// no XML file to trust.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Icon {
    Person,
    World,
    Heart,
    Bag,
    Clock,
    Folder,
    Restore,
}

/// Paints `icon` centred on `center`, one colour, `16` points tall.
pub fn paint_icon(painter: &egui::Painter, icon: Icon, center: egui::Pos2, color: Color32) {
    let stroke = egui::Stroke::new(1.6, color);
    let at = |x: f32, y: f32| center + egui::vec2(x, y);
    match icon {
        Icon::Person => {
            painter.circle_stroke(at(0.0, -4.0), 3.0, stroke);
            let arc: Vec<egui::Pos2> = (0..=10)
                .map(|step| {
                    let angle = std::f32::consts::PI * step as f32 / 10.0;
                    at(-6.0 * angle.cos(), 6.0 - 4.0 * angle.sin())
                })
                .collect();
            painter.add(egui::Shape::line(arc, stroke));
        }
        Icon::World => {
            painter.circle_stroke(center, 6.0, stroke);
            painter.line_segment([at(-6.0, 0.0), at(6.0, 0.0)], stroke);
            painter.add(egui::Shape::line(
                (0..=12)
                    .map(|step| {
                        let angle = std::f32::consts::TAU * step as f32 / 12.0;
                        at(2.6 * angle.cos(), 6.0 * angle.sin())
                    })
                    .collect(),
                stroke,
            ));
        }
        Icon::Heart => {
            painter.circle_stroke(at(-2.8, -2.6), 2.9, stroke);
            painter.circle_stroke(at(2.8, -2.6), 2.9, stroke);
            painter.add(egui::Shape::line(
                vec![at(-5.5, -1.2), at(0.0, 6.0), at(5.5, -1.2)],
                stroke,
            ));
        }
        Icon::Bag => {
            painter.rect_stroke(
                egui::Rect::from_min_max(at(-5.5, -2.0), at(5.5, 6.0)),
                2.0,
                stroke,
            );
            painter.add(egui::Shape::line(
                vec![at(-2.5, -2.0), at(-2.5, -5.0), at(2.5, -5.0), at(2.5, -2.0)],
                stroke,
            ));
        }
        Icon::Clock => {
            painter.circle_stroke(center, 6.0, stroke);
            painter.line_segment([center, at(0.0, -3.4)], stroke);
            painter.line_segment([center, at(3.0, 1.0)], stroke);
        }
        Icon::Folder => {
            painter.add(egui::Shape::line(
                vec![
                    at(-6.0, 5.0),
                    at(-6.0, -4.0),
                    at(-1.5, -4.0),
                    at(0.0, -2.0),
                    at(6.0, -2.0),
                    at(6.0, 5.0),
                    at(-6.0, 5.0),
                ],
                stroke,
            ));
        }
        Icon::Restore => {
            painter.circle_stroke(center, 5.6, stroke);
            painter.add(egui::Shape::line(
                vec![at(-5.4, -1.0), at(-5.4, -4.6), at(-1.8, -4.6)],
                stroke,
            ));
        }
    }
}

/// The same, taking its own space in a row of widgets.
pub fn icon(ui: &mut egui::Ui, icon: Icon, color: Color32) {
    let (rect, _) = ui.allocate_exact_size(egui::vec2(16.0, 16.0), egui::Sense::hover());
    paint_icon(ui.painter(), icon, rect.center(), color);
}
