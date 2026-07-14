use egui::{Color32, Rounding, Stroke, vec2};

pub const BG_BASE: Color32 = Color32::from_rgb(15, 17, 21);
pub const BG_PANEL: Color32 = Color32::from_rgb(21, 24, 33);
pub const BG_CARD: Color32 = Color32::from_rgb(29, 33, 44);
pub const BG_HOVER: Color32 = Color32::from_rgb(36, 41, 54);

pub const ACCENT: Color32 = Color32::from_rgb(91, 140, 255);

pub const GREEN: Color32 = Color32::from_rgb(39, 196, 107);
pub const YELLOW: Color32 = Color32::from_rgb(247, 183, 49);
pub const RED: Color32 = Color32::from_rgb(255, 93, 93);

pub const TEXT: Color32 = Color32::from_rgb(245, 246, 248);
pub const TEXT_DIM: Color32 = Color32::from_rgb(183, 189, 200);
pub const TEXT_MUTED: Color32 = Color32::from_rgb(115, 122, 134);

pub const ROUND_CARD: Rounding = Rounding::same(8.0);

pub fn setup_style(ctx: &egui::Context) {
    let mut style = (*ctx.style()).clone();
    style.visuals.dark_mode = true;
    style.visuals.window_fill = BG_PANEL;
    style.visuals.panel_fill = BG_PANEL;
    style.visuals.extreme_bg_color = BG_BASE;
    style.visuals.window_rounding = ROUND_CARD;

    style.visuals.widgets.noninteractive.bg_fill = BG_PANEL;
    style.visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, TEXT_DIM);
    style.visuals.widgets.inactive.bg_fill = BG_CARD;
    style.visuals.widgets.hovered.bg_fill = BG_HOVER;
    style.visuals.widgets.active.bg_fill = ACCENT;
    style.visuals.widgets.active.fg_stroke = Stroke::new(1.0, TEXT);

    style.visuals.selection.bg_fill = ACCENT.linear_multiply(0.25);
    style.visuals.selection.stroke = Stroke::new(1.0, ACCENT);

    style.visuals.override_text_color = Some(TEXT);
    style.visuals.hyperlink_color = ACCENT;

    style.spacing.item_spacing = vec2(8.0, 8.0);
    style.spacing.window_margin = egui::Margin::symmetric(16.0, 16.0);
    style.spacing.button_padding = vec2(8.0, 4.0);
    style.spacing.indent = 16.0;

    ctx.set_style(style);
}
