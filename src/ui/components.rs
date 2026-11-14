use crate::config::{Game, Source};
use crate::theme::{
    ThemeColors, BNET_C, DIM, EA_C, EPIC_C, GOG_C, GOLD, PANEL, RED, STEAM_C, UBI_C, WHITE, XBOX_C,
};
use crate::types::format_playtime;
use eframe::egui::{
    self, ecolor::Hsva, epaint::Mesh, pos2, vec2, Align, Align2, Color32, CornerRadius,
    FontId, Id, Layout, Margin, Pos2, Rect, RichText, Sense, Shape, Stroke, StrokeKind,
    TextureHandle, Ui,
};
use std::f32::consts::PI;

#[derive(Clone, Copy)]
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

pub fn stat_chip(ui: &mut Ui, text: &str, col: Color32) {
    egui::Frame::new()
        .fill(col.gamma_multiply(0.14))
        .stroke(Stroke::new(1.0, col.gamma_multiply(0.40)))
        .corner_radius(12.0)
        .inner_margin(Margin::symmetric(10, 4))
        .show(ui, |ui| {
            ui.label(RichText::new(text).size(11.5).strong().color(WHITE));
        });
}

pub fn search_bar(
    ui: &mut Ui,
    query: &mut String,
    focus_req: &mut bool,
    accent: Color32,
    th: ThemeColors,
) {
    let active = !query.is_empty();
    egui::Frame::new()
        .fill(if active { th.card_hover } else { th.card })
        .stroke(Stroke::new(
            if active { 1.5 } else { 1.0 },
            if active { accent } else { th.card_border },
        ))
        .corner_radius(20.0)
        .inner_margin(Margin::symmetric(12, 6))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 8.0;
                let (r, _) = ui.allocate_exact_size(vec2(16.0, 16.0), Sense::hover());
                draw_icon(ui.painter(), Icon::Search, r.center(), if active { accent } else { DIM });

                let resp = ui.add(
                    egui::TextEdit::singleline(query)
                        .frame(egui::Frame::NONE)
                        .hint_text(RichText::new("Rechercher (Ctrl+F)…").size(12.5).color(DIM))
                        .desired_width(190.0),
                );
                if *focus_req {
                    resp.request_focus();
                    *focus_req = false;
                }
                if !query.is_empty() && ui.small_button("✕").clicked() {
                    query.clear();
                }
            });
        });
}

#[allow(clippy::too_many_arguments)]
pub fn modern_action_button(
    ui: &mut Ui,
    label: &str,
    icon: Icon,
    fill_col: Color32,
    primary: bool,
    th: ThemeColors,
) -> bool {
    let font = FontId::proportional(13.0);
    let gal = ui.painter().layout_no_wrap(label.to_string(), font, WHITE);
    let size = vec2(gal.size().x + 42.0, 36.0);
    let (rect, resp) = ui.allocate_exact_size(size, Sense::click());
    let hov_t = ui.ctx().animate_bool(resp.id, resp.hovered());

    let bg = if primary {
        fill_col.lerp_to_gamma(Color32::WHITE, hov_t * 0.16)
    } else {
        th.card.lerp_to_gamma(th.card_hover, hov_t)
    };
    let p = ui.painter();
    p.rect_filled(rect, CornerRadius::same(10), bg);
    let border_col = if primary {
        fill_col.lerp_to_gamma(WHITE, 0.3)
    } else {
        th.card_border.lerp_to_gamma(WHITE, hov_t * 0.25)
    };
    p.rect_stroke(rect, CornerRadius::same(10), Stroke::new(1.0, border_col), StrokeKind::Inside);

    let icon_pos = pos2(rect.left() + 16.0, rect.center().y);
    draw_icon(p, icon, icon_pos, WHITE);
    p.galley(
        pos2(rect.left() + 30.0, rect.center().y - gal.size().y / 2.0),
        gal,
        WHITE,
    );

    resp.on_hover_cursor(egui::CursorIcon::PointingHand)
        .clicked()
}

