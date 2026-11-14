use eframe::egui::{self, vec2, Color32, CornerRadius, Stroke};

// ═══════════════════════════════════════════════════════════════════════════════
//  UniLauncher — Premium Design System
//  Inspired by Linear, Discord, Arc Browser, Notion, Raycast
// ═══════════════════════════════════════════════════════════════════════════════

// ─────────────────────────── Surface Hierarchy ───────────────────────────────
// Deep anthracite base layers — never pure black, always with subtle warmth.
#[allow(dead_code)]
pub const BG: Color32 = Color32::from_rgb(15, 17, 23);         // #0F1117  — deepest background
pub const PANEL: Color32 = Color32::from_rgb(19, 20, 26);      // #13141A  — sidebar / nav panels
#[allow(dead_code)]
pub const PANEL_BORDER: Color32 = Color32::from_rgb(42, 45, 58);// #2A2D3A — panel dividers
#[allow(dead_code)]
pub const CARD: Color32 = Color32::from_rgb(30, 32, 41);       // #1E2029  — card / elevated surface
#[allow(dead_code)]
pub const CARD_HOVER: Color32 = Color32::from_rgb(36, 38, 49); // #242631  — card hover state
#[allow(dead_code)]
pub const CARD_BORDER: Color32 = Color32::from_rgb(50, 54, 74);// #32364A  — card outlines

/// Layered surface tokens — use higher numbers for more elevated UI.
#[allow(dead_code)]
pub const SURFACE_0: Color32 = Color32::from_rgb(15, 17, 23);  // #0F1117  — app background
#[allow(dead_code)]
pub const SURFACE_1: Color32 = Color32::from_rgb(19, 20, 26);  // #13141A  — panels, sidebars
#[allow(dead_code)]
pub const SURFACE_2: Color32 = Color32::from_rgb(30, 32, 41);  // #1E2029  — cards, dropdowns
#[allow(dead_code)]
pub const SURFACE_3: Color32 = Color32::from_rgb(36, 38, 49);  // #242631  — tooltips, popovers

// ─────────────────────────── Text Hierarchy ──────────────────────────────────
pub const WHITE: Color32 = Color32::from_rgb(248, 250, 252);   // #F8FAFC  — primary text
pub const DIM: Color32 = Color32::from_rgb(148, 163, 184);     // #94A3B8  — secondary text

/// Additional text tokens.
#[allow(dead_code)]
pub const TEXT_PRIMARY: Color32 = Color32::from_rgb(248, 250, 252);   // #F8FAFC
#[allow(dead_code)]
pub const TEXT_SECONDARY: Color32 = Color32::from_rgb(148, 163, 184); // #94A3B8
#[allow(dead_code)]
pub const TEXT_TERTIARY: Color32 = Color32::from_rgb(100, 116, 139);  // #64748B

// ─────────────────────────── Border States ───────────────────────────────────
#[allow(dead_code)]
pub const BORDER_SUBTLE: Color32 = Color32::from_rgb(38, 41, 54);    // #26293A — barely visible
#[allow(dead_code)]
pub const BORDER_DEFAULT: Color32 = Color32::from_rgb(50, 54, 74);   // #32364A — default
#[allow(dead_code)]
pub const BORDER_HOVER: Color32 = Color32::from_rgb(71, 76, 102);    // #474C66 — hover

// ─────────────────────────── Semantic Colors ─────────────────────────────────
pub const GREEN: Color32 = Color32::from_rgb(52, 211, 153);    // #34D399  — softer emerald
pub const RED: Color32 = Color32::from_rgb(248, 113, 113);     // #F87171  — softer rose
pub const GOLD: Color32 = Color32::from_rgb(251, 191, 36);     // #FBBF24  — warm amber

#[allow(dead_code)]
pub const SUCCESS: Color32 = Color32::from_rgb(52, 211, 153);  // #34D399
#[allow(dead_code)]
pub const WARNING: Color32 = Color32::from_rgb(251, 191, 36);  // #FBBF24
#[allow(dead_code)]
pub const ERROR: Color32 = Color32::from_rgb(248, 113, 113);   // #F87171
#[allow(dead_code)]
pub const INFO: Color32 = Color32::from_rgb(96, 165, 250);     // #60A5FA

