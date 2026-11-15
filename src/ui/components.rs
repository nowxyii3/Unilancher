use crate::config::{Game, Source};
use crate::theme::{
    ThemeColors, BNET_C, EA_C, EPIC_C, GOG_C, GOLD, PANEL, RED, STEAM_C, TEXT_PRIMARY,
    TEXT_SECONDARY, TEXT_TERTIARY, UBI_C, WHITE, XBOX_C,
};
use crate::types::format_playtime;
use eframe::egui::{
    self, ecolor::Hsva, epaint::Mesh, pos2, vec2, Align, Align2, Color32, CornerRadius,
    FontId, Id, Layout, Margin, Pos2, Rect, RichText, Sense, Shape, Stroke, StrokeKind,
    TextureHandle, Ui,
};
use std::f32::consts::PI;

// ═══════════════════════════════════════════════════════════════════════════════
//  UniLauncher — Premium Component System
//  Refined for Raycast / Linear / Arc / Discord visual quality
// ═══════════════════════════════════════════════════════════════════════════════

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[allow(dead_code)]
pub enum Icon {
    Library,
    Star,
    Steam,
    Epic,
    Gog,
    Ubisoft,
    Ea,
    Battlenet,
    Xbox,
    Manual,
    EyeOff,
    Discord,
    Gear,
    Search,
    Refresh,
    Plus,
    Clock,
    Play,
    Dice,
    Grid,
    List,
    Folder,
    Sliders,
    SidebarToggle,
    Fullscreen,
    Tag,
    Chart,
    Gamepad,
    Save,
    Download,
    Gift,
}

// ───────────────────────────── Stat Chip / Tag ──────────────────────────────
pub fn stat_chip(ui: &mut Ui, text: &str, col: Color32) {
    egui::Frame::new()
        .fill(col.gamma_multiply(0.12))
        .stroke(Stroke::new(1.0, col.gamma_multiply(0.32)))
        .corner_radius(CornerRadius::same(6))
        .inner_margin(Margin::symmetric(8, 3))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 5.0;
                let (dot_rect, _) = ui.allocate_exact_size(vec2(6.0, 6.0), Sense::hover());
                ui.painter().circle_filled(dot_rect.center(), 2.5, col);
                ui.label(RichText::new(text).size(11.0).strong().color(TEXT_PRIMARY));
            });
        });
}

// ───────────────────────────── Barre de recherche ───────────────────────────
pub fn search_bar(
    ui: &mut Ui,
    query: &mut String,
    focus_req: &mut bool,
    accent: Color32,
    th: ThemeColors,
) {
    let has_query = !query.is_empty();
    let bar_height = 32.0;

    let (rect, resp) = ui.allocate_exact_size(vec2(220.0, bar_height), Sense::click());
    let focused = resp.has_focus() || *focus_req;
    let hovered = resp.hovered();

    let ease_focus = ui.ctx().animate_bool(resp.id.with("focus"), focused);
    let ease_hover = ui.ctx().animate_bool(resp.id.with("hov"), hovered);
    let active_t = ease_focus.max(ease_hover * 0.4);

    let bg = th.surface_2.lerp_to_gamma(th.surface_3, active_t);
    let border_col = if focused {
        accent.gamma_multiply(0.85)
    } else if has_query {
        accent.gamma_multiply(0.5)
    } else {
        th.border_default.lerp_to_gamma(th.border_hover, ease_hover)
    };

    let p = ui.painter();
    p.rect_filled(rect, CornerRadius::same(8), bg);

    // Lueur subtile quand le champ est actif
    if ease_focus > 0.01 {
        let glow_col = accent.gamma_multiply(0.12 * ease_focus);
        p.rect_stroke(
            rect.expand(2.0 * ease_focus),
            CornerRadius::same(9),
            Stroke::new(1.0, glow_col),
            StrokeKind::Outside,
        );
    }

    p.rect_stroke(
        rect,
        CornerRadius::same(8),
        Stroke::new(if focused { 1.2 } else { 1.0 }, border_col),
        StrokeKind::Inside,
    );

    // Contenu interne : Icône + Input + Bouton Clear
    let mut child = ui.new_child(
        egui::UiBuilder::new()
            .max_rect(rect.shrink2(vec2(10.0, 4.0)))
            .layout(Layout::left_to_right(Align::Center)),
    );

    child.spacing_mut().item_spacing.x = 6.0;

    // Icône de recherche
    let (icon_r, _) = child.allocate_exact_size(vec2(14.0, 14.0), Sense::hover());
    let icon_col = if focused || has_query {
        accent
    } else {
        TEXT_TERTIARY.lerp_to_gamma(TEXT_SECONDARY, ease_hover)
    };
    draw_icon(child.painter(), Icon::Search, icon_r.center(), icon_col);

    let text_w = child.available_width() - if has_query { 20.0 } else { 0.0 };
    let text_edit = egui::TextEdit::singleline(query)
        .frame(egui::Frame::NONE)
        .font(FontId::proportional(12.5))
        .text_color(TEXT_PRIMARY)
        .hint_text(RichText::new("Rechercher... (Ctrl+F)").size(12.0).color(TEXT_TERTIARY))
        .desired_width(text_w.max(60.0));

    let edit_resp = child.add(text_edit);
    if *focus_req {
        edit_resp.request_focus();
        *focus_req = false;
    }

    if has_query {
        let (btn_rect, btn_resp) = child.allocate_exact_size(vec2(16.0, 16.0), Sense::click());
        let btn_hov = btn_resp.hovered();
        let btn_bg = if btn_hov {
            th.card_hover
        } else {
            Color32::from_black_alpha(40)
        };
        child.painter().circle_filled(btn_rect.center(), 7.5, btn_bg);
        child.painter().text(
            btn_rect.center(),
            Align2::CENTER_CENTER,
            "✕",
            FontId::proportional(9.5),
            if btn_hov { TEXT_PRIMARY } else { TEXT_TERTIARY },
        );
        if btn_resp.clicked() {
            query.clear();
        }
    }
}

