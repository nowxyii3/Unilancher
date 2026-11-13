use eframe::egui::{self, vec2, Color32, CornerRadius, Stroke};

// ───────────────────────────── Palette Zinc / Obsidian (Sobre & Moderne) ─────────────────────────────
#[allow(dead_code)]
pub const BG: Color32 = Color32::from_rgb(11, 12, 15);
pub const PANEL: Color32 = Color32::from_rgb(16, 17, 22);
#[allow(dead_code)]
pub const PANEL_BORDER: Color32 = Color32::from_rgb(28, 30, 38);
#[allow(dead_code)]
pub const CARD: Color32 = Color32::from_rgb(20, 22, 29);
#[allow(dead_code)]
pub const CARD_HOVER: Color32 = Color32::from_rgb(27, 30, 40);
#[allow(dead_code)]
pub const CARD_BORDER: Color32 = Color32::from_rgb(36, 39, 50);
pub const DIM: Color32 = Color32::from_rgb(138, 144, 160);
pub const WHITE: Color32 = Color32::from_rgb(242, 244, 248);
pub const GOLD: Color32 = Color32::from_rgb(234, 179, 8);
pub const GREEN: Color32 = Color32::from_rgb(34, 197, 94);
pub const RED: Color32 = Color32::from_rgb(239, 68, 68);
pub const STEAM_C: Color32 = Color32::from_rgb(102, 192, 244);
pub const EPIC_C: Color32 = Color32::from_rgb(228, 228, 235);
pub const GOG_C: Color32 = Color32::from_rgb(168, 85, 247);
pub const UBI_C: Color32 = Color32::from_rgb(0, 112, 244);
pub const EA_C: Color32 = Color32::from_rgb(239, 68, 68);
pub const XBOX_C: Color32 = Color32::from_rgb(16, 124, 65);
pub const DISCORD_C: Color32 = Color32::from_rgb(88, 101, 242);

pub const ACCENTS: [Color32; 5] = [
    Color32::from_rgb(99, 102, 241),  // Indigo
    Color32::from_rgb(59, 130, 246),  // Bleu Acier
    Color32::from_rgb(16, 185, 129),  // Émeraude
    Color32::from_rgb(245, 158, 11),  // Ambre
    Color32::from_rgb(236, 72, 153),  // Rose Carmin
];

#[derive(Clone, Copy)]
pub struct ThemeColors {
    pub bg: Color32,
    pub panel: Color32,
    pub panel_border: Color32,
    pub card: Color32,
    pub card_hover: Color32,
    pub card_border: Color32,
}

pub fn get_theme(theme_idx: usize) -> ThemeColors {
    match theme_idx {
        1 => ThemeColors {
            // OLED Pur Noir
            bg: Color32::from_rgb(0, 0, 0),
            panel: Color32::from_rgb(6, 6, 8),
            panel_border: Color32::from_rgb(22, 22, 26),
            card: Color32::from_rgb(10, 10, 12),
            card_hover: Color32::from_rgb(18, 18, 22),
            card_border: Color32::from_rgb(32, 32, 38),
        },
        2 => ThemeColors {
            // Cyberpunk Néon
            bg: Color32::from_rgb(13, 10, 26),
            panel: Color32::from_rgb(20, 14, 40),
            panel_border: Color32::from_rgb(60, 28, 85),
            card: Color32::from_rgb(28, 18, 54),
            card_hover: Color32::from_rgb(42, 26, 76),
            card_border: Color32::from_rgb(90, 36, 120),
        },
        3 => ThemeColors {
            // Midnight Blue
            bg: Color32::from_rgb(10, 15, 26),
            panel: Color32::from_rgb(15, 23, 42),
            panel_border: Color32::from_rgb(30, 45, 75),
            card: Color32::from_rgb(20, 30, 52),
            card_hover: Color32::from_rgb(28, 42, 72),
            card_border: Color32::from_rgb(40, 60, 95),
        },
        _ => ThemeColors {
            // Obsidian / Zinc (Défaut)
            bg: Color32::from_rgb(11, 12, 15),
            panel: Color32::from_rgb(16, 17, 22),
            panel_border: Color32::from_rgb(28, 30, 38),
            card: Color32::from_rgb(20, 22, 29),
            card_hover: Color32::from_rgb(27, 30, 40),
            card_border: Color32::from_rgb(36, 39, 50),
        },
    }
}

pub fn apply_style(ctx: &egui::Context, accent: Color32, theme_idx: usize) {
    let th = get_theme(theme_idx);
    let mut v = egui::Visuals::dark();
    v.panel_fill = th.bg;
    v.window_fill = th.card;
    v.window_stroke = Stroke::new(1.0, th.card_border);
    v.extreme_bg_color = th.panel;
    v.faint_bg_color = th.card;
    v.selection.bg_fill = accent.gamma_multiply(0.35);
    v.selection.stroke = Stroke::new(1.0, accent);

    let r = CornerRadius::same(8);
    for w in [
        &mut v.widgets.noninteractive,
        &mut v.widgets.inactive,
        &mut v.widgets.hovered,
        &mut v.widgets.active,
        &mut v.widgets.open,
    ] {
        w.corner_radius = r;
    }
    v.widgets.noninteractive.bg_stroke = Stroke::new(1.0, th.card_border);
    v.widgets.inactive.bg_fill = th.card;
    v.widgets.inactive.weak_bg_fill = th.card;
    v.widgets.inactive.bg_stroke = Stroke::new(1.0, th.card_border);
    v.widgets.hovered.bg_fill = th.card_hover;
    v.widgets.hovered.weak_bg_fill = th.card_hover;
    v.widgets.hovered.bg_stroke = Stroke::new(1.0, accent.gamma_multiply(0.6));
    v.widgets.active.bg_fill = accent.gamma_multiply(0.55);
    v.widgets.active.weak_bg_fill = accent.gamma_multiply(0.55);
    ctx.set_visuals(v);

    ctx.all_styles_mut(|st| {
        st.spacing.item_spacing = vec2(8.0, 8.0);
        st.spacing.button_padding = vec2(12.0, 7.0);
        st.animation_time = 0.16;
    });
}
