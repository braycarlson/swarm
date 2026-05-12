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
const STROKE_FRACTION: f32 = 0.05;
const STROKE_MINIMUM: f32 = 0.5;
const STROKE_MAXIMUM: f32 = 1.0;
const HOVER_INSET_PIXELS: f32 = 1.0;

pub fn hover_rectangle(r: Rect, clip: Rect) -> Rect {
    let rr = Rect::from_min_max(pos2(r.min.x + HOVER_INSET_PIXELS, r.min.y), r.max);
    let intersection = rr.intersect(clip);
    Rect::from_min_max(pos2(intersection.min.x + 0.5, intersection.min.y), pos2(intersection.max.x, intersection.max.y - 0.5))
}

pub fn draw_rectangle_outline(p: &Painter, rectangle: Rect, s: Stroke) {
    p.line_segment([rectangle.left_top(),     rectangle.right_top()],    s);
    p.line_segment([rectangle.right_top(),    rectangle.right_bottom()], s);
    p.line_segment([rectangle.right_bottom(), rectangle.left_bottom()],  s);
    p.line_segment([rectangle.left_bottom(),  rectangle.left_top()],     s);
}

pub fn draw_title_icon(p: &Painter, rectangle: Rect, icon: TitleIcon, color: Color32) {
    let side = rectangle.width().min(rectangle.height());
    let icon_side = side * ICON_SCALE;

    let y_offset = match icon {
        TitleIcon::Restore => 1.0,
        _ => 0.0,
    };

    let icon_rectangle = Rect::from_center_size(rectangle.center() + vec2(0.0, y_offset), vec2(icon_side, icon_side));

    let stroke_width = (side * STROKE_FRACTION).clamp(STROKE_MINIMUM, STROKE_MAXIMUM);
    let s = Stroke { width: stroke_width, color };

    match icon {
        TitleIcon::Close => {
            let r = icon_rectangle.shrink(icon_side * 0.12);
            p.line_segment([r.left_top(),  r.right_bottom()], s);
            p.line_segment([r.right_top(), r.left_bottom()],  s);
        }
        TitleIcon::Minimize => {
            let r = icon_rectangle;
            let y = r.bottom() - r.height() * 0.18;
            p.line_segment([pos2(r.left(), y), pos2(r.right(), y)], s);
        }
        TitleIcon::Maximize => {
            let r = icon_rectangle.shrink(stroke_width * 0.75);
            draw_rectangle_outline(p, r, s);
        }
        TitleIcon::Restore => {
            let r = icon_rectangle.shrink(stroke_width * 0.75);
            let off = vec2(r.width() * 0.24, r.height() * 0.24);

            let front = r;
            let back  = Rect::from_min_max(
                r.min + vec2(off.x, -off.y),
                r.max + vec2(off.x, -off.y),
            );

            p.line_segment([back.left_top(),  back.right_top()],    s);
            p.line_segment([back.right_top(), back.right_bottom()], s);

            draw_rectangle_outline(p, front, s);
        }
    }
}