// ───────────────────────────── Bouton d'action moderne ───────────────────────
#[allow(clippy::too_many_arguments)]
pub fn modern_action_button(
    ui: &mut Ui,
    label: &str,
    icon: Icon,
    fill_col: Color32,
    primary: bool,
    th: ThemeColors,
) -> bool {
    let font = FontId::proportional(12.5);
    let gal = ui.painter().layout_no_wrap(label.to_string(), font, TEXT_PRIMARY);
    let size = vec2(gal.size().x + 38.0, 32.0);
    let (rect, resp) = ui.allocate_exact_size(size, Sense::click());
    let hov_t = ui.ctx().animate_bool(resp.id, resp.hovered());
    let press_t = ui.ctx().animate_bool(resp.id.with("press"), resp.is_pointer_button_down_on());

    let round = CornerRadius::same(7);
    let p = ui.painter();

    if primary {
        // Bouton primaire avec fond accentué et lueur douce au survol
        let bg = fill_col.lerp_to_gamma(Color32::WHITE, hov_t * 0.12);
        let bg = bg.gamma_multiply(1.0 - press_t * 0.15);

        if hov_t > 0.02 {
            let glow = fill_col.gamma_multiply(0.20 * hov_t);
            p.rect_filled(rect.expand(2.0), CornerRadius::same(9), glow);
        }

        p.rect_filled(rect, round, bg);

        // Ligne de biseau subtile en haut pour effet premium
        let top_line = Rect::from_min_size(
            pos2(rect.left() + 2.0, rect.top() + 1.0),
            vec2(rect.width() - 4.0, 1.0),
        );
        p.rect_filled(top_line, CornerRadius::same(1), Color32::from_white_alpha(35));

        p.rect_stroke(
            rect,
            round,
            Stroke::new(1.0, fill_col.lerp_to_gamma(Color32::WHITE, 0.25)),
            StrokeKind::Inside,
        );

        let icon_pos = pos2(rect.left() + 14.0, rect.center().y);
        draw_icon(p, icon, icon_pos, Color32::WHITE);
        p.galley(
            pos2(rect.left() + 26.0, rect.center().y - gal.size().y / 2.0),
            gal,
            Color32::WHITE,
        );
    } else {
        // Bouton secondaire sobre (Surface 2 -> Surface 3)
        let bg = th.surface_2.lerp_to_gamma(th.surface_3, hov_t);
        let border = th.border_default.lerp_to_gamma(th.border_hover, hov_t);
        let fg = TEXT_SECONDARY.lerp_to_gamma(TEXT_PRIMARY, hov_t);

        p.rect_filled(rect, round, bg);
        p.rect_stroke(rect, round, Stroke::new(1.0, border), StrokeKind::Inside);

        let icon_pos = pos2(rect.left() + 14.0, rect.center().y);
        draw_icon(p, icon, icon_pos, fg);
        p.galley(
            pos2(rect.left() + 26.0, rect.center().y - gal.size().y / 2.0),
            gal,
            fg,
        );
    }

    resp.on_hover_cursor(egui::CursorIcon::PointingHand).clicked()
}

// ───────────────────────────── Bouton carré d'icône ─────────────────────────
pub fn icon_square_button(
    ui: &mut Ui,
    icon: Icon,
    tip: &str,
    active: bool,
    th: ThemeColors,
) -> bool {
    let size = vec2(32.0, 32.0);
    let (rect, resp) = ui.allocate_exact_size(size, Sense::click());
    let hov_t = ui.ctx().animate_bool(resp.id, resp.hovered());

    let round = CornerRadius::same(7);
    let bg = if active {
        th.surface_3
    } else {
        th.surface_1.lerp_to_gamma(th.surface_2, hov_t)
    };

    let border = if active {
        th.border_hover
    } else {
        th.border_subtle.lerp_to_gamma(th.border_default, hov_t)
    };

    let fg = if active {
        TEXT_PRIMARY
    } else {
        TEXT_TERTIARY.lerp_to_gamma(TEXT_PRIMARY, hov_t)
    };

    let p = ui.painter();
    p.rect_filled(rect, round, bg);
    p.rect_stroke(rect, round, Stroke::new(1.0, border), StrokeKind::Inside);
    draw_icon(p, icon, rect.center(), fg);

    resp.on_hover_cursor(egui::CursorIcon::PointingHand)
        .on_hover_text(tip)
        .clicked()
}

// ───────────────────────────── Élément de navigation Sidebar ─────────────────
#[allow(clippy::too_many_arguments)]
pub fn sidebar_item(
    ui: &mut Ui,
    id: Id,
    icon: Icon,
    color: Color32,
    active: bool,
    label: &str,
    count: Option<usize>,
    compact: bool,
    th: ThemeColors,
) -> bool {
    let desired_w = ui.available_width();
    let (rect, resp) = ui.allocate_exact_size(vec2(desired_w, 34.0), Sense::click());
    let hov = resp.hovered();
    let act_t = ui.ctx().animate_bool(id.with("act"), active);
    let hov_t = ui.ctx().animate_bool(id.with("hov"), hov);

    let p = ui.painter();
    let round = CornerRadius::same(7);

    // Fond avec transition douce
    if act_t > 0.01 || hov_t > 0.01 {
        let fill = if active {
            color.gamma_multiply(0.12)
        } else {
            th.surface_2.gamma_multiply(hov_t * 0.7)
        };
        p.rect_filled(rect, round, fill);
    }

    // Bordure gauche d'accent pour l'élément actif (fine et discrète : 2.5px)
    if act_t > 0.01 {
        let bar_h = 16.0 * act_t;
        let bar_rect = Rect::from_center_size(
            pos2(rect.left() + 2.0, rect.center().y),
            vec2(2.5, bar_h),
        );
        p.rect_filled(bar_rect, CornerRadius::same(2), color);
    }

    let fg = if active {
        TEXT_PRIMARY
    } else {
        TEXT_SECONDARY.lerp_to_gamma(TEXT_PRIMARY, hov_t)
    };
    let icon_col = if active { color } else { fg };

    if compact {
        draw_icon(p, icon, rect.center(), icon_col);
        return resp
            .on_hover_cursor(egui::CursorIcon::PointingHand)
            .on_hover_text(label)
            .clicked();
    }

    let icon_pos = pos2(rect.left() + 18.0, rect.center().y);
    draw_icon(p, icon, icon_pos, icon_col);

    p.text(
        pos2(rect.left() + 36.0, rect.center().y),
        Align2::LEFT_CENTER,
        label,
        FontId::proportional(12.5),
        fg,
    );

    // Badge de compteur
    if let Some(c) = count {
        let badge_str = c.to_string();
        let badge_w = (badge_str.len() as f32 * 6.5 + 10.0).max(18.0);
        let badge_rect = Rect::from_center_size(
            pos2(rect.right() - 8.0 - badge_w / 2.0, rect.center().y),
            vec2(badge_w, 16.0),
        );

        let badge_bg = if active {
            color.gamma_multiply(0.20)
        } else {
            th.surface_2
        };

        p.rect_filled(badge_rect, CornerRadius::same(8), badge_bg);
        p.text(
            badge_rect.center(),
            Align2::CENTER_CENTER,
            badge_str,
            FontId::proportional(10.5),
            if active { color } else { TEXT_TERTIARY },
        );
    }

    resp.on_hover_cursor(egui::CursorIcon::PointingHand).clicked()
}