pub fn icon_square_button(
    ui: &mut Ui,
    icon: Icon,
    tip: &str,
    active: bool,
    th: ThemeColors,
) -> bool {
    let (rect, resp) = ui.allocate_exact_size(vec2(34.0, 34.0), Sense::click());
    let hov_t = ui.ctx().animate_bool(resp.id, resp.hovered());
    let bg = if active {
        th.card_hover
    } else {
        th.card.lerp_to_gamma(th.card_hover, hov_t)
    };
    let fg = if active || hov_t > 0.1 { WHITE } else { DIM };
    let p = ui.painter();
    p.rect_filled(rect, CornerRadius::same(8), bg);
    p.rect_stroke(
        rect,
        CornerRadius::same(8),
        Stroke::new(1.0, if active { WHITE.gamma_multiply(0.3) } else { th.card_border }),
        StrokeKind::Inside,
    );
    draw_icon(p, icon, rect.center(), fg);
    resp.on_hover_cursor(egui::CursorIcon::PointingHand)
        .on_hover_text(tip)
        .clicked()
}

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
    let (rect, resp) = ui.allocate_exact_size(vec2(desired_w, 38.0), Sense::click());
    let hov = resp.hovered();
    let act_t = ui.ctx().animate_bool(id.with("act"), active);
    let hov_t = ui.ctx().animate_bool(id.with("hov"), hov);
    let blend = act_t.max(hov_t * 0.6);

    let p = ui.painter();
    if blend > 0.01 {
        let bg = if active {
            th.card_hover
        } else {
            th.card.gamma_multiply(hov_t)
        };
        p.rect_filled(rect, CornerRadius::same(8), bg);
    }

    if act_t > 0.01 {
        p.rect_filled(
            Rect::from_center_size(
                pos2(rect.left() + 2.0, rect.center().y),
                vec2(3.0, 18.0 * act_t),
            ),
            CornerRadius::same(2),
            color,
        );
    }

    let fg = DIM.lerp_to_gamma(WHITE, blend);
    let icon_col = if active { color } else { fg };

    if compact {
        draw_icon(p, icon, rect.center(), icon_col);
        return resp
            .on_hover_cursor(egui::CursorIcon::PointingHand)
            .on_hover_text(label)
            .clicked();
    }

    let icon_pos = pos2(rect.left() + 20.0, rect.center().y);
    draw_icon(p, icon, icon_pos, icon_col);

    p.text(
        pos2(rect.left() + 38.0, rect.center().y),
        Align2::LEFT_CENTER,
        label,
        FontId::proportional(13.0),
        fg,
    );

    if let Some(c) = count {
        let badge_str = c.to_string();
        let badge_w = (badge_str.len() as f32 * 7.0 + 12.0).max(18.0);
        let badge_rect = Rect::from_center_size(
            pos2(rect.right() - 8.0 - badge_w / 2.0, rect.center().y),
            vec2(badge_w, 18.0),
        );
        p.rect_filled(badge_rect, CornerRadius::same(9), th.badge_bg);
        p.text(
            badge_rect.center(),
            Align2::CENTER_CENTER,
            badge_str,
            FontId::proportional(11.0),
            if active { color } else { DIM },
        );
    }

    resp.on_hover_cursor(egui::CursorIcon::PointingHand)
        .clicked()
}

