use eframe::egui::{self, Align, Frame, Layout, Rect, RichText, Sense, vec2};

use crate::constants::{APP_NAME, TITLE_BAR_HEIGHT_PIXELS, TITLE_BUTTON_WIDTH_PIXELS};

use super::icon::{CLOSE_HOVER_COLOR, IconButton, TitleIcon};

const BUTTON_COUNT: u8 = 3;
const LABEL_INSET_PIXELS: f32 = 10.0;

fn button_rectangle(bar_rectangle: Rect, slot: u8) -> Rect {
    assert!(slot >= 1);
    assert!(slot <= BUTTON_COUNT);

    Rect::from_min_size(
        bar_rectangle.right_top() - vec2(TITLE_BUTTON_WIDTH_PIXELS * f32::from(slot), 0.0),
        vec2(TITLE_BUTTON_WIDTH_PIXELS, TITLE_BAR_HEIGHT_PIXELS),
    )
}

pub fn render(ui: &mut egui::Ui) {
    let fill = ui.ctx().global_style().visuals.window_fill;

    egui::Panel::top("custom_title_bar")
        .frame(Frame::default().fill(fill).inner_margin(0.0))
        .exact_size(TITLE_BAR_HEIGHT_PIXELS)
        .show_inside(ui, |panel| {
            let bar_rectangle = panel.max_rect();

            render_drag_area(panel, bar_rectangle);
        });
}

fn render_buttons(ui: &egui::Ui, bar_rectangle: Rect) {
    let context = ui.ctx();
    let fill_hover = context.global_style().visuals.widgets.hovered.weak_bg_fill;
    let is_maximized = ui.input(|input| input.viewport().maximized.unwrap_or(false));

    let close = IconButton {
        clip: bar_rectangle,
        fill_hover: CLOSE_HOVER_COLOR,
        icon: TitleIcon::Close,
        id_salt: "close",
        rectangle: button_rectangle(bar_rectangle, 1),
    };

    if close.render(ui) {
        context.send_viewport_cmd(egui::ViewportCommand::Close);
    }

    let maximize = IconButton {
        clip: bar_rectangle,
        fill_hover,
        icon: if is_maximized {
            TitleIcon::Restore
        } else {
            TitleIcon::Maximize
        },
        id_salt: "maximize",
        rectangle: button_rectangle(bar_rectangle, 2),
    };

    if maximize.render(ui) {
        context.send_viewport_cmd(egui::ViewportCommand::Maximized(!is_maximized));
    }

    let minimize = IconButton {
        clip: bar_rectangle,
        fill_hover,
        icon: TitleIcon::Minimize,
        id_salt: "minimize",
        rectangle: button_rectangle(bar_rectangle, 3),
    };

    if minimize.render(ui) {
        context.send_viewport_cmd(egui::ViewportCommand::Minimized(true));
    }
}

fn render_drag_area(ui: &mut egui::Ui, bar_rectangle: Rect) {
    let buttons_width = TITLE_BUTTON_WIDTH_PIXELS * f32::from(BUTTON_COUNT);

    let drag_rectangle = Rect::from_min_size(
        bar_rectangle.min,
        vec2(
            bar_rectangle.width() - buttons_width,
            TITLE_BAR_HEIGHT_PIXELS,
        ),
    );

    let drag_response = ui.interact(drag_rectangle, ui.id().with("title_drag"), Sense::drag());

    if drag_response.dragged() {
        ui.ctx().send_viewport_cmd(egui::ViewportCommand::StartDrag);
    }

    ui.horizontal_centered(|row| {
        row.add_space(LABEL_INSET_PIXELS);

        row.allocate_ui_with_layout(
            vec2(
                LABEL_INSET_PIXELS.mul_add(-2.0, drag_rectangle.width()),
                TITLE_BAR_HEIGHT_PIXELS,
            ),
            Layout::left_to_right(Align::Center),
            |cell| {
                cell.label(RichText::new(APP_NAME).size(13.0));
            },
        );
    });

    render_buttons(ui, bar_rectangle);
}