// ───────────────────────────── Moteur d'Icônes Vectorielles ──────────────────
pub fn draw_icon(p: &egui::Painter, icon: Icon, c: Pos2, col: Color32) {
    let st = Stroke::new(1.45, col);
    let st_thin = Stroke::new(1.2, col);

    match icon {
        Icon::Library => {
            // 4 micro-tuiles stylisées
            for (dx, dy) in [(-4.2, -4.2), (4.2, -4.2), (-4.2, 4.2), (4.2, 4.2)] {
                let r = Rect::from_center_size(c + vec2(dx, dy), vec2(5.8, 5.8));
                p.rect_stroke(r, CornerRadius::same(1), st, StrokeKind::Inside);
            }
        }
        Icon::Star => {
            let pts: Vec<Pos2> = (0..10)
                .map(|i| {
                    let a = -PI / 2.0 + i as f32 * PI / 5.0;
                    let r = if i % 2 == 0 { 7.5 } else { 3.2 };
                    c + vec2(a.cos() * r, a.sin() * r)
                })
                .collect();
            p.add(Shape::closed_line(pts, st));
        }
        Icon::Steam => {
            p.circle_stroke(c, 7.5, st);
            let p1 = c + vec2(2.5, -2.0);
            let p2 = c + vec2(-3.0, 2.5);
            p.line_segment([p2, p1], Stroke::new(1.8, col));
            p.circle_filled(p1, 2.6, col);
            p.circle_filled(p2, 1.8, col);
        }
        Icon::Epic => {
            let pts = vec![
                c + vec2(-5.5, -7.0),
                c + vec2(5.5, -7.0),
                c + vec2(5.5, 2.5),
                c + vec2(0.0, 7.5),
                c + vec2(-5.5, 2.5),
            ];
            p.add(Shape::closed_line(pts, st));
        }
        Icon::Manual => {
            let body = Rect::from_center_size(c, vec2(15.0, 10.0));
            p.rect_stroke(body, CornerRadius::same(3), st, StrokeKind::Inside);
            p.circle_filled(c + vec2(-3.2, 0.0), 1.3, col);
            p.circle_filled(c + vec2(3.2, -1.2), 1.1, col);
            p.circle_filled(c + vec2(2.0, 1.2), 1.1, col);
        }
        Icon::EyeOff => {
            p.circle_stroke(c, 5.0, st);
            p.line_segment([c + vec2(-6.0, -6.0), c + vec2(6.0, 6.0)], st);
        }
        Icon::Discord => {
            let body = Rect::from_center_size(c, vec2(15.0, 11.0));
            p.rect_filled(body, CornerRadius::same(4), col);
            p.circle_filled(c + vec2(-3.0, 0.0), 1.4, PANEL);
            p.circle_filled(c + vec2(3.0, 0.0), 1.4, PANEL);
        }
        Icon::Gear => {
            p.circle_stroke(c, 5.2, st);
            p.circle_filled(c, 1.8, col);
            for k in 0..6 {
                let a = k as f32 * PI / 3.0;
                let (s, co) = a.sin_cos();
                p.line_segment(
                    [c + vec2(co * 5.2, s * 5.2), c + vec2(co * 7.5, s * 7.5)],
                    st,
                );
            }
        }
        Icon::Search => {
            let center = c + vec2(-1.0, -1.0);
            p.circle_stroke(center, 4.8, st);
            p.line_segment([center + vec2(3.4, 3.4), center + vec2(6.5, 6.5)], Stroke::new(1.8, col));
        }
        Icon::Refresh => {
            p.circle_stroke(c, 5.5, st);
            p.circle_filled(c + vec2(4.0, -3.0), 1.8, col);
        }
        Icon::Plus => {
            p.line_segment([c + vec2(-4.5, 0.0), c + vec2(4.5, 0.0)], st);
            p.line_segment([c + vec2(0.0, -4.5), c + vec2(0.0, 4.5)], st);
        }
        Icon::Clock => {
            p.circle_stroke(c, 5.0, st_thin);
            p.line_segment([c, c + vec2(0.0, -2.8)], st_thin);
            p.line_segment([c, c + vec2(2.0, 0.8)], st_thin);
        }
        Icon::Play => {
            let tri = vec![
                c + vec2(-3.0, -4.5),
                c + vec2(-3.0, 4.5),
                c + vec2(5.0, 0.0),
            ];
            p.add(Shape::convex_polygon(tri, col, Stroke::NONE));
        }
        Icon::Dice => {
            let r = Rect::from_center_size(c, vec2(13.0, 13.0));
            p.rect_stroke(r, CornerRadius::same(3), st, StrokeKind::Inside);
            p.circle_filled(c + vec2(-3.0, -3.0), 1.2, col);
            p.circle_filled(c, 1.2, col);
            p.circle_filled(c + vec2(3.0, 3.0), 1.2, col);
        }
        Icon::Grid => {
            for (dx, dy) in [(-3.5, -3.5), (3.5, -3.5), (-3.5, 3.5), (3.5, 3.5)] {
                p.rect_filled(
                    Rect::from_center_size(c + vec2(dx, dy), vec2(4.5, 4.5)),
                    CornerRadius::same(1),
                    col,
                );
            }
        }
        Icon::List => {
            for dy in [-4.0, 0.0, 4.0] {
                p.line_segment([c + vec2(-5.5, dy), c + vec2(5.5, dy)], st);
            }
        }
        Icon::Folder => {
            let r = Rect::from_center_size(c + vec2(0.0, 0.8), vec2(13.0, 9.5));
            p.rect_stroke(r, CornerRadius::same(2), st, StrokeKind::Inside);
            p.line_segment([c + vec2(-4.5, -4.0), c + vec2(-1.5, -4.0)], st);
        }
        Icon::Sliders => {
            for (dy, dot_x) in [(-3.5, -2.0), (0.0, 2.2), (3.5, -1.0)] {
                p.line_segment([c + vec2(-5.5, dy), c + vec2(5.5, dy)], Stroke::new(1.2, col));
                p.circle_filled(c + vec2(dot_x, dy), 1.8, col);
            }
        }
        Icon::SidebarToggle => {
            let r = Rect::from_center_size(c, vec2(14.0, 11.0));
            p.rect_stroke(r, CornerRadius::same(2), st, StrokeKind::Inside);
            p.line_segment([c + vec2(-2.2, -5.5), c + vec2(-2.2, 5.5)], st);
        }
        Icon::Fullscreen => {
            let d = 5.0;
            for (sx, sy) in [(-1.0, -1.0), (1.0, -1.0), (-1.0, 1.0), (1.0, 1.0)] {
                let corner = c + vec2(sx * d, sy * d);
                p.line_segment([corner, corner + vec2(-sx * 3.0, 0.0)], st);
                p.line_segment([corner, corner + vec2(0.0, -sy * 3.0)], st);
            }
        }
        Icon::Gog => {
            p.circle_stroke(c, 7.0, st);
            p.circle_filled(c, 2.2, col);
        }
        Icon::Ubisoft => {
            p.circle_stroke(c, 7.0, st);
            p.circle_stroke(c + vec2(1.2, -0.8), 3.2, Stroke::new(1.2, col));
        }
        Icon::Ea => {
            let pts = vec![
                c + vec2(-5.5, 4.5),
                c + vec2(0.0, -6.0),
                c + vec2(5.5, 4.5),
            ];
            p.add(Shape::closed_line(pts, st));
            p.line_segment([c + vec2(-2.8, 1.2), c + vec2(2.8, 1.2)], st);
        }
        Icon::Battlenet => {
            p.circle_stroke(c, 7.0, st);
            p.circle_filled(c, 2.0, col);
            for k in 0..3 {
                let a = k as f32 * (2.0 * PI / 3.0);
                let (s, co) = a.sin_cos();
                p.line_segment([c, c + vec2(co * 6.5, s * 6.5)], Stroke::new(1.6, col));
            }
        }
        Icon::Xbox => {
            p.circle_stroke(c, 7.5, st);
            p.line_segment([c + vec2(-4.0, -4.0), c + vec2(4.0, 4.0)], Stroke::new(1.8, col));
            p.line_segment([c + vec2(4.0, -4.0), c + vec2(-4.0, 4.0)], Stroke::new(1.8, col));
        }
        Icon::Tag => {
            let pts = vec![
                c + vec2(-5.5, -1.8),
                c + vec2(-1.8, -5.5),
                c + vec2(5.0, -5.5),
                c + vec2(5.0, 2.5),
                c + vec2(-0.5, 5.0),
            ];
            p.add(Shape::closed_line(pts, st));
            p.circle_filled(c + vec2(2.2, -2.6), 1.1, col);
        }
        Icon::Chart => {
            p.rect_filled(Rect::from_center_size(c + vec2(-4.5, 2.0), vec2(2.6, 5.0)), CornerRadius::same(1), col);
            p.rect_filled(Rect::from_center_size(c + vec2(0.0, 0.0), vec2(2.6, 9.0)), CornerRadius::same(1), col);
            p.rect_filled(Rect::from_center_size(c + vec2(4.5, -2.0), vec2(2.6, 13.0)), CornerRadius::same(1), col);
        }
        Icon::Gamepad => {
            let body = Rect::from_center_size(c, vec2(15.0, 9.5));
            p.rect_stroke(body, CornerRadius::same(3), st, StrokeKind::Inside);
            p.line_segment([c + vec2(-4.0, -1.8), c + vec2(-4.0, 1.8)], Stroke::new(1.2, col));
            p.line_segment([c + vec2(-5.8, 0.0), c + vec2(-2.2, 0.0)], Stroke::new(1.2, col));
            p.circle_filled(c + vec2(4.0, -1.2), 1.1, col);
            p.circle_filled(c + vec2(2.6, 1.2), 1.1, col);
        }
        Icon::Save => {
            let r = Rect::from_center_size(c, vec2(12.0, 12.0));
            p.rect_stroke(r, CornerRadius::same(2), st, StrokeKind::Inside);
            p.rect_filled(Rect::from_center_size(c + vec2(0.0, -2.8), vec2(6.0, 4.5)), CornerRadius::same(1), col);
            p.rect_stroke(Rect::from_center_size(c + vec2(0.0, 3.2), vec2(7.0, 3.5)), CornerRadius::same(1), Stroke::new(1.0, col), StrokeKind::Inside);
        }
        Icon::Download => {
            p.line_segment([c + vec2(0.0, -5.5), c + vec2(0.0, 2.5)], st);
            p.line_segment([c + vec2(-3.5, -0.8), c + vec2(0.0, 2.5)], st);
            p.line_segment([c + vec2(3.5, -0.8), c + vec2(0.0, 2.5)], st);
            p.line_segment([c + vec2(-5.5, 5.5), c + vec2(5.5, 5.5)], st);
        }
        Icon::Gift => {
            let r = Rect::from_center_size(c + vec2(0.0, 1.2), vec2(11.0, 8.0));
            p.rect_stroke(r, CornerRadius::same(1), st, StrokeKind::Inside);
            let lid = Rect::from_center_size(c + vec2(0.0, -3.2), vec2(13.0, 3.0));
            p.rect_stroke(lid, CornerRadius::same(1), st, StrokeKind::Inside);
            p.line_segment([c + vec2(0.0, -4.5), c + vec2(0.0, 5.2)], Stroke::new(1.2, col));
        }
    }
}