pub fn draw_icon(p: &egui::Painter, icon: Icon, c: Pos2, col: Color32) {
    let st = Stroke::new(1.7, col);
    match icon {
        Icon::Library => {
            for (dx, dy) in [(-4.5, -4.5), (4.5, -4.5), (-4.5, 4.5), (4.5, 4.5)] {
                let r = Rect::from_center_size(c + vec2(dx, dy), vec2(6.5, 6.5));
                p.rect_stroke(r, CornerRadius::same(1), st, StrokeKind::Inside);
            }
        }
        Icon::Star => {
            let pts: Vec<Pos2> = (0..10)
                .map(|i| {
                    let a = -PI / 2.0 + i as f32 * PI / 5.0;
                    let r = if i % 2 == 0 { 8.0 } else { 3.5 };
                    c + vec2(a.cos() * r, a.sin() * r)
                })
                .collect();
            p.add(Shape::closed_line(pts, st));
        }
        Icon::Steam => {
            p.circle_stroke(c, 8.5, st);
            let p1 = c + vec2(2.8, -2.2);
            let p2 = c + vec2(-3.5, 3.0);
            p.line_segment([p2, p1], Stroke::new(2.2, col));
            p.circle_filled(p1, 3.0, col);
            p.circle_filled(p2, 2.2, col);
        }
        Icon::Epic => {
            let pts = vec![
                c + vec2(-6.5, -8.0),
                c + vec2(6.5, -8.0),
                c + vec2(6.5, 3.0),
                c + vec2(0.0, 8.5),
                c + vec2(-6.5, 3.0),
            ];
            p.add(Shape::closed_line(pts, st));
        }
        Icon::Manual => {
            let body = Rect::from_center_size(c, vec2(16.0, 11.0));
            p.rect_stroke(body, CornerRadius::same(4), st, StrokeKind::Inside);
            p.circle_filled(c + vec2(-3.5, 0.0), 1.5, col);
            p.circle_filled(c + vec2(3.5, -1.5), 1.3, col);
            p.circle_filled(c + vec2(2.0, 1.5), 1.3, col);
        }
        Icon::EyeOff => {
            p.circle_stroke(c, 5.5, st);
            p.line_segment([c + vec2(-7.0, -7.0), c + vec2(7.0, 7.0)], st);
        }
        Icon::Discord => {
            let body = Rect::from_center_size(c, vec2(16.0, 12.0));
            p.rect_filled(body, CornerRadius::same(5), col);
            p.circle_filled(c + vec2(-3.2, 0.0), 1.6, PANEL);
            p.circle_filled(c + vec2(3.2, 0.0), 1.6, PANEL);
        }
        Icon::Gear => {
            p.circle_stroke(c, 6.0, st);
            p.circle_filled(c, 2.2, col);
            for k in 0..6 {
                let a = k as f32 * PI / 3.0;
                let (s, co) = a.sin_cos();
                p.line_segment(
                    [c + vec2(co * 6.0, s * 6.0), c + vec2(co * 8.5, s * 8.5)],
                    st,
                );
            }
        }
        Icon::Search => {
            let center = c + vec2(-1.2, -1.2);
            p.circle_stroke(center, 5.0, st);
            p.line_segment([center + vec2(3.6, 3.6), center + vec2(7.0, 7.0)], st);
        }
        Icon::Refresh => {
            p.circle_stroke(c, 6.0, st);
            p.circle_filled(c + vec2(4.5, -3.5), 2.0, col);
        }
        Icon::Plus => {
            p.line_segment([c + vec2(-5.0, 0.0), c + vec2(5.0, 0.0)], st);
            p.line_segment([c + vec2(0.0, -5.0), c + vec2(0.0, 5.0)], st);
        }
        Icon::Clock => {
            p.circle_stroke(c, 5.2, Stroke::new(1.4, col));
            p.line_segment([c, c + vec2(0.0, -3.0)], Stroke::new(1.4, col));
            p.line_segment([c, c + vec2(2.2, 1.0)], Stroke::new(1.4, col));
        }
        Icon::Play => {
            let tri = vec![
                c + vec2(-3.5, -5.0),
                c + vec2(-3.5, 5.0),
                c + vec2(5.5, 0.0),
            ];
            p.add(Shape::convex_polygon(tri, col, Stroke::NONE));
        }
        Icon::Dice => {
            let r = Rect::from_center_size(c, vec2(14.0, 14.0));
            p.rect_stroke(r, CornerRadius::same(3), st, StrokeKind::Inside);
            p.circle_filled(c + vec2(-3.2, -3.2), 1.3, col);
            p.circle_filled(c, 1.3, col);
            p.circle_filled(c + vec2(3.2, 3.2), 1.3, col);
        }
        Icon::Grid => {
            for (dx, dy) in [(-3.8, -3.8), (3.8, -3.8), (-3.8, 3.8), (3.8, 3.8)] {
                p.rect_filled(
                    Rect::from_center_size(c + vec2(dx, dy), vec2(5.0, 5.0)),
                    CornerRadius::same(1),
                    col,
                );
            }
        }
        Icon::List => {
            for dy in [-4.5, 0.0, 4.5] {
                p.line_segment([c + vec2(-6.0, dy), c + vec2(6.0, dy)], st);
            }
        }
        Icon::Folder => {
            let r = Rect::from_center_size(c + vec2(0.0, 1.0), vec2(14.0, 10.0));
            p.rect_stroke(r, CornerRadius::same(2), st, StrokeKind::Inside);
        }
        Icon::Sliders => {
            for (dy, dot_x) in [(-4.0, -2.0), (0.0, 2.5), (4.0, -1.0)] {
                p.line_segment([c + vec2(-6.5, dy), c + vec2(6.5, dy)], Stroke::new(1.4, col));
                p.circle_filled(c + vec2(dot_x, dy), 2.2, col);
            }
        }
        Icon::SidebarToggle => {
            let r = Rect::from_center_size(c, vec2(15.0, 12.0));
            p.rect_stroke(r, CornerRadius::same(2), st, StrokeKind::Inside);
            p.line_segment([c + vec2(-2.5, -6.0), c + vec2(-2.5, 6.0)], st);
        }
        Icon::Fullscreen => {
            let d = 5.5;
            for (sx, sy) in [(-1.0, -1.0), (1.0, -1.0), (-1.0, 1.0), (1.0, 1.0)] {
                let corner = c + vec2(sx * d, sy * d);
                p.line_segment([corner, corner + vec2(-sx * 3.5, 0.0)], st);
                p.line_segment([corner, corner + vec2(0.0, -sy * 3.5)], st);
            }
        }
        Icon::Gog => {
            p.circle_stroke(c, 7.5, st);
            p.circle_filled(c, 2.5, col);
        }
        Icon::Ubisoft => {
            p.circle_stroke(c, 7.5, st);
            p.circle_stroke(c + vec2(1.5, -1.0), 3.5, Stroke::new(1.3, col));
        }
        Icon::Ea => {
            let pts = vec![
                c + vec2(-6.0, 5.0),
                c + vec2(0.0, -6.5),
                c + vec2(6.0, 5.0),
            ];
            p.add(Shape::closed_line(pts, st));
            p.line_segment([c + vec2(-3.0, 1.5), c + vec2(3.0, 1.5)], st);
        }
        Icon::Battlenet => {
            p.circle_stroke(c, 7.5, st);
            p.circle_filled(c, 2.2, col);
            for k in 0..3 {
                let a = k as f32 * (2.0 * PI / 3.0);
                let (s, co) = a.sin_cos();
                p.line_segment([c, c + vec2(co * 7.0, s * 7.0)], Stroke::new(1.8, col));
            }
        }
        Icon::Xbox => {
            p.circle_stroke(c, 8.0, st);
            p.line_segment([c + vec2(-4.5, -4.5), c + vec2(4.5, 4.5)], Stroke::new(2.0, col));
            p.line_segment([c + vec2(4.5, -4.5), c + vec2(-4.5, 4.5)], Stroke::new(2.0, col));
        }
        Icon::Tag => {
            let pts = vec![
                c + vec2(-6.0, -2.0),
                c + vec2(-2.0, -6.0),
                c + vec2(5.5, -6.0),
                c + vec2(5.5, 3.0),
                c + vec2(-0.5, 5.5),
            ];
            p.add(Shape::closed_line(pts, st));
            p.circle_filled(c + vec2(2.5, -3.0), 1.2, col);
        }
        Icon::Chart => {
            p.rect_filled(Rect::from_center_size(c + vec2(-5.0, 2.0), vec2(3.0, 6.0)), CornerRadius::same(1), col);
            p.rect_filled(Rect::from_center_size(c + vec2(0.0, 0.0), vec2(3.0, 10.0)), CornerRadius::same(1), col);
            p.rect_filled(Rect::from_center_size(c + vec2(5.0, -2.0), vec2(3.0, 14.0)), CornerRadius::same(1), col);
        }
        Icon::Gamepad => {
            let body = Rect::from_center_size(c, vec2(16.0, 10.0));
            p.rect_stroke(body, CornerRadius::same(3), st, StrokeKind::Inside);
            p.line_segment([c + vec2(-4.5, -2.0), c + vec2(-4.5, 2.0)], Stroke::new(1.4, col));
            p.line_segment([c + vec2(-6.5, 0.0), c + vec2(-2.5, 0.0)], Stroke::new(1.4, col));
            p.circle_filled(c + vec2(4.5, -1.5), 1.2, col);
            p.circle_filled(c + vec2(3.0, 1.5), 1.2, col);
        }
        Icon::Save => {
            let r = Rect::from_center_size(c, vec2(13.0, 13.0));
            p.rect_stroke(r, CornerRadius::same(2), st, StrokeKind::Inside);
            p.rect_filled(Rect::from_center_size(c + vec2(0.0, -3.0), vec2(7.0, 5.0)), CornerRadius::same(1), col);
            p.rect_stroke(Rect::from_center_size(c + vec2(0.0, 3.5), vec2(8.0, 4.0)), CornerRadius::same(1), Stroke::new(1.0, col), StrokeKind::Inside);
        }
        Icon::Download => {
            p.line_segment([c + vec2(0.0, -6.0), c + vec2(0.0, 3.0)], st);
            p.line_segment([c + vec2(-4.0, -1.0), c + vec2(0.0, 3.0)], st);
            p.line_segment([c + vec2(4.0, -1.0), c + vec2(0.0, 3.0)], st);
            p.line_segment([c + vec2(-6.0, 6.0), c + vec2(6.0, 6.0)], st);
        }
        Icon::Gift => {
            let r = Rect::from_center_size(c + vec2(0.0, 1.5), vec2(12.0, 9.0));
            p.rect_stroke(r, CornerRadius::same(1), st, StrokeKind::Inside);
            let lid = Rect::from_center_size(c + vec2(0.0, -3.5), vec2(14.0, 3.5));
            p.rect_stroke(lid, CornerRadius::same(1), st, StrokeKind::Inside);
            p.line_segment([c + vec2(0.0, -5.0), c + vec2(0.0, 6.0)], Stroke::new(1.2, col));
        }
    }
}

