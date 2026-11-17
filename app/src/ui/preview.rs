//! On-screen keyboard preview. The animations approximate the firmware effects; they are not exact.

use super::theme::{MUTED, RADIUS};
use crate::config::{Config, Direction, Effect, Rgb};
use eframe::egui::{self, Color32};

const ZONES: usize = 4;
const ZONE_GAP: f32 = 6.0;
const KEY_COLS: u8 = 4;
const KEY_ROWS: u8 = 3;

/// Color of each zone at time `t` (seconds).
pub fn zone_colors(c: &Config, t: f32) -> [Color32; ZONES] {
    let t = t * f32::from(c.speed) / 3.0;
    let dir = if c.direction == Direction::RightToLeft {
        1.0
    } else {
        -1.0
    };
    let base = rgb(c.color);
    let brightness = f32::from(c.brightness) / 100.0;

    std::array::from_fn(|i| {
        let x = f32::from(u8::try_from(i).unwrap_or_default());
        let color = match c.effect {
            Effect::Static => rgb(c.zones[i]),
            Effect::Breath => base.gamma_multiply(0.15 + 0.85 * (0.5 + 0.5 * t.sin())),
            Effect::Neon => hue(t * 0.1),
            Effect::Wave => hue(t * 0.15 + dir * x * 0.15),
            Effect::Shifting => {
                let pos = (t * 0.8 * dir).rem_euclid(4.0);
                let dist = (pos - x).abs().min(4.0 - (pos - x).abs());
                base.gamma_multiply((1.0 - dist / 1.5).clamp(0.08, 1.0))
            }
            Effect::Zoom => {
                let ring = (1.5 - x).abs() / 1.5; // 0 at the center, 1 at the edges
                let wave = (t * 0.8).rem_euclid(1.0);
                base.gamma_multiply((1.0 - (ring - wave).abs() * 2.5).clamp(0.08, 1.0))
            }
        };
        color.gamma_multiply(brightness)
    })
}

/// Draws four glowing zones with key caps.
pub fn show(ui: &mut egui::Ui, colors: [Color32; ZONES]) {
    let (rect, _) = ui.allocate_exact_size(
        egui::vec2(ui.available_width(), 110.0),
        egui::Sense::hover(),
    );
    let painter = ui.painter();
    painter.rect_filled(rect, RADIUS, Color32::from_rgb(8, 9, 13));

    let inner = rect.shrink(12.0);
    let zone_size = egui::vec2((inner.width() - 3.0 * ZONE_GAP) / 4.0, inner.height());
    let mut x = inner.left();
    for (label, color) in (1..).zip(colors) {
        let zone = egui::Rect::from_min_size(egui::pos2(x, inner.top()), zone_size);
        x += zone_size.x + ZONE_GAP;

        for glow in [10.0, 6.0, 3.0] {
            painter.rect_filled(zone.expand(glow), RADIUS, color.gamma_multiply(0.06));
        }
        painter.rect_filled(zone, RADIUS - 4, color.gamma_multiply(0.85));

        let key = egui::vec2(
            zone.width() / f32::from(KEY_COLS),
            zone.height() / f32::from(KEY_ROWS),
        );
        for row in 0..KEY_ROWS {
            for col in 0..KEY_COLS {
                let min = zone.min + egui::vec2(f32::from(col) * key.x, f32::from(row) * key.y);
                painter.rect_filled(
                    egui::Rect::from_min_size(min, key).shrink(3.0),
                    3,
                    Color32::from_black_alpha(110),
                );
            }
        }
        painter.text(
            zone.center_bottom() + egui::vec2(0.0, 12.0),
            egui::Align2::CENTER_CENTER,
            label.to_string(),
            egui::FontId::proportional(11.0),
            MUTED,
        );
    }
}

fn rgb([r, g, b]: Rgb) -> Color32 {
    Color32::from_rgb(r, g, b)
}

fn hue(h: f32) -> Color32 {
    egui::ecolor::Hsva::new(h.rem_euclid(1.0), 1.0, 1.0, 1.0).into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn static_shows_zone_colors_scaled_by_brightness() {
        let c = Config {
            zones: [[255, 0, 0]; 4],
            brightness: 0,
            ..Config::default()
        };
        assert!(zone_colors(&c, 0.0).iter().all(|z| z.r() == 0));
        let c = Config {
            brightness: 100,
            ..c
        };
        assert!(zone_colors(&c, 0.0).iter().all(|z| *z == Color32::RED));
    }
}