// ───────────────────────────── Toggle / Switch Moderne ───────────────────────
pub fn toggle(ui: &mut Ui, id_str: &str, on: &mut bool, accent: Color32) -> egui::Response {
    let size = vec2(38.0, 20.0);
    let (rect, mut resp) = ui.allocate_exact_size(size, Sense::click());
    if resp.clicked() {
        *on = !*on;
        resp.mark_changed();
    }

    let t = ui.ctx().animate_bool(Id::unique(id_str), *on);
    let off_bg = Color32::from_rgb(38, 41, 54);
    let bg = off_bg.lerp_to_gamma(accent, t);

    let p = ui.painter();
    p.rect_filled(rect, CornerRadius::same(10), bg);
    p.rect_stroke(
        rect,
        CornerRadius::same(10),
        Stroke::new(1.0, Color32::from_white_alpha(15)),
        StrokeKind::Inside,
    );

    // Curseur blanc avec ombre portée légère
    let cx = egui::lerp((rect.left() + 10.0)..=(rect.right() - 10.0), t);
    let knob_center = pos2(cx, rect.center().y);

    // Micro-ombre sous le curseur
    p.circle_filled(knob_center + vec2(0.0, 1.0), 7.0, Color32::from_black_alpha(45));
    p.circle_filled(knob_center, 7.0, Color32::WHITE);

    resp.on_hover_cursor(egui::CursorIcon::PointingHand)
}