// ─────────────────────────── Accent Glow ─────────────────────────────────────
#[allow(dead_code)]
pub const ACCENT_GLOW: Color32 = Color32::from_rgb(99, 102, 241); // #6366F1 with alpha applied at use site

// ─────────────────────────── Platform Colors (softer, harmonious) ────────────
pub const STEAM_C: Color32 = Color32::from_rgb(110, 194, 244);  // Soft sky blue
pub const EPIC_C: Color32 = Color32::from_rgb(210, 212, 220);   // Muted pearl
pub const GOG_C: Color32 = Color32::from_rgb(167, 139, 250);    // Lavender violet
pub const UBI_C: Color32 = Color32::from_rgb(59, 130, 246);     // Clean blue
pub const EA_C: Color32 = Color32::from_rgb(248, 113, 113);     // Rose red
pub const BNET_C: Color32 = Color32::from_rgb(56, 189, 248);    // Cyan blue
pub const XBOX_C: Color32 = Color32::from_rgb(74, 222, 128);    // Mint green
pub const DISCORD_C: Color32 = Color32::from_rgb(114, 137, 218);// Soft periwinkle

// ─────────────────────────── Accent Palette (Linear-inspired, desaturated) ───
pub const ACCENTS: [Color32; 8] = [
    Color32::from_rgb(99, 102, 241),   // Indigo — refined, Linear-like
    Color32::from_rgb(56, 189, 248),   // Sky — clean, airy
    Color32::from_rgb(52, 211, 153),   // Emerald — calm, sophisticated
    Color32::from_rgb(251, 191, 36),   // Amber — warm, premium gold
    Color32::from_rgb(251, 113, 133),  // Rose — soft, elegant pink
    Color32::from_rgb(167, 139, 250),  // Violet — muted, rich purple
    Color32::from_rgb(34, 211, 238),   // Cyan — futuristic, refined
    Color32::from_rgb(251, 146, 60),   // Tangerine — warm sunset
];

// ─────────────────────────── Theme Definitions ───────────────────────────────
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

/// Theme surface & border colours.
///
/// Fields `surface_*` provide a finer 4-level elevation ramp inside each theme
/// while the original fields (`bg`, `panel`, `card`, …) remain for backward
/// compatibility.
#[derive(Clone, Copy)]
#[allow(dead_code)]
pub struct ThemeColors {
    // ── Legacy fields (public API — do not remove) ──
    pub bg: Color32,
    pub panel: Color32,
    pub panel_border: Color32,
    pub card: Color32,
    pub card_hover: Color32,
    pub card_border: Color32,
    pub badge_bg: Color32,

    // ── Extended surface hierarchy ──
    pub surface_0: Color32,
    pub surface_1: Color32,
    pub surface_2: Color32,
    pub surface_3: Color32,

    // ── Extended border states ──
    pub border_subtle: Color32,
    pub border_default: Color32,
    pub border_hover: Color32,
}

// Helper: build ThemeColors from compact tuples
impl ThemeColors {
    #[allow(clippy::too_many_arguments)]
    const fn new(
        bg: Color32,
        panel: Color32,
        panel_border: Color32,
        card: Color32,
        card_hover: Color32,
        card_border: Color32,
        badge_bg: Color32,
        border_subtle: Color32,
        border_hover: Color32,
    ) -> Self {
        Self {
            bg,
            panel,
            panel_border,
            card,
            card_hover,
            card_border,
            badge_bg,
            surface_0: bg,
            surface_1: panel,
            surface_2: card,
            surface_3: card_hover,
            border_subtle,
            border_default: panel_border,
            border_hover,
        }
    }
}

