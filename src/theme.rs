//! The look: the official Horizon launcher's "Glass" (crates/horizon-theme/src/glass.rs) — plates
//! of frosted white over a near-black page, white text at a few opacities, round corners and one
//! accent. OffHorizon keeps its own accent and its fracture red, and draws its icons in code, so
//! no image or font file is needed.

use egui::{Color32, Response, RichText, Sense, Style, Visuals};

/// The page behind the plates (`glass::PAGE`).
pub const BG: Color32 = Color32::from_rgb(0x0a, 0x09, 0x19);
/// A plate: white at 6 % over the accent-tinted base (`Glass::plate`), for our accent.
pub const PANEL: Color32 = Color32::from_rgb(0x1e, 0x1c, 0x2c);
/// A plate a step up: hovered, or the one chosen (white at 10 %).
pub const PANEL_HI: Color32 = Color32::from_rgb(0x27, 0x26, 0x35);
/// The 1 px line on a plate's edge.
pub const LINE: Color32 = Color32::from_rgb(0x31, 0x2f, 0x3e);
/// The ink: white.
pub const TEXT: Color32 = Color32::from_rgb(0xff, 0xff, 0xff);
/// White at 65 %, a row's second line (`glass::ink::MUTED`).
pub const DIM: Color32 = Color32::from_rgb(0xa4, 0xa2, 0xb0);
/// White at 45 % (`glass::ink::DIM`), for what is read once.
pub const FAINT: Color32 = Color32::from_rgb(0x74, 0x72, 0x80);
/// OffHorizon's accent, the logo's violet.
pub const ACCENT: Color32 = Color32::from_rgb(0x9a, 0x86, 0xff);
/// The accent mixed half with white (`accent_soft`): links, ticks, what is hovered.
pub const ACCENT_SOFT: Color32 = Color32::from_rgb(0xce, 0xc4, 0xff);
/// The text on the accent: the dark violet of a light accent (`on_accent`).
pub const ON_ACCENT: Color32 = Color32::from_rgb(0x10, 0x0a, 0x1f);
/// A ready instance, a game that is running (`glass::LIVE`).
pub const GREEN: Color32 = Color32::from_rgb(0x4a, 0xde, 0x80);
/// Busy, or only partly working (`glass::WARN`).
pub const WARN: Color32 = Color32::from_rgb(0xf2, 0xc4, 0x6b);
/// What went wrong (`glass::DANGER`).
pub const RED: Color32 = Color32::from_rgb(0xff, 0x8a, 0x8a);

/// The official corners, in points (`glass::radius`).
pub const RADIUS_MODAL: f32 = 28.0;
pub const RADIUS_PANEL: f32 = 20.0;
pub const RADIUS_ROW: f32 = 12.0;
pub const RADIUS_CONTROL: f32 = 10.0;
/// The official sizes: the sidebar, the top bar, the page's inset, Play.
pub const SIDEBAR: f32 = 248.0;
pub const TOP_BAR: f32 = 52.0;
pub const INSET: f32 = 24.0;
pub const PLAY_HEIGHT: f32 = 56.0;

/// Applies the look to a context.
pub fn apply(ctx: &egui::Context) {
    let mut style: Style = (*ctx.style()).clone();
    let mut visuals = Visuals::dark();
    visuals.panel_fill = BG;
    visuals.window_fill = PANEL;
    visuals.window_rounding = RADIUS_MODAL.into();
    visuals.extreme_bg_color = Color32::from_rgb(0x12, 0x10, 0x21);
    visuals.faint_bg_color = PANEL_HI;
    visuals.widgets.noninteractive.bg_fill = PANEL;
    visuals.widgets.noninteractive.fg_stroke.color = TEXT;
    visuals.widgets.inactive.bg_fill = PANEL_HI;
    visuals.widgets.inactive.fg_stroke.color = TEXT;
    visuals.widgets.inactive.rounding = RADIUS_CONTROL.into();
    visuals.widgets.hovered.bg_fill = PANEL_HI;
    visuals.widgets.hovered.fg_stroke.color = ACCENT_SOFT;
    visuals.widgets.hovered.rounding = RADIUS_CONTROL.into();
    visuals.widgets.active.bg_fill = PANEL_HI;
    visuals.widgets.active.fg_stroke.color = ACCENT_SOFT;
    visuals.widgets.active.rounding = RADIUS_CONTROL.into();
    visuals.selection.bg_fill = ACCENT.linear_multiply(0.35);
    visuals.selection.stroke.color = ACCENT;
    visuals.window_stroke.color = LINE;
    style.visuals = visuals;
    style.spacing.item_spacing = egui::vec2(10.0, 8.0);
    style.spacing.button_padding = egui::vec2(14.0, 8.0);
    ctx.set_style(style);
}