// ───────────────────────────── Section Card Conteneur ────────────────────────
pub fn section(ui: &mut Ui, title: &str, th: ThemeColors, add: impl FnOnce(&mut Ui)) {
    egui::Frame::new()
        .fill(th.card)
        .stroke(Stroke::new(1.0, th.card_border))
        .corner_radius(CornerRadius::same(10))
        .inner_margin(Margin::same(18))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());

            // Titre de section stylisé
            ui.horizontal(|ui| {
                ui.label(RichText::new(title).size(14.5).strong().color(TEXT_PRIMARY));
            });

            ui.add_space(8.0);
            add(ui);
        });
    ui.add_space(12.0);
}

// ───────────────────────────── Ligne de paramètre (Row) ──────────────────────
pub fn row(ui: &mut Ui, title: &str, desc: &str, control: impl FnOnce(&mut Ui)) {
    ui.horizontal(|ui| {
        ui.vertical(|ui| {
            ui.label(RichText::new(title).size(13.0).strong().color(TEXT_PRIMARY));
            if !desc.is_empty() {
                ui.add_space(1.5);
                ui.label(RichText::new(desc).size(11.5).color(TEXT_SECONDARY));
            }
        });
        ui.with_layout(Layout::right_to_left(Align::Center), control);
    });
    ui.add_space(4.0);
}

// ───────────────────────────── Générateur de Couleurs de Jaquette ────────────
pub fn cover_colors(name: &str) -> (Color32, Color32) {
    let mut h: u32 = 2166136261;
    for b in name.bytes() {
        h ^= b as u32;
        h = h.wrapping_mul(16777619);
    }
    let hue = (h % 360) as f32 / 360.0;
    (
        Hsva::new(hue, 0.40, 0.32, 1.0).into(),
        Hsva::new((hue + 0.05) % 1.0, 0.50, 0.12, 1.0).into(),
    )
}

// ───────────────────────────── Gradients Mesh ────────────────────────────────
pub fn draw_gradient_rect(p: &egui::Painter, rect: Rect, top_left: Color32, bottom_right: Color32) {
    let mid = top_left.lerp_to_gamma(bottom_right, 0.5);
    let mut mesh = Mesh::default();
    mesh.colored_vertex(rect.left_top(), top_left);
    mesh.colored_vertex(rect.right_top(), mid);
    mesh.colored_vertex(rect.right_bottom(), bottom_right);
    mesh.colored_vertex(rect.left_bottom(), mid);
    mesh.add_triangle(0, 1, 2);
    mesh.add_triangle(0, 2, 3);
    p.add(Shape::mesh(mesh));
}

pub fn draw_vertical_fade(p: &egui::Painter, rect: Rect, top: Color32, bottom: Color32) {
    let mut mesh = Mesh::default();
    mesh.colored_vertex(rect.left_top(), top);
    mesh.colored_vertex(rect.right_top(), top);
    mesh.colored_vertex(rect.right_bottom(), bottom);
    mesh.colored_vertex(rect.left_bottom(), bottom);
    mesh.add_triangle(0, 1, 2);
    mesh.add_triangle(0, 2, 3);
    p.add(Shape::mesh(mesh));
}

// ───────────────────────────── Pilule / Badge Compact ────────────────────────
pub fn pill(p: &egui::Painter, pos: Pos2, text: &str, bg: Color32, fg: Color32) -> f32 {
    let gal = p.layout_no_wrap(text.to_string(), FontId::proportional(9.5), fg);
    let w = gal.size().x + 10.0;
    let r = Rect::from_min_size(pos, vec2(w, gal.size().y + 4.0));
    p.rect_filled(r, CornerRadius::same(4), bg);
    p.rect_stroke(
        r,
        CornerRadius::same(4),
        Stroke::new(1.0, Color32::from_white_alpha(15)),
        StrokeKind::Inside,
    );
    p.galley(pos + vec2(5.0, 2.0), gal, fg);
    w
}

pub fn trunc(s: &str, n: usize) -> String {
    if s.chars().count() > n {
        format!("{}…", s.chars().take(n - 1).collect::<String>())
    } else {
        s.to_string()
    }
}