pub fn toggle(ui: &mut Ui, id_str: &str, on: &mut bool, accent: Color32) -> egui::Response {
    let (rect, mut resp) = ui.allocate_exact_size(vec2(42.0, 22.0), Sense::click());
    if resp.clicked() {
        *on = !*on;
        resp.mark_changed();
    }
    let t = ui.ctx().animate_bool(Id::unique(id_str), *on);
    let bg = Color32::from_rgb(38, 41, 54).lerp_to_gamma(accent, t);
    let p = ui.painter();
    p.rect_filled(rect, CornerRadius::same(11), bg);
    let cx = egui::lerp((rect.left() + 11.0)..=(rect.right() - 11.0), t);
    p.circle_filled(pos2(cx, rect.center().y), 8.0, WHITE);
    resp.on_hover_cursor(egui::CursorIcon::PointingHand)
}

pub fn section(ui: &mut Ui, title: &str, th: ThemeColors, add: impl FnOnce(&mut Ui)) {
    egui::Frame::new()
        .fill(th.card)
        .stroke(Stroke::new(1.0, th.card_border))
        .corner_radius(12.0)
        .inner_margin(Margin::same(20))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.label(RichText::new(title).size(15.5).strong().color(WHITE));
            ui.add_space(10.0);
            add(ui);
        });
    ui.add_space(14.0);
}

