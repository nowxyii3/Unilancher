use eframe::egui::{self, vec2, Color32, CornerRadius, Stroke};

// ───────────────────────────── Palette & Typographie Moderne ─────────────────────────────
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
pub const DIM: Color32 = Color32::from_rgb(148, 154, 172);
pub const WHITE: Color32 = Color32::from_rgb(248, 250, 252);
pub const GOLD: Color32 = Color32::from_rgb(245, 158, 11);
pub const GREEN: Color32 = Color32::from_rgb(34, 197, 94);
pub const RED: Color32 = Color32::from_rgb(239, 68, 68);
pub const STEAM_C: Color32 = Color32::from_rgb(102, 192, 244);
pub const EPIC_C: Color32 = Color32::from_rgb(228, 228, 235);
pub const GOG_C: Color32 = Color32::from_rgb(168, 85, 247);
pub const UBI_C: Color32 = Color32::from_rgb(0, 112, 244);
pub const EA_C: Color32 = Color32::from_rgb(239, 68, 68);
pub const BNET_C: Color32 = Color32::from_rgb(0, 174, 255);
pub const XBOX_C: Color32 = Color32::from_rgb(16, 124, 65);
pub const DISCORD_C: Color32 = Color32::from_rgb(88, 101, 242);

pub const ACCENTS: [Color32; 8] = [
    Color32::from_rgb(99, 102, 241),  // Indigo React
    Color32::from_rgb(14, 165, 233),  // Bleu Électrique
    Color32::from_rgb(16, 185, 129),  // Émeraude
    Color32::from_rgb(245, 158, 11),  // Ambre Doré
    Color32::from_rgb(244, 63, 94),   // Rose Néon
    Color32::from_rgb(168, 85, 247),  // Violet Radiant
    Color32::from_rgb(6, 182, 212),   // Cyan Futuriste
    Color32::from_rgb(251, 146, 60),  // Sunset Corail
];

pub const THEME_NAMES: [&str; 8] = [
    "Obsidian Zinc",
    "OLED Pur Noir",
    "Cyberpunk Néon",
    "Midnight Blue",
    "Nord Arctique",
    "Émeraude Forêt",
    "Synthwave 80s",
    "Dracula Deluxe",
];

#[derive(Clone, Copy)]
pub struct ThemeColors {
    pub bg: Color32,
    pub panel: Color32,
    pub panel_border: Color32,
    pub card: Color32,
    pub card_hover: Color32,
    pub card_border: Color32,
    pub badge_bg: Color32,
}

pub fn get_theme(theme_idx: usize) -> ThemeColors {
    match theme_idx {
        1 => ThemeColors {
            // OLED Pur Noir
            bg: Color32::from_rgb(0, 0, 0),
            panel: Color32::from_rgb(5, 5, 7),
            panel_border: Color32::from_rgb(24, 24, 28),
            card: Color32::from_rgb(10, 10, 13),
            card_hover: Color32::from_rgb(20, 20, 25),
            card_border: Color32::from_rgb(34, 34, 42),
            badge_bg: Color32::from_rgb(18, 18, 22),
        },
        2 => ThemeColors {
            // Cyberpunk Néon
            bg: Color32::from_rgb(13, 10, 26),
            panel: Color32::from_rgb(20, 14, 40),
            panel_border: Color32::from_rgb(60, 28, 85),
            card: Color32::from_rgb(28, 18, 54),
            card_hover: Color32::from_rgb(42, 26, 76),
            card_border: Color32::from_rgb(90, 36, 120),
            badge_bg: Color32::from_rgb(38, 24, 68),
        },
        3 => ThemeColors {
            // Midnight Blue
            bg: Color32::from_rgb(10, 15, 26),
            panel: Color32::from_rgb(15, 23, 42),
            panel_border: Color32::from_rgb(30, 45, 75),
            card: Color32::from_rgb(20, 30, 52),
            card_hover: Color32::from_rgb(28, 42, 72),
            card_border: Color32::from_rgb(40, 60, 95),
            badge_bg: Color32::from_rgb(24, 38, 64),
        },
        4 => ThemeColors {
            // Nord Arctique
            bg: Color32::from_rgb(15, 18, 24),
            panel: Color32::from_rgb(20, 25, 34),
            panel_border: Color32::from_rgb(38, 46, 62),
            card: Color32::from_rgb(26, 33, 45),
            card_hover: Color32::from_rgb(34, 43, 58),
            card_border: Color32::from_rgb(52, 64, 88),
            badge_bg: Color32::from_rgb(30, 38, 52),
        },
        5 => ThemeColors {
            // Émeraude Forêt
            bg: Color32::from_rgb(7, 18, 14),
            panel: Color32::from_rgb(11, 26, 20),
            panel_border: Color32::from_rgb(20, 48, 38),
            card: Color32::from_rgb(15, 36, 28),
            card_hover: Color32::from_rgb(22, 52, 40),
            card_border: Color32::from_rgb(32, 74, 58),
            badge_bg: Color32::from_rgb(18, 42, 32),
        },
        6 => ThemeColors {
            // Synthwave 80s
            bg: Color32::from_rgb(18, 8, 28),
            panel: Color32::from_rgb(26, 12, 40),
            panel_border: Color32::from_rgb(72, 26, 95),
            card: Color32::from_rgb(36, 16, 56),
            card_hover: Color32::from_rgb(52, 22, 80),
            card_border: Color32::from_rgb(110, 35, 130),
            badge_bg: Color32::from_rgb(48, 20, 72),
        },
        7 => ThemeColors {
            // Dracula Deluxe
            bg: Color32::from_rgb(18, 19, 26),
            panel: Color32::from_rgb(24, 25, 36),
            panel_border: Color32::from_rgb(48, 50, 70),
            card: Color32::from_rgb(32, 34, 48),
            card_hover: Color32::from_rgb(44, 46, 66),
            card_border: Color32::from_rgb(64, 66, 94),
            badge_bg: Color32::from_rgb(38, 40, 56),
        },
        _ => ThemeColors {
            // Obsidian / Zinc (Défaut)
            bg: Color32::from_rgb(11, 12, 15),
            panel: Color32::from_rgb(16, 17, 22),
            panel_border: Color32::from_rgb(28, 30, 38),
            card: Color32::from_rgb(20, 22, 29),
            card_hover: Color32::from_rgb(27, 30, 40),
            card_border: Color32::from_rgb(36, 39, 50),
            badge_bg: Color32::from_rgb(25, 27, 36),
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

    let r = CornerRadius::same(10);
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
    v.widgets.hovered.bg_stroke = Stroke::new(1.0, accent.gamma_multiply(0.7));
    v.widgets.active.bg_fill = accent.gamma_multiply(0.6);
    v.widgets.active.weak_bg_fill = accent.gamma_multiply(0.6);
    ctx.set_visuals(v);

    ctx.all_styles_mut(|st| {
        st.spacing.item_spacing = vec2(8.0, 8.0);
        st.spacing.button_padding = vec2(12.0, 7.0);
        st.animation_time = 0.18;
    });
}