// ───────────────────────────── Hero Banner (Bannière Reprise) ────────────────
pub fn hero_banner(
    ui: &mut Ui,
    g: &Game,
    play_mins: u32,
    tex: Option<&TextureHandle>,
    accent: Color32,
    th: ThemeColors,
) -> (bool, bool) {
    let width = ui.available_width();
    let banner_h = 160.0;
    let (rect, resp) = ui.allocate_exact_size(vec2(width, banner_h), Sense::hover());
    let hov_t = ui
        .ctx()
        .animate_bool(Id::unique(("hero_hov", &g.id)), resp.hovered());

    let p = ui.painter().clone();
    let clip = p.with_clip_rect(rect);
    let (c1, c2) = cover_colors(&g.name);

    // Fond avec dégradé subtil
    draw_gradient_rect(&clip, rect, c2, th.card);

    // Halo lumineux dynamique lors du survol
    if hov_t > 0.01 {
        let glow_col = accent.gamma_multiply(0.08 * hov_t);
        p.rect_filled(rect.expand(2.0), CornerRadius::same(12), glow_col);
    }

    let thumb_rect = Rect::from_min_size(rect.min + vec2(16.0, 16.0), vec2(92.0, 128.0));
    if let Some(t) = tex {
        clip.image(
            t.id(),
            thumb_rect,
            Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
            WHITE,
        );
    } else {
        draw_gradient_rect(&clip, thumb_rect, c1, c2);
    }
    p.rect_stroke(
        thumb_rect,
        CornerRadius::same(7),
        Stroke::new(1.0, th.card_border),
        StrokeKind::Inside,
    );

    let text_x = thumb_rect.right() + 20.0;

    // Badge "REPRENDRE LA SESSION"
    pill(
        &clip,
        pos2(text_x, rect.top() + 18.0),
        "REPRENDRE LA SESSION",
        accent.gamma_multiply(0.22),
        TEXT_PRIMARY,
    );

    clip.text(
        pos2(text_x, rect.top() + 44.0),
        Align2::LEFT_TOP,
        &g.name,
        FontId::proportional(20.0),
        TEXT_PRIMARY,
    );

    let sub_info = format!("{}  •  {}", g.source.label(), format_playtime(play_mins));
    clip.text(
        pos2(text_x, rect.top() + 76.0),
        Align2::LEFT_TOP,
        sub_info,
        FontId::proportional(12.0),
        TEXT_SECONDARY,
    );

    // Boutons Jouer et Propriétés
    let btn_play_rect = Rect::from_min_size(pos2(text_x, rect.top() + 108.0), vec2(146.0, 32.0));
    let play_resp = ui.interact(btn_play_rect, Id::unique(("hero_play", &g.id)), Sense::click());
    let play_hov = play_resp.hovered();
    let play_bg = if play_hov { accent.lerp_to_gamma(WHITE, 0.12) } else { accent };

    clip.rect_filled(btn_play_rect, CornerRadius::same(7), play_bg);
    clip.text(
        btn_play_rect.center(),
        Align2::CENTER_CENTER,
        "▶  Jouer maintenant",
        FontId::proportional(12.5),
        Color32::WHITE,
    );

    let btn_prop_rect = Rect::from_min_size(pos2(btn_play_rect.right() + 10.0, rect.top() + 108.0), vec2(112.0, 32.0));
    let prop_resp = ui.interact(btn_prop_rect, Id::unique(("hero_prop", &g.id)), Sense::click());
    let prop_hov = prop_resp.hovered();
    let prop_bg = if prop_hov { th.surface_3 } else { th.surface_2 };

    clip.rect_filled(btn_prop_rect, CornerRadius::same(7), prop_bg);
    clip.rect_stroke(
        btn_prop_rect,
        CornerRadius::same(7),
        Stroke::new(1.0, if prop_hov { th.border_hover } else { th.border_default }),
        StrokeKind::Inside,
    );
    clip.text(
        btn_prop_rect.center(),
        Align2::CENTER_CENTER,
        "⚙  Propriétés",
        FontId::proportional(12.0),
        if prop_hov { TEXT_PRIMARY } else { TEXT_SECONDARY },
    );

    p.rect_stroke(
        rect,
        CornerRadius::same(12),
        Stroke::new(1.0, th.card_border.lerp_to_gamma(accent, hov_t * 0.5)),
        StrokeKind::Inside,
    );

    (play_resp.clicked(), prop_resp.clicked())
}