pub fn row(ui: &mut Ui, title: &str, desc: &str, control: impl FnOnce(&mut Ui)) {
    ui.horizontal(|ui| {
        ui.vertical(|ui| {
            ui.label(RichText::new(title).size(13.5).strong().color(WHITE));
            ui.add_space(2.0);
            ui.label(RichText::new(desc).size(12.0).color(DIM));
        });
        ui.with_layout(Layout::right_to_left(Align::Center), control);
    });
    ui.add_space(4.0);
}

// ───────────────────────────── Cartes, Liste & Bannière ─────────────────────────────
pub fn cover_colors(name: &str) -> (Color32, Color32) {
    let mut h: u32 = 2166136261;
    for b in name.bytes() {
        h ^= b as u32;
        h = h.wrapping_mul(16777619);
    }
    let hue = (h % 360) as f32 / 360.0;
    (
        Hsva::new(hue, 0.45, 0.36, 1.0).into(),
        Hsva::new((hue + 0.06) % 1.0, 0.55, 0.14, 1.0).into(),
    )
}

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

pub fn pill(p: &egui::Painter, pos: Pos2, text: &str, bg: Color32, fg: Color32) -> f32 {
    let gal = p.layout_no_wrap(text.to_string(), FontId::proportional(10.0), fg);
    let w = gal.size().x + 10.0;
    let r = Rect::from_min_size(pos, vec2(w, gal.size().y + 5.0));
    p.rect_filled(r, CornerRadius::same(5), bg);
    p.galley(pos + vec2(5.0, 2.5), gal, fg);
    w
}

