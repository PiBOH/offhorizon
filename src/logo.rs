//! The OffHorizon mark: a dark tile with a violet horizon line and a red fracture across it, drawn
//! from the same 16-unit grid as `brand/offhorizon.svg` so the window icon needs no image file.

use egui::Color32;

/// The tile background.
pub const BACKDROP: Color32 = Color32::from_rgb(0x16, 0x18, 0x1f);
/// The two columns of the mark.
pub const PILLAR: Color32 = Color32::from_rgb(0xec, 0xed, 0xf3);
/// The horizon line.
pub const HORIZON: Color32 = Color32::from_rgb(0x9a, 0x86, 0xff);
/// The fracture.
pub const FRACTURE: Color32 = Color32::from_rgb(0xff, 0x4d, 0x4d);

/// The mark as RGBA pixels, `size` wide and high, for the window and taskbar icon.
pub fn rgba(size: u32) -> Vec<u8> {
    let size = size.max(16);
    let scale = size as f32 / 16.0;
    let mut pixels = vec![0u8; (size * size * 4) as usize];
    let mut plot = |x: f32, y: f32, color: Color32, alpha: f32| {
        let (px, py) = (x.round() as i32, y.round() as i32);
        if px < 0 || py < 0 || px >= size as i32 || py >= size as i32 {
            return;
        }
        let at = ((py as u32 * size + px as u32) * 4) as usize;
        pixels[at] = color.r();
        pixels[at + 1] = color.g();
        pixels[at + 2] = color.b();
        pixels[at + 3] = (255.0 * alpha) as u8;
    };
    // The rounded tile: the corners outside the radius stay empty.
    for y in 0..size {
        for x in 0..size {
            let (cx, cy) = (x as f32 / scale, y as f32 / scale);
            let inside = {
                let r = 3.5;
                let (ix, iy) = (cx.clamp(r, 16.0 - r), cy.clamp(r, 16.0 - r));
                let (dx, dy) = (cx - ix, cy - iy);
                dx * dx + dy * dy <= r * r
            };
            if inside {
                plot(x as f32, y as f32, BACKDROP, 1.0);
            }
        }
    }
    let rect = |x0: f32, y0: f32, w: f32, h: f32, color: Color32, alpha: f32, plot: &mut dyn FnMut(f32, f32, Color32, f32)| {
        let mut y = y0 * scale;
        while y < (y0 + h) * scale {
            let mut x = x0 * scale;
            while x < (x0 + w) * scale {
                plot(x, y, color, alpha);
                x += 1.0;
            }
            y += 1.0;
        }
    };
    rect(4.0, 3.0, 2.0, 4.0, PILLAR, 1.0, &mut plot);
    rect(10.0, 3.0, 2.0, 4.0, PILLAR, 1.0, &mut plot);
    rect(4.0, 8.0, 2.0, 4.0, PILLAR, 0.36, &mut plot);
    rect(10.0, 8.0, 2.0, 4.0, PILLAR, 0.36, &mut plot);
    rect(0.0, 7.0, 16.0, 1.0, HORIZON, 1.0, &mut plot);
    let fracture = [(8.0, -1.0), (9.5, 3.0), (6.5, 6.5), (10.5, 10.0), (7.5, 13.0), (9.0, 17.0)];
    let mut segment = |width: f32, color: Color32, alpha: f32, plot: &mut dyn FnMut(f32, f32, Color32, f32)| {
        for pair in fracture.windows(2) {
            let ((x0, y0), (x1, y1)) = (pair[0], pair[1]);
            let steps = (((x1 - x0) * scale).abs().max(((y1 - y0) * scale).abs()) * 2.0) as i32;
            for step in 0..=steps.max(1) {
                let t = step as f32 / steps.max(1) as f32;
                let (x, y) = (x0 + (x1 - x0) * t, y0 + (y1 - y0) * t);
                let radius = width * scale * 0.5;
                let mut dy = -radius;
                while dy <= radius {
                    let mut dx = -radius;
                    while dx <= radius {
                        if dx * dx + dy * dy <= radius * radius {
                            plot(x * scale + dx, y * scale + dy, color, alpha);
                        }
                        dx += 0.5;
                    }
                    dy += 0.5;
                }
            }
        }
    };
    segment(1.8, FRACTURE, 0.4, &mut plot);
    segment(0.9, FRACTURE, 1.0, &mut plot);
    segment(0.3, Color32::WHITE, 1.0, &mut plot);
    pixels
}

/// The window and taskbar icon.
pub fn icon() -> egui::IconData {
    let size = 64;
    egui::IconData {
        rgba: rgba(size),
        width: size,
        height: size,
    }
}

/// The mark drawn inside the window, `size` points square.
pub fn draw(ui: &mut egui::Ui, size: f32) {
    let (rect, _) = ui.allocate_exact_size(egui::vec2(size, size), egui::Sense::hover());
    let painter = ui.painter();
    let unit = size / 16.0;
    let at = |x: f32, y: f32| rect.min + egui::vec2(x * unit, y * unit);
    painter.rect_filled(rect, size * 0.22, BACKDROP);
    for (x, y, alpha) in [(4.0, 3.0, 1.0), (10.0, 3.0, 1.0), (4.0, 8.0, 0.36), (10.0, 8.0, 0.36)] {
        let color = Color32::from_rgba_unmultiplied(236, 237, 243, (255.0 * alpha) as u8);
        painter.rect_filled(
            egui::Rect::from_min_size(at(x, y), egui::vec2(2.0 * unit, 4.0 * unit)),
            0.0,
            color,
        );
    }
    painter.rect_filled(
        egui::Rect::from_min_size(at(0.0, 7.0), egui::vec2(16.0 * unit, 1.0 * unit)),
        0.0,
        HORIZON,
    );
    let points = [
        (8.0, -1.0),
        (9.5, 3.0),
        (6.5, 6.5),
        (10.5, 10.0),
        (7.5, 13.0),
        (9.0, 17.0),
    ];
    let path: Vec<egui::Pos2> = points.iter().map(|(x, y)| at(*x, *y)).collect();
    painter.add(egui::Shape::line(
        path.clone(),
        egui::Stroke::new(unit * 1.8, Color32::from_rgba_unmultiplied(255, 30, 30, 100)),
    ));
    painter.add(egui::Shape::line(path, egui::Stroke::new(unit * 0.9, FRACTURE)));
}