/// A plate: the panel every page group sits on, with the official panel corners.
pub fn plate() -> egui::Frame {
    egui::Frame::none()
        .fill(PANEL)
        .rounding(RADIUS_PANEL)
        .inner_margin(egui::Margin::same(16.0))
}

/// A page's head: its name at 30, its description under it in the muted ink, then room below.
pub fn page_head(ui: &mut egui::Ui, title: &str, description: &str) {
    ui.label(RichText::new(title).size(30.0).color(TEXT).strong());
    if !description.is_empty() {
        ui.add_space(4.0);
        ui.label(RichText::new(description).size(13.5).color(DIM));
    }
    ui.add_space(18.0);
}

/// The heading of a group inside a page ("Instances", "Versions").
pub fn group_title(ui: &mut egui::Ui, label: &str) {
    ui.label(RichText::new(label).size(13.0).color(DIM).strong());
    ui.add_space(4.0);
}

/// The heading of a page (the older pages' name for [`page_head`]).
pub fn title(text: &str) -> RichText {
    RichText::new(text).size(26.0).color(TEXT).strong()
}

/// A caption above a field or a list.
pub fn label(text: impl Into<String>) -> RichText {
    RichText::new(text).size(13.0).color(DIM)
}

/// A number a player reads.
pub fn value(text: impl Into<String>) -> RichText {
    RichText::new(text.into()).size(14.5).color(TEXT)
}

/// A note the tool made.
pub fn note(text: impl Into<String>, ok: bool) -> RichText {
    RichText::new(text.into())
        .size(13.0)
        .color(if ok { GREEN } else { RED })
}

/// The accent used by what acts.
pub fn action(text: &str) -> RichText {
    RichText::new(text).size(13.5).color(ACCENT).strong()
}

/// A row of a page's list: its name, a second line, painted on a plate when it is the one chosen.
pub fn list_row(ui: &mut egui::Ui, name: &str, sub: &str, active: bool) -> Response {
    let width = ui.available_width();
    let (rect, response) = ui.allocate_exact_size(egui::vec2(width, 54.0), Sense::click());
    let fill = if active {
        Some(PANEL_HI)
    } else if response.hovered() {
        Some(PANEL)
    } else {
        None
    };
    if let Some(fill) = fill {
        ui.painter().rect_filled(rect, RADIUS_ROW, fill);
    }
    let painter = ui.painter();
    painter.text(
        rect.left_center() + egui::vec2(14.0, -9.0),
        egui::Align2::LEFT_CENTER,
        name,
        egui::FontId::proportional(15.0),
        TEXT,
    );
    painter.text(
        rect.left_center() + egui::vec2(14.0, 10.0),
        egui::Align2::LEFT_CENTER,
        sub,
        egui::FontId::proportional(12.0),
        DIM,
    );
    response.on_hover_cursor(egui::CursorIcon::PointingHand)
}

/// A row of a settings plate: the name on the left, the control on the right.
pub fn field_row(ui: &mut egui::Ui, label: &str, control: impl FnOnce(&mut egui::Ui)) {
    ui.horizontal(|ui| {
        ui.set_min_height(34.0);
        ui.label(RichText::new(label).size(14.0).color(TEXT).strong());
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), control);
    });
}

/// A page's tab in the top bar: its name, on a plate when it is the page shown.
pub fn tab(ui: &mut egui::Ui, label: &str, active: bool) -> Response {
    let width = 12.0 + ui
        .painter()
        .layout_no_wrap(label.to_string(), egui::FontId::proportional(14.0), TEXT)
        .size()
        .x
        + 12.0;
    let (rect, response) = ui.allocate_exact_size(egui::vec2(width, 34.0), Sense::click());
    let fill = if active {
        Some(PANEL_HI)
    } else if response.hovered() {
        Some(PANEL)
    } else {
        None
    };
    if let Some(fill) = fill {
        ui.painter().rect_filled(rect, RADIUS_CONTROL, fill);
    }
    ui.painter().text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        label,
        egui::FontId::proportional(14.0),
        if active { TEXT } else { DIM },
    );
    response.on_hover_cursor(egui::CursorIcon::PointingHand)
}

