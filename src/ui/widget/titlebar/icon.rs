use eframe::egui::{
    Color32,
    Painter,
    Rect,
    Stroke,
    pos2,
    vec2,
};

#[derive(Clone, Copy)]
pub enum TitleIcon {
    Close,
    Minimize,
    Maximize,
    Restore,
}

const ICON_SCALE: f32 = 0.25;
const STROKE_FRAC: f32 = 0.05;
const STROKE_MIN: f32 = 0.5;
const STROKE_MAX: f32 = 1.0;
const HOVER_INSET_PX: f32 = 1.0;

pub fn hover_rect(r: Rect, clip: Rect) -> Rect {
    let rr = Rect::from_min_max(pos2(r.min.x + HOVER_INSET_PX, r.min.y), r.max);
    let inter = rr.intersect(clip);
    Rect::from_min_max(pos2(inter.min.x + 0.5, inter.min.y), pos2(inter.max.x, inter.max.y - 0.5))
}

pub fn draw_rect_outline(p: &Painter, r: Rect, s: Stroke) {
    p.line_segment([r.left_top(),     r.right_top()],    s);
    p.line_segment([r.right_top(),    r.right_bottom()], s);
    p.line_segment([r.right_bottom(), r.left_bottom()],  s);
    p.line_segment([r.left_bottom(),  r.left_top()],     s);
}

pub fn draw_title_icon(p: &Painter, rect: Rect, icon: TitleIcon, color: Color32) {
    let side = rect.width().min(rect.height());
    let icon_side = side * ICON_SCALE;

    let y_off = match icon {
        TitleIcon::Restore => 1.0,
        _ => 0.0,
    };

    let icon_rect = Rect::from_center_size(rect.center() + vec2(0.0, y_off), vec2(icon_side, icon_side));

    let stroke_w = (side * STROKE_FRAC).clamp(STROKE_MIN, STROKE_MAX);
    let s = Stroke { width: stroke_w, color };

    match icon {
        TitleIcon::Close => {
            let r = icon_rect.shrink(icon_side * 0.12);
            p.line_segment([r.left_top(),  r.right_bottom()], s);
            p.line_segment([r.right_top(), r.left_bottom()],  s);
        }
        TitleIcon::Minimize => {
            let r = icon_rect;
            let y = r.bottom() - r.height() * 0.18;
            p.line_segment([pos2(r.left(), y), pos2(r.right(), y)], s);
        }
        TitleIcon::Maximize => {
            let r = icon_rect.shrink(stroke_w * 0.75);
            draw_rect_outline(p, r, s);
        }
        TitleIcon::Restore => {
            let r = icon_rect.shrink(stroke_w * 0.75);
            let off = vec2(r.width() * 0.24, r.height() * 0.24);

            let front = r;
            let back  = Rect::from_min_max(
                r.min + vec2(off.x, -off.y),
                r.max + vec2(off.x, -off.y),
            );

            p.line_segment([back.left_top(),  back.right_top()],    s);
            p.line_segment([back.right_top(), back.right_bottom()], s);

            draw_rect_outline(p, front, s);
        }
    }
}