pub fn get_theme(theme_idx: usize) -> ThemeColors {
    match theme_idx {
        // ───── 0: Obsidian Zinc (Default) ─────
        // Deep anthracite with cool zinc undertones — the Linear / Raycast vibe.
        0 => ThemeColors::new(
            Color32::from_rgb(15, 17, 23),   // bg
            Color32::from_rgb(19, 20, 26),   // panel
            Color32::from_rgb(42, 45, 58),   // panel_border
            Color32::from_rgb(30, 32, 41),   // card
            Color32::from_rgb(36, 38, 49),   // card_hover
            Color32::from_rgb(50, 54, 74),   // card_border
            Color32::from_rgb(25, 27, 36),   // badge_bg
            Color32::from_rgb(34, 37, 50),   // border_subtle
            Color32::from_rgb(62, 66, 88),   // border_hover
        ),

        // ───── 1: OLED Pur Noir ─────
        // True-black OLED-optimised — minimal surface differentiation.
        1 => ThemeColors::new(
            Color32::from_rgb(0, 0, 0),
            Color32::from_rgb(8, 8, 10),
            Color32::from_rgb(28, 28, 34),
            Color32::from_rgb(14, 14, 18),
            Color32::from_rgb(22, 22, 28),
            Color32::from_rgb(38, 38, 48),
            Color32::from_rgb(18, 18, 22),
            Color32::from_rgb(22, 22, 28),
            Color32::from_rgb(48, 48, 58),
        ),

        // ───── 2: Cyberpunk Néon ─────
        // Electric violet tint — neon glow feel without being garish.
        2 => ThemeColors::new(
            Color32::from_rgb(12, 10, 22),
            Color32::from_rgb(18, 15, 32),
            Color32::from_rgb(48, 30, 72),
            Color32::from_rgb(24, 20, 42),
            Color32::from_rgb(34, 28, 58),
            Color32::from_rgb(64, 40, 92),
            Color32::from_rgb(30, 24, 52),
            Color32::from_rgb(40, 28, 62),
            Color32::from_rgb(78, 48, 108),
        ),

        // ───── 3: Midnight Blue ─────
        // Deep ocean-navy — Discord's darker cousin, elegant and calm.
        3 => ThemeColors::new(
            Color32::from_rgb(10, 14, 24),
            Color32::from_rgb(14, 20, 36),
            Color32::from_rgb(30, 42, 68),
            Color32::from_rgb(20, 28, 48),
            Color32::from_rgb(28, 38, 62),
            Color32::from_rgb(40, 56, 88),
            Color32::from_rgb(22, 32, 56),
            Color32::from_rgb(26, 36, 58),
            Color32::from_rgb(48, 64, 100),
        ),

        // ───── 4: Nord Arctique ─────
        // Polar frost — the Nordic palette distilled into dark surfaces.
        4 => ThemeColors::new(
            Color32::from_rgb(14, 18, 26),
            Color32::from_rgb(20, 26, 36),
            Color32::from_rgb(40, 50, 66),
            Color32::from_rgb(28, 36, 48),
            Color32::from_rgb(36, 46, 60),
            Color32::from_rgb(54, 68, 90),
            Color32::from_rgb(32, 40, 54),
            Color32::from_rgb(36, 44, 60),
            Color32::from_rgb(60, 74, 98),
        ),

        // ───── 5: Émeraude Forêt ─────
        // Deep forest — lush greens that stay muted and premium.
        5 => ThemeColors::new(
            Color32::from_rgb(8, 16, 14),
            Color32::from_rgb(12, 22, 20),
            Color32::from_rgb(24, 46, 40),
            Color32::from_rgb(18, 32, 28),
            Color32::from_rgb(26, 44, 38),
            Color32::from_rgb(36, 64, 54),
            Color32::from_rgb(20, 36, 30),
            Color32::from_rgb(22, 40, 34),
            Color32::from_rgb(40, 72, 60),
        ),

        // ───── 6: Synthwave 80s ─────
        // Retrowave magenta — rich purples without overwhelming saturation.
        6 => ThemeColors::new(
            Color32::from_rgb(16, 10, 24),
            Color32::from_rgb(22, 14, 34),
            Color32::from_rgb(56, 30, 78),
            Color32::from_rgb(30, 18, 48),
            Color32::from_rgb(42, 26, 66),
            Color32::from_rgb(80, 40, 108),
            Color32::from_rgb(36, 22, 56),
            Color32::from_rgb(46, 28, 66),
            Color32::from_rgb(92, 48, 124),
        ),

        // ───── 7: Dracula Deluxe ─────
        // Dracula-inspired — balanced warm-grey purples, refined.
        7 => ThemeColors::new(
            Color32::from_rgb(18, 19, 28),
            Color32::from_rgb(24, 26, 38),
            Color32::from_rgb(46, 48, 68),
            Color32::from_rgb(32, 34, 50),
            Color32::from_rgb(42, 44, 64),
            Color32::from_rgb(58, 60, 86),
            Color32::from_rgb(36, 38, 54),
            Color32::from_rgb(40, 42, 60),
            Color32::from_rgb(68, 70, 100),
        ),

        // Fallback to default theme if out of bounds
        _ => get_theme(0),
    }
}