// ───────────────────────────── Carte de Jeu Moderne (Game Card) ──────────────────────
#[allow(clippy::too_many_arguments)]
pub fn game_card(
    ui: &mut Ui,
    g: &Game,
    tex: Option<&TextureHandle>,
    play_mins: u32,
    fav: bool,
    broken: bool,
    is_running: bool,
    custom: bool,
    accent: Color32,
    scale: f32,
    grid_size: crate::types::GridCardSize,
    th: ThemeColors,
) -> (egui::Response, bool, bool, bool) {
    let scale = scale.clamp(0.70, 1.40);
    let (card_w, card_h) = grid_size.dimensions(scale);
    let (base_rect, resp) = ui.allocate_exact_size(vec2(card_w, card_h), Sense::click());
    let resp = resp.on_hover_cursor(egui::CursorIcon::PointingHand);

    // Culling visuel ultra-rapide : zéro coût GPU/CPU si hors écran (supporte des milliers de jeux)
    if !ui.is_rect_visible(base_rect) {
        return (resp, false, false, false);
    }

    let hov_t = ui
        .ctx()
        .animate_bool(Id::unique(("card_hov", &g.id)), resp.hovered());

    let p = ui.painter().clone();

    // Élévation douce de 3.5px au survol
    let rect = base_rect.translate(vec2(0.0, -3.5 * hov_t));
    let round = CornerRadius::same((10.0 * scale).round() as u8);

    let src_col = match g.source {
        Source::Steam => STEAM_C,
        Source::Epic => EPIC_C,
        Source::Gog => GOG_C,
        Source::Ubisoft => UBI_C,
        Source::Ea => EA_C,
        Source::Battlenet => BNET_C,
        Source::Xbox => XBOX_C,
        Source::Manual => accent,
    };

    // Halo d'ambiance lissé (Ambient Glow)
    if hov_t > 0.02 {
        let glow_outer = src_col.gamma_multiply(0.12 * hov_t);
        let glow_inner = src_col.gamma_multiply(0.24 * hov_t);
        p.rect_filled(
            rect.expand(7.0 * hov_t * scale),
            CornerRadius::same(14),
            glow_outer,
        );
        p.rect_filled(
            rect.expand(2.5 * hov_t * scale),
            CornerRadius::same(11),
            glow_inner,
        );
    }

    p.rect_filled(rect, round, th.card);

    let inner = rect.shrink(2.0 * scale);
    let clip = p.with_clip_rect(inner);

    if let Some(t) = tex {
        // Micro-zoom fluide de 1.8% au survol
        let zoom = 0.018 * hov_t;
        let uv = Rect::from_min_max(pos2(zoom, zoom), pos2(1.0 - zoom, 1.0 - zoom));
        clip.image(t.id(), inner, uv, WHITE);
    } else {
        let (c1, c2) = cover_colors(&g.name);
        draw_gradient_rect(&clip, inner, c1, c2);
        let initials: String = g
            .name
            .split_whitespace()
            .filter_map(|w| w.chars().next())
            .take(2)
            .collect::<String>()
            .to_uppercase();
        clip.text(
            inner.center() - vec2(0.0, 18.0 * scale),
            Align2::CENTER_CENTER,
            if initials.is_empty() { "?" } else { &initials },
            FontId::proportional(38.0 * scale),
            Color32::from_white_alpha(35),
        );
    }

    // Dégradé sombre progressif au bas de la jaquette pour une lisibilité parfaite des titres
    let fade_h = 110.0 * scale;
    let fade_rect = Rect::from_min_max(pos2(inner.left(), inner.bottom() - fade_h), inner.right_bottom());
    draw_vertical_fade(
        &clip,
        fade_rect,
        Color32::TRANSPARENT,
        Color32::from_black_alpha(230),
    );

    // Badges en haut à gauche : Statut d'exécution ("EN JEU") ou Plateforme
    if is_running {
        pill(
            &clip,
            inner.min + vec2(6.0 * scale, 6.0 * scale),
            "🟢 EN JEU",
            Color32::from_rgb(16, 185, 129),
            WHITE,
        );
    } else {
        pill(
            &clip,
            inner.min + vec2(6.0 * scale, 6.0 * scale),
            g.source.label(),
            Color32::from_black_alpha(185),
            src_col,
        );
    }

    // Étoile de favori en haut à droite
    let star_center = pos2(inner.right() - 17.0 * scale, inner.top() + 17.0 * scale);
    let star_rect = Rect::from_center_size(star_center, vec2(26.0 * scale, 26.0 * scale));
    let star_hovered = resp
        .hover_pos()
        .map(|hp| star_rect.contains(hp))
        .unwrap_or(false);
    let star_clicked = resp.clicked() && star_hovered;

    if fav || hov_t > 0.05 {
        clip.circle_filled(star_center, 11.5 * scale, Color32::from_black_alpha(175));
        let s_col = if fav {
            GOLD
        } else if star_hovered {
            WHITE
        } else {
            TEXT_TERTIARY
        };
        draw_icon(&clip, Icon::Star, star_center, s_col);
    }

    // Badge d'installation ou avertissement si manquant
    if broken {
        pill(
            &clip,
            pos2(inner.left() + 7.0 * scale, inner.bottom() - 68.0 * scale),
            "NON INSTALLÉ",
            RED.gamma_multiply(0.90),
            WHITE,
        );
    } else if custom {
        pill(
            &clip,
            pos2(inner.left() + 7.0 * scale, inner.bottom() - 68.0 * scale),
            "EXE MANUEL",
            Color32::from_black_alpha(180),
            TEXT_SECONDARY,
        );
    }

    // Actions rapides au survol (Bouton Jouer central + Bouton Détails)
    let mut play_clicked = false;
    let mut details_clicked = false;

    if hov_t > 0.10 {
        let center_y = inner.center().y - 6.0 * scale;
        let btn_play_rect = Rect::from_center_size(
            pos2(inner.center().x - 22.0 * scale, center_y),
            vec2(38.0 * scale, 38.0 * scale),
        );
        let btn_det_rect = Rect::from_center_size(
            pos2(inner.center().x + 22.0 * scale, center_y),
            vec2(38.0 * scale, 38.0 * scale),
        );

        let hp = resp.hover_pos();
        let play_hov = hp.map(|p| btn_play_rect.contains(p)).unwrap_or(false);
        let det_hov = hp.map(|p| btn_det_rect.contains(p)).unwrap_or(false);

        // Bouton Jouer rapide
        let play_bg = if play_hov { accent } else { accent.gamma_multiply(0.85) };
        clip.circle_filled(btn_play_rect.center(), 18.0 * scale, play_bg);
        clip.circle_stroke(btn_play_rect.center(), 18.0 * scale, Stroke::new(1.0, Color32::WHITE.gamma_multiply(0.50)));
        draw_icon(&clip, Icon::Play, btn_play_rect.center(), WHITE);

        // Bouton Détails rapide
        let det_bg = if det_hov { th.surface_3 } else { th.surface_2 };
        clip.circle_filled(btn_det_rect.center(), 18.0 * scale, det_bg);
        clip.circle_stroke(btn_det_rect.center(), 18.0 * scale, Stroke::new(1.0, th.border_hover));
        draw_icon(&clip, Icon::Sliders, btn_det_rect.center(), if det_hov { WHITE } else { TEXT_SECONDARY });

        if resp.clicked() {
            if play_hov {
                play_clicked = true;
            } else if det_hov {
                details_clicked = true;
            }
        }
    }

    // Zone d'informations au bas de la carte (alignée rigoureusement)
    let title_font_size = (13.5 * scale).clamp(11.0, 16.0);
    let title_char_limit = match grid_size {
        crate::types::GridCardSize::Small => 16,
        crate::types::GridCardSize::Medium => 22,
        crate::types::GridCardSize::Large => 28,
    };

    clip.text(
        pos2(inner.left() + 8.0 * scale, inner.bottom() - 44.0 * scale),
        Align2::LEFT_TOP,
        trunc(&g.name, title_char_limit),
        FontId::proportional(title_font_size),
        TEXT_PRIMARY,
    );

    // Temps de jeu
    let time_y = inner.bottom() - 18.0 * scale;
    draw_icon(&clip, Icon::Clock, pos2(inner.left() + 14.0 * scale, time_y), TEXT_TERTIARY);
    clip.text(
        pos2(inner.left() + 24.0 * scale, time_y - 6.5 * scale),
        Align2::LEFT_TOP,
        format_playtime(play_mins),
        FontId::proportional((11.0 * scale).clamp(9.5, 13.5)),
        TEXT_SECONDARY,
    );

    // Bordure avec mise en valeur au survol
    let border_col = th.card_border.lerp_to_gamma(accent, hov_t * 0.70);
    p.rect_stroke(
        rect,
        round,
        Stroke::new(if hov_t > 0.1 { 1.2 } else { 1.0 }, border_col),
        StrokeKind::Inside,
    );

    (resp, star_clicked, details_clicked, play_clicked)
}

