//! Colors, spacing and reusable containers.

use eframe::egui::{self, Color32, CornerRadius, FontId, Stroke, TextStyle};

pub const ACCENT: Color32 = Color32::from_rgb(0, 200, 255);
pub const BG: Color32 = Color32::from_rgb(14, 16, 22);
pub const CARD: Color32 = Color32::from_rgb(24, 27, 36);
pub const CARD_STROKE: Color32 = Color32::from_rgb(38, 42, 54);
pub const MUTED: Color32 = Color32::from_rgb(140, 148, 165);
pub const OK: Color32 = Color32::from_rgb(70, 210, 120);
pub const ERROR: Color32 = Color32::from_rgb(240, 90, 90);
pub const RADIUS: u8 = 10;

pub fn apply(ctx: &egui::Context) {
    ctx.set_theme(egui::Theme::Dark);
    ctx.all_styles_mut(|s| {
        s.spacing.item_spacing = egui::vec2(8.0, 8.0);
        s.spacing.button_padding = egui::vec2(12.0, 6.0);
        s.spacing.slider_width = 260.0;
        for (style, size) in [
            (TextStyle::Heading, 24.0),
            (TextStyle::Body, 15.0),
            (TextStyle::Button, 15.0),
            (TextStyle::Small, 12.0),
        ] {
            s.text_styles.insert(style, FontId::proportional(size));
        }

        let v = &mut s.visuals;
        v.panel_fill = BG;
        v.window_fill = CARD;
        v.extreme_bg_color = Color32::from_rgb(10, 11, 16);
        v.window_corner_radius = CornerRadius::same(RADIUS);
        v.selection.bg_fill = ACCENT.gamma_multiply(0.35);
        v.selection.stroke = Stroke::new(1.0, ACCENT);
        v.slider_trailing_fill = true;

        let w = &mut v.widgets;
        for state in [&mut w.inactive, &mut w.hovered, &mut w.active, &mut w.open] {
            state.corner_radius = CornerRadius::same(RADIUS - 2);
        }
        w.inactive.weak_bg_fill = Color32::from_rgb(34, 38, 50);
        w.inactive.bg_fill = w.inactive.weak_bg_fill;
        w.hovered.weak_bg_fill = Color32::from_rgb(44, 50, 66);
        w.hovered.bg_stroke = Stroke::new(1.0, ACCENT.gamma_multiply(0.6));
        w.active.weak_bg_fill = ACCENT.gamma_multiply(0.5);
        w.active.bg_stroke = Stroke::new(1.0, ACCENT);
    });
}

/// A rounded section with a small uppercase title.
pub fn card(ui: &mut egui::Ui, title: &str, add_contents: impl FnOnce(&mut egui::Ui)) {
    egui::Frame::new()
        .fill(CARD)
        .stroke(Stroke::new(1.0, CARD_STROKE))
        .corner_radius(RADIUS)
        .inner_margin(16)
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.label(
                egui::RichText::new(title.to_uppercase())
                    .small()
                    .strong()
                    .color(MUTED),
            );
            ui.add_space(4.0);
            add_contents(ui);
        });
    ui.add_space(10.0);
}