// ─────────────────────────── Style Application ───────────────────────────────
pub fn apply_style(ctx: &egui::Context, accent: Color32, theme_idx: usize) {
    let th = get_theme(theme_idx);
    let mut v = egui::Visuals::dark();

    // ── Core fills ──
    v.panel_fill = th.bg;
    v.window_fill = th.card;
    v.window_stroke = Stroke::new(1.0, th.card_border);
    v.extreme_bg_color = th.panel;
    v.faint_bg_color = th.card;

    // ── Selection ──
    v.selection.bg_fill = accent.gamma_multiply(0.28);
    v.selection.stroke = Stroke::new(1.5, accent);

    // ── Window / popup shadow ──
    v.window_shadow.color = Color32::from_black_alpha(80);
    v.popup_shadow.color = Color32::from_black_alpha(60);

    // ── Corner radii ──
    let r_default = CornerRadius::same(8);
    let r_small = CornerRadius::same(6);
    let _r_large = CornerRadius::same(12);    // available for custom widgets
    let _r_modal = CornerRadius::same(16);    // available for modals

    // ── Non-interactive widgets (labels, separators, etc.) ──
    v.widgets.noninteractive.corner_radius = r_default;
    v.widgets.noninteractive.bg_fill = th.surface_1;
    v.widgets.noninteractive.weak_bg_fill = th.surface_1;
    v.widgets.noninteractive.bg_stroke = Stroke::new(1.0, th.border_subtle);
    v.widgets.noninteractive.fg_stroke = Stroke::new(1.0, DIM);

    // ── Inactive widgets (buttons at rest) ──
    v.widgets.inactive.corner_radius = r_default;
    v.widgets.inactive.bg_fill = th.card;
    v.widgets.inactive.weak_bg_fill = th.card;
    v.widgets.inactive.bg_stroke = Stroke::new(1.0, th.border_default);
    v.widgets.inactive.fg_stroke = Stroke::new(1.0, TEXT_SECONDARY);

    // ── Hovered widgets ──
    v.widgets.hovered.corner_radius = r_default;
    v.widgets.hovered.bg_fill = th.card_hover;
    v.widgets.hovered.weak_bg_fill = th.card_hover;
    v.widgets.hovered.bg_stroke = Stroke::new(1.5, accent.gamma_multiply(0.55));
    v.widgets.hovered.fg_stroke = Stroke::new(1.0, WHITE);

    // ── Active / pressed widgets ──
    v.widgets.active.corner_radius = r_small;
    v.widgets.active.bg_fill = accent.gamma_multiply(0.50);
    v.widgets.active.weak_bg_fill = accent.gamma_multiply(0.50);
    v.widgets.active.bg_stroke = Stroke::new(1.5, accent.gamma_multiply(0.75));
    v.widgets.active.fg_stroke = Stroke::new(1.0, WHITE);

    // ── Open / expanded widgets ──
    v.widgets.open.corner_radius = r_default;
    v.widgets.open.bg_fill = th.card_hover;
    v.widgets.open.weak_bg_fill = th.card_hover;
    v.widgets.open.bg_stroke = Stroke::new(1.0, accent.gamma_multiply(0.45));
    v.widgets.open.fg_stroke = Stroke::new(1.0, WHITE);

    ctx.set_visuals(v);

    // ── Spacing & animation ──
    ctx.all_styles_mut(|st| {
        st.spacing.item_spacing = vec2(6.0, 6.0);
        st.spacing.button_padding = vec2(14.0, 8.0);
        st.spacing.window_margin = egui::Margin::same(16);
        st.animation_time = 0.15;
    });
}