/// A square button with one icon on a plate (the sidebar's: play the instance, add one).
pub fn icon_button(ui: &mut egui::Ui, icon: Icon, color: Color32) -> Response {
    let (rect, response) = ui.allocate_exact_size(egui::vec2(26.0, 26.0), Sense::click());
    if response.hovered() {
        ui.painter().rect_filled(rect, 8.0, PANEL_HI);
    }
    paint_icon(ui.painter(), icon, rect.center(), color);
    response.on_hover_cursor(egui::CursorIcon::PointingHand)
}

/// A sidebar row, from the official launcher: a tile with the instance's mark, its name and its
/// state, on a plate when it is the one picked.
pub fn sidebar_row(ui: &mut egui::Ui, name: &str, sub: &str, current: bool) -> Response {
    let width = ui.available_width();
    let (rect, response) = ui.allocate_exact_size(egui::vec2(width, 52.0), Sense::click());
    let fill = if current {
        Some(PANEL_HI)
    } else if response.hovered() {
        Some(PANEL)
    } else {
        None
    };
    if let Some(fill) = fill {
        ui.painter().rect_filled(rect, RADIUS_ROW, fill);
    }
    let painter = ui.painter();
    let tile = egui::Rect::from_center_size(
        egui::pos2(rect.left() + 24.0, rect.center().y),
        egui::vec2(32.0, 32.0),
    );
    painter.rect_filled(
        tile,
        9.0,
        if current {
            ACCENT
        } else {
            Color32::from_rgba_unmultiplied(0x9a, 0x86, 0xff, 60)
        },
    );
    paint_icon(
        painter,
        Icon::Play,
        tile.center(),
        if current { ON_ACCENT } else { ACCENT_SOFT },
    );
    painter.text(
        egui::pos2(tile.right() + 12.0, rect.center().y - 9.0),
        egui::Align2::LEFT_CENTER,
        name,
        egui::FontId::proportional(14.0),
        TEXT,
    );
    painter.circle_filled(
        egui::pos2(tile.right() + 15.0, rect.center().y + 10.0),
        2.8,
        GREEN,
    );
    painter.text(
        egui::pos2(tile.right() + 21.0, rect.center().y + 10.0),
        egui::Align2::LEFT_CENTER,
        sub,
        egui::FontId::proportional(12.0),
        DIM,
    );
    response.on_hover_cursor(egui::CursorIcon::PointingHand)
}

/// The main action, from the official launcher: 56 tall, the accent, the triangle and the label.
pub fn play_button(ui: &mut egui::Ui, label: &str) -> Response {
    let width = ui.available_width().min(320.0);
    let (rect, response) = ui.allocate_exact_size(
        egui::vec2(width, PLAY_HEIGHT),
        Sense::click(),
    );
    let fill = if response.is_pointer_button_down_on() {
        ACCENT_SOFT
    } else if response.hovered() {
        ACCENT_SOFT
    } else {
        ACCENT
    };
    let painter = ui.painter();
    painter.rect_filled(rect, 14.0, fill);
    paint_icon(
        painter,
        Icon::Play,
        rect.left_center() + egui::vec2(30.0, 0.0),
        ON_ACCENT,
    );
    painter.text(
        rect.left_center() + egui::vec2(52.0, 0.0),
        egui::Align2::LEFT_CENTER,
        label,
        egui::FontId::proportional(18.0),
        ON_ACCENT,
    );
    response.on_hover_cursor(egui::CursorIcon::PointingHand)
}

/// The icons the pages use: thin, one colour, drawn from lines and circles — the Material way, and
/// no file to trust.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Icon {
    Home,
    Play,
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
        Icon::Home => {
            painter.add(egui::Shape::line(
                vec![at(-6.2, 0.0), at(0.0, -6.0), at(6.2, 0.0)],
                stroke,
            ));
            painter.rect_stroke(
                egui::Rect::from_min_max(at(-4.6, 0.0), at(4.6, 6.0)),
                1.0,
                stroke,
            );
        }
        Icon::Play => {
            painter.add(egui::Shape::convex_polygon(
                vec![at(-3.4, -6.0), at(6.0, 0.0), at(-3.4, 6.0)],
                color,
                egui::Stroke::NONE,
            ));
        }
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