pub fn trunc(s: &str, n: usize) -> String {
    if s.chars().count() > n {
        format!("{}…", s.chars().take(n - 1).collect::<String>())
    } else {
        s.to_string()
    }
}

pub fn hero_banner(
    ui: &mut Ui,
    g: &Game,
    play_mins: u32,
    tex: Option<&TextureHandle>,
    accent: Color32,
    th: ThemeColors,
) -> (bool, bool) {
    let width = ui.available_width();
    let banner_h = 166.0;
    let (rect, resp) = ui.allocate_exact_size(vec2(width, banner_h), Sense::hover());
    let hov_t = ui
        .ctx()
        .animate_bool(Id::unique(("hero_hov", &g.id)), resp.hovered());

    let p = ui.painter().clone();
    let clip = p.with_clip_rect(rect);
    let (c1, c2) = cover_colors(&g.name);
    draw_gradient_rect(&clip, rect, c2, th.card);

    // Halo lumineux dynamique lors du survol
    if hov_t > 0.01 {
        let glow_col = accent.gamma_multiply(0.12 * hov_t);
        p.rect_filled(rect.expand(2.0), CornerRadius::same(14), glow_col);
    }

    let thumb_rect = Rect::from_min_size(rect.min + vec2(16.0, 16.0), vec2(96.0, 134.0));
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
        CornerRadius::same(8),
        Stroke::new(1.0, th.card_border),
        StrokeKind::Inside,
    );

    let text_x = thumb_rect.right() + 22.0;

    // Badge "REPRENDRE LA SESSION"
    pill(
        &clip,
        pos2(text_x, rect.top() + 18.0),
        "REPRENDRE LA SESSION",
        accent.gamma_multiply(0.25),
        WHITE,
    );

    clip.text(
        pos2(text_x, rect.top() + 46.0),
        Align2::LEFT_TOP,
        &g.name,
        FontId::proportional(22.0),
        WHITE,
    );

    let sub_info = format!("{}  •  {}", g.source.label(), format_playtime(play_mins));
    clip.text(
        pos2(text_x, rect.top() + 80.0),
        Align2::LEFT_TOP,
        sub_info,
        FontId::proportional(12.5),
        DIM,
    );

    // Boutons Jouer et Propriétés
    let btn_play_rect = Rect::from_min_size(pos2(text_x, rect.top() + 112.0), vec2(150.0, 36.0));
    let play_resp = ui.interact(btn_play_rect, Id::unique(("hero_play", &g.id)), Sense::click());
    let play_hov = play_resp.hovered();
    let play_bg = if play_hov { accent.lerp_to_gamma(WHITE, 0.15) } else { accent };
    clip.rect_filled(btn_play_rect, CornerRadius::same(8), play_bg);
    clip.text(
        btn_play_rect.center(),
        Align2::CENTER_CENTER,
        "▶  Jouer maintenant",
        FontId::proportional(13.0),
        WHITE,
    );

    let btn_prop_rect = Rect::from_min_size(pos2(btn_play_rect.right() + 12.0, rect.top() + 112.0), vec2(120.0, 36.0));
    let prop_resp = ui.interact(btn_prop_rect, Id::unique(("hero_prop", &g.id)), Sense::click());
    let prop_hov = prop_resp.hovered();
    let prop_bg = if prop_hov { th.card_hover } else { th.card };
    clip.rect_filled(btn_prop_rect, CornerRadius::same(8), prop_bg);
    clip.rect_stroke(
        btn_prop_rect,
        CornerRadius::same(8),
        Stroke::new(1.0, if prop_hov { WHITE.gamma_multiply(0.3) } else { th.card_border }),
        StrokeKind::Inside,
    );
    clip.text(
        btn_prop_rect.center(),
        Align2::CENTER_CENTER,
        "⚙  Propriétés",
        FontId::proportional(12.5),
        if prop_hov { WHITE } else { DIM },
    );

    p.rect_stroke(
        rect,
        CornerRadius::same(14),
        Stroke::new(1.0, th.card_border.lerp_to_gamma(accent, hov_t * 0.6)),
        StrokeKind::Inside,
    );

    (play_resp.clicked(), prop_resp.clicked())
}