// ───────────────────────────── Ligne de Liste (Game List Row) ────────────────
// ───────────────────────────── Ligne de Liste Moderne (Game List Row) ────────────────
#[allow(clippy::too_many_arguments)]
pub fn game_list_row(
    ui: &mut Ui,
    g: &Game,
    tex: Option<&TextureHandle>,
    play_mins: u32,
    last_session: Option<u64>,
    fav: bool,
    broken: bool,
    is_running: bool,
    accent: Color32,
    th: ThemeColors,
) -> (egui::Response, bool, bool, bool) {
    let width = ui.available_width();
    let (rect, resp) = ui.allocate_exact_size(vec2(width, 56.0), Sense::click());

    // Culling visuel : économise 100% du rendu pour les lignes hors écran
    if !ui.is_rect_visible(rect) {
        return (resp, false, false, false);
    }

    let hov_t = ui
        .ctx()
        .animate_bool(Id::unique(("row_hov", &g.id)), resp.hovered());

    let p = ui.painter();
    let bg = th.card.lerp_to_gamma(th.card_hover, hov_t);
    let round = CornerRadius::same(8);

    p.rect_filled(rect, round, bg);
    p.rect_stroke(
        rect,
        round,
        Stroke::new(1.0, th.card_border.lerp_to_gamma(accent, hov_t * 0.45)),
        StrokeKind::Inside,
    );

    // 1. Miniature jaquette
    let thumb = Rect::from_min_size(rect.min + vec2(8.0, 7.0), vec2(32.0, 42.0));
    if let Some(t) = tex {
        p.image(
            t.id(),
            thumb,
            Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
            WHITE,
        );
    } else {
        let (c1, c2) = cover_colors(&g.name);
        draw_gradient_rect(p, thumb, c1, c2);
    }
    p.rect_stroke(thumb, CornerRadius::same(5), Stroke::new(1.0, th.border_subtle), StrokeKind::Inside);

    let src_col = match g.source {
        Source::Steam => STEAM_C,
        Source::Epic => EPIC_C,
        Source::Gog => GOG_C,
        Source::Ubisoft => UBI_C,
        Source::Ea => EA_C,
        Source::Battlenet => BNET_C,
        Source::Xbox => XBOX_C,
        Source::Manual => accent,
    };

    // 2. Colonne Titre & Plateforme
    let name_x = thumb.right() + 14.0;
    p.text(
        pos2(name_x, rect.top() + 10.0),
        Align2::LEFT_TOP,
        trunc(&g.name, 32),
        FontId::proportional(13.5),
        TEXT_PRIMARY,
    );

    let src_label = g.source.label();
    let src_gal = p.layout_no_wrap(src_label.to_string(), FontId::proportional(10.5), src_col);
    p.galley(pos2(name_x, rect.top() + 31.0), src_gal, src_col);

    // 3. Colonne Temps de jeu & Dernière session
    let col_time_x = (rect.left() + width * 0.44).max(name_x + 180.0);
    let time_str = format_playtime(play_mins);
    p.text(
        pos2(col_time_x, rect.top() + 11.0),
        Align2::LEFT_TOP,
        time_str,
        FontId::proportional(12.5),
        TEXT_PRIMARY,
    );

    let last_str = crate::types::format_last_played_time(last_session);
    p.text(
        pos2(col_time_x, rect.top() + 30.0),
        Align2::LEFT_TOP,
        last_str,
        FontId::proportional(11.0),
        TEXT_TERTIARY,
    );

    // 4. Colonne Statut
    let col_status_x = (rect.left() + width * 0.68).max(col_time_x + 120.0);
    let (status_text, status_col) = if is_running {
        ("🟢 En jeu", Color32::from_rgb(16, 185, 129))
    } else if broken {
        ("⚠️ Non installé", RED)
    } else {
        ("🟢 Prêt", TEXT_SECONDARY)
    };
    p.text(
        pos2(col_status_x, rect.center().y),
        Align2::LEFT_CENTER,
        status_text,
        FontId::proportional(12.0),
        status_col,
    );

    // 5. Actions interactives à droite
    let play_rect = Rect::from_center_size(pos2(rect.right() - 44.0, rect.center().y), vec2(64.0, 30.0));
    let opt_rect = Rect::from_center_size(pos2(rect.right() - 98.0, rect.center().y), vec2(30.0, 30.0));
    let fav_rect = Rect::from_center_size(pos2(rect.right() - 136.0, rect.center().y), vec2(30.0, 30.0));

    let hp = resp.hover_pos();
    let play_hov = hp.map(|pos| play_rect.contains(pos)).unwrap_or(false);
    let opt_hov = hp.map(|pos| opt_rect.contains(pos)).unwrap_or(false);
    let fav_hov = hp.map(|pos| fav_rect.contains(pos)).unwrap_or(false);

    // Étoile de favori
    draw_icon(
        p,
        Icon::Star,
        fav_rect.center(),
        if fav {
            GOLD
        } else if fav_hov {
            WHITE
        } else {
            TEXT_TERTIARY
        },
    );

    // Bouton Détails / Configuration
    draw_icon(
        p,
        Icon::Sliders,
        opt_rect.center(),
        if opt_hov { TEXT_PRIMARY } else { TEXT_TERTIARY },
    );

    // Bouton Jouer
    p.rect_filled(
        play_rect,
        CornerRadius::same(6),
        if play_hov {
            accent.lerp_to_gamma(WHITE, 0.12)
        } else {
            accent
        },
    );
    p.text(
        play_rect.center(),
        Align2::CENTER_CENTER,
        "▶ Jouer",
        FontId::proportional(12.0),
        Color32::WHITE,
    );

    let clicked = resp.clicked();
    (
        resp.on_hover_cursor(egui::CursorIcon::PointingHand),
        clicked && play_hov,
        clicked && fav_hov,
        clicked && opt_hov,
    )
}
