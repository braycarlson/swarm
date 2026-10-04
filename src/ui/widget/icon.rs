use eframe::egui::{Color32, Painter, Rect, Sense, Stroke, Ui, pos2, vec2};

pub const CLOSE_HOVER_COLOR: Color32 = Color32::from_rgb(232, 17, 35);
const HOVER_EDGE_PIXELS: f32 = 0.5;
const HOVER_INSET_PIXELS: f32 = 1.0;
const ICON_SCALE: f32 = 0.25;
const STROKE_FRACTION: f32 = 0.05;
const STROKE_MAXIMUM: f32 = 1.0;
const STROKE_MINIMUM: f32 = 0.5;

pub struct IconButton<'button> {
    pub clip: Rect,
    pub fill_hover: Color32,
    pub icon: TitleIcon,
    pub id_salt: &'button str,
    pub rectangle: Rect,
}

impl IconButton<'_> {
    fn hover_rectangle(&self) -> Rect {
        let inset = Rect::from_min_max(
            pos2(
                self.rectangle.min.x + HOVER_INSET_PIXELS,
                self.rectangle.min.y,
            ),
            self.rectangle.max,
        );

        let intersection = inset.intersect(self.clip);

        Rect::from_min_max(
            pos2(intersection.min.x + HOVER_EDGE_PIXELS, intersection.min.y),
            pos2(intersection.max.x, intersection.max.y - HOVER_EDGE_PIXELS),
        )
    }

    pub fn render(&self, ui: &Ui) -> bool {
        debug_assert!(self.rectangle.is_finite());
        debug_assert!(self.clip.is_finite());

        let response = ui.interact(self.rectangle, ui.id().with(self.id_salt), Sense::click());
        let hovered = response.hovered();
        let painter = ui.painter().with_clip_rect(self.clip);

        if hovered {
            painter.rect_filled(self.hover_rectangle(), 0.0, self.fill_hover);
        }

        let mut foreground = ui.visuals().text_color();

        if hovered {
            if self.icon == TitleIcon::Close {
                foreground = Color32::WHITE;
            }
        }

        draw_title_icon(&painter, self.rectangle, self.icon, foreground);

        response.clicked()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TitleIcon {
    Close,
    Maximize,
    Minimize,
    Restore,
}

fn draw_close(painter: &Painter, icon_rectangle: Rect, stroke: Stroke) {
    let inner = icon_rectangle.shrink(icon_rectangle.width() * 0.12);

    painter.line_segment([inner.left_top(), inner.right_bottom()], stroke);
    painter.line_segment([inner.right_top(), inner.left_bottom()], stroke);
}

fn draw_minimize(painter: &Painter, icon_rectangle: Rect, stroke: Stroke) {
    let y = icon_rectangle
        .height()
        .mul_add(-0.18, icon_rectangle.bottom());

    painter.line_segment(
        [
            pos2(icon_rectangle.left(), y),
            pos2(icon_rectangle.right(), y),
        ],
        stroke,
    );
}

fn draw_rectangle_outline(painter: &Painter, rectangle: Rect, stroke: Stroke) {
    painter.line_segment([rectangle.left_top(), rectangle.right_top()], stroke);
    painter.line_segment([rectangle.right_top(), rectangle.right_bottom()], stroke);
    painter.line_segment([rectangle.right_bottom(), rectangle.left_bottom()], stroke);
    painter.line_segment([rectangle.left_bottom(), rectangle.left_top()], stroke);
}

fn draw_restore(painter: &Painter, front: Rect, stroke: Stroke) {
    let offset = vec2(front.width() * 0.24, front.height() * 0.24);

    let back = Rect::from_min_max(
        front.min + vec2(offset.x, -offset.y),
        front.max + vec2(offset.x, -offset.y),
    );

    painter.line_segment([back.left_top(), back.right_top()], stroke);
    painter.line_segment([back.right_top(), back.right_bottom()], stroke);

    draw_rectangle_outline(painter, front, stroke);
}

pub fn draw_title_icon(painter: &Painter, rectangle: Rect, icon: TitleIcon, color: Color32) {
    let side = rectangle.width().min(rectangle.height());
    let icon_side = side * ICON_SCALE;
    let offset_y = if icon == TitleIcon::Restore { 1.0 } else { 0.0 };

    let icon_rectangle = Rect::from_center_size(
        rectangle.center() + vec2(0.0, offset_y),
        vec2(icon_side, icon_side),
    );

    let stroke_width = (side * STROKE_FRACTION).clamp(STROKE_MINIMUM, STROKE_MAXIMUM);

    let stroke = Stroke {
        color,
        width: stroke_width,
    };

    match icon {
        TitleIcon::Close => draw_close(painter, icon_rectangle, stroke),
        TitleIcon::Maximize => {
            draw_rectangle_outline(painter, icon_rectangle.shrink(stroke_width * 0.75), stroke);
        }
        TitleIcon::Minimize => draw_minimize(painter, icon_rectangle, stroke),
        TitleIcon::Restore => {
            draw_restore(painter, icon_rectangle.shrink(stroke_width * 0.75), stroke);
        }
    }
}