#[allow(clippy::too_many_arguments)]
pub fn game_card(
    ui: &mut Ui,
    g: &Game,
    tex: Option<&TextureHandle>,
    play_mins: u32,
    fav: bool,
    broken: bool,
    custom: bool,
    accent: Color32,
    scale: f32,
    th: ThemeColors,
) -> (egui::Response, bool) {
    let scale = scale.clamp(0.7, 1.4);
    let card_w = 188.0 * scale;
    let card_h = 264.0 * scale;
    let (base_rect, resp) = ui.allocate_exact_size(vec2(card_w, card_h), Sense::click());
    let resp = resp.on_hover_cursor(egui::CursorIcon::PointingHand);
    if !ui.is_rect_visible(base_rect) {
        return (resp, false);
    }

    let hov_t = ui
        .ctx()
        .animate_bool(Id::unique(("card_hov", &g.id)), resp.hovered());

    let p = ui.painter().clone();
    let rect = base_rect.translate(vec2(0.0, -5.0 * hov_t * scale));
    let round = CornerRadius::same((12.0 * scale).round() as u8);

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

    // Halo lumineux / Ambient glow lors du survol
    if hov_t > 0.02 {
        let glow_outer = src_col.gamma_multiply(0.22 * hov_t);
        let glow_inner = src_col.gamma_multiply(0.40 * hov_t);
        p.rect_filled(
            rect.expand(12.0 * hov_t * scale),
            CornerRadius::same(18),
            glow_outer,
        );
        p.rect_filled(
            rect.expand(4.0 * hov_t * scale),
            CornerRadius::same(14),
            glow_inner,
        );
    }

    p.rect_filled(rect, round, th.card);

    let inner = rect.shrink(3.0 * scale);
    let clip = p.with_clip_rect(inner);

    if let Some(t) = tex {
        let zoom = 0.03 * hov_t;
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
            FontId::proportional(44.0 * scale),
            Color32::from_white_alpha(40),
        );
    }

    let fade_h = 115.0 * scale;
    let fade_rect = Rect::from_min_max(pos2(inner.left(), inner.bottom() - fade_h), inner.right_bottom());
    draw_vertical_fade(
        &clip,
        fade_rect,
        Color32::TRANSPARENT,
        Color32::from_black_alpha(238),
    );

    pill(
        &clip,
        inner.min + vec2(8.0 * scale, 8.0 * scale),
        g.source.label(),
        Color32::from_black_alpha(190),
        src_col,
    );

    let star_center = pos2(inner.right() - 20.0 * scale, inner.top() + 20.0 * scale);
    let star_rect = Rect::from_center_size(star_center, vec2(26.0 * scale, 26.0 * scale));
    let star_hovered = resp
        .hover_pos()
        .map(|hp| star_rect.contains(hp))
        .unwrap_or(false);
    let star_clicked = resp.clicked() && star_hovered;

    if fav || hov_t > 0.05 {
        clip.circle_filled(star_center, 12.0 * scale, Color32::from_black_alpha(175));
        let s_col = if fav {
            GOLD
        } else if star_hovered {
            WHITE
        } else {
            DIM
        };
        draw_icon(&clip, Icon::Star, star_center, s_col);
    }

    if broken {
        pill(
            &clip,
            pos2(inner.left() + 10.0 * scale, inner.bottom() - 78.0 * scale),
            "EXÉCUTABLE MANQUANT",
            RED.gamma_multiply(0.85),
            WHITE,
        );
    } else if custom {
        pill(
            &clip,
            pos2(inner.left() + 10.0 * scale, inner.bottom() - 78.0 * scale),
            "EXE PERSONNALISÉ",
            Color32::from_black_alpha(180),
            WHITE,
        );
    }

    let title_font_size = (14.5 * scale.max(0.85)).clamp(11.0, 20.0);
    clip.text(
        pos2(inner.left() + 10.0 * scale, inner.bottom() - 50.0 * scale),
        Align2::LEFT_TOP,
        trunc(&g.name, 20),
        FontId::proportional(title_font_size),
        WHITE,
    );

    let time_pos = pos2(inner.left() + 16.0 * scale, inner.bottom() - 20.0 * scale);
    draw_icon(&clip, Icon::Clock, time_pos, DIM);
    clip.text(
        pos2(inner.left() + 26.0 * scale, inner.bottom() - 27.0 * scale),
        Align2::LEFT_TOP,
        format_playtime(play_mins),
        FontId::proportional((11.5 * scale.max(0.85)).clamp(9.0, 16.0)),
        DIM,
    );

    let border_col = th.card_border.lerp_to_gamma(accent, hov_t * 0.75);
    p.rect_stroke(
        rect,
        round,
        Stroke::new(1.0, border_col),
        StrokeKind::Inside,
    );

    (resp, star_clicked)
}

pub fn game_list_row(
    ui: &mut Ui,
    g: &Game,
    tex: Option<&TextureHandle>,
    play_mins: u32,
    fav: bool,
    accent: Color32,
    th: ThemeColors,
) -> (egui::Response, bool, bool, bool) {
    let width = ui.available_width();
    let (rect, resp) = ui.allocate_exact_size(vec2(width, 58.0), Sense::click());
    let hov_t = ui
        .ctx()
        .animate_bool(Id::unique(("row_hov", &g.id)), resp.hovered());

    let p = ui.painter();
    let bg = th.card.lerp_to_gamma(th.card_hover, hov_t);
    p.rect_filled(rect, CornerRadius::same(10), bg);
    p.rect_stroke(
        rect,
        CornerRadius::same(10),
        Stroke::new(1.0, th.card_border.lerp_to_gamma(accent, hov_t * 0.4)),
        StrokeKind::Inside,
    );

    // Miniature
    let thumb = Rect::from_min_size(rect.min + vec2(10.0, 7.0), vec2(34.0, 44.0));
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

    // Titre & Plateforme
    p.text(
        pos2(thumb.right() + 14.0, rect.top() + 12.0),
        Align2::LEFT_TOP,
        &g.name,
        FontId::proportional(14.5),
        WHITE,
    );
    p.text(
        pos2(thumb.right() + 14.0, rect.top() + 32.0),
        Align2::LEFT_TOP,
        format!("{}  •  {}", g.source.label(), format_playtime(play_mins)),
        FontId::proportional(11.5),
        DIM,
    );

    // Zones d'action à droite
    let play_rect = Rect::from_center_size(pos2(rect.right() - 48.0, rect.center().y), vec2(68.0, 30.0));
    let opt_rect = Rect::from_center_size(pos2(rect.right() - 104.0, rect.center().y), vec2(30.0, 30.0));
    let fav_rect = Rect::from_center_size(pos2(rect.right() - 140.0, rect.center().y), vec2(30.0, 30.0));

    let hp = resp.hover_pos();
    let play_hov = hp.map(|pos| play_rect.contains(pos)).unwrap_or(false);
    let opt_hov = hp.map(|pos| opt_rect.contains(pos)).unwrap_or(false);
    let fav_hov = hp.map(|pos| fav_rect.contains(pos)).unwrap_or(false);

    draw_icon(
        p,
        Icon::Star,
        fav_rect.center(),
        if fav {
            GOLD
        } else if fav_hov {
            WHITE
        } else {
            DIM
        },
    );
    draw_icon(
        p,
        Icon::Sliders,
        opt_rect.center(),
        if opt_hov { WHITE } else { DIM },
    );

    p.rect_filled(
        play_rect,
        CornerRadius::same(7),
        if play_hov {
            accent.lerp_to_gamma(WHITE, 0.15)
        } else {
            accent
        },
    );
    p.text(
        play_rect.center(),
        Align2::CENTER_CENTER,
        "Jouer",
        FontId::proportional(12.0),
        WHITE,
    );

    let clicked = resp.clicked();
    (
        resp.on_hover_cursor(egui::CursorIcon::PointingHand),
        clicked && play_hov,
        clicked && fav_hov,
        clicked && opt_hov,
    )
}
