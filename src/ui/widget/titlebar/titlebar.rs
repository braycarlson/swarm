use eframe::egui::{
    self,
    Align,
    Color32,
    Frame,
    Layout,
    Rect,
    RichText,
    Sense,
    vec2,
};

use crate::constants::APP_NAME;
use crate::ui::widget::titlebar::icon::{TitleIcon, draw_title_icon, hover_rectangle};

pub struct TitleBar;

impl TitleBar {
    pub fn render(ui: &mut egui::Ui) {
        let context = ui.ctx().clone();
        let title_bar_height = 32.0;

        egui::Panel::top("custom_title_bar")
            .frame(
                Frame::default()
                    .fill(context.global_style().visuals.window_fill)
                    .inner_margin(0.0),
            )
            .exact_size(title_bar_height)
            .show_inside(ui, |ui| {
                let title_bar_rectangle = ui.max_rect();

                let button_width = 46.0;
                let buttons_number  = 3;
                let button_width_total = button_width * buttons_number as f32;

                let drag_rectangle = Rect::from_min_size(
                    title_bar_rectangle.min,
                    egui::vec2(title_bar_rectangle.width() - button_width_total, title_bar_height),
                );

                let drag_response = ui.interact(drag_rectangle, ui.id().with("title_drag"), Sense::drag());

                if drag_response.dragged() {
                    context.send_viewport_cmd(egui::ViewportCommand::StartDrag);
                }

                ui.horizontal_centered(|ui| {
                    ui.add_space(10.0);

                    ui.allocate_ui_with_layout(
                        egui::vec2(drag_rectangle.width() - 20.0, title_bar_height),
                        Layout::left_to_right(Align::Center),
                        |ui| {
                            ui.label(RichText::new(APP_NAME).size(13.0));
                        },
                    );

                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        let close_rectangle = Rect::from_min_size(
                            title_bar_rectangle.right_top() - vec2(button_width, 0.0),
                            egui::vec2(button_width, title_bar_height),
                        );

                        let close_response = ui.interact(close_rectangle, ui.id().with("close"), Sense::click());

                        if close_response.hovered() {
                            let painter = ui.painter().with_clip_rect(title_bar_rectangle);
                            painter.rect_filled(hover_rectangle(close_rectangle, title_bar_rectangle), 0.0, Color32::from_rgb(232, 17, 35));
                        }

                        {
                            let foreground = if close_response.hovered() {
                                Color32::WHITE
                            } else {
                                ui.style().visuals.text_color()
                            };
                            let painter = ui.painter().with_clip_rect(title_bar_rectangle);
                            draw_title_icon(&painter, close_rectangle, TitleIcon::Close, foreground);
                        }

                        if close_response.clicked() {
                            context.send_viewport_cmd(egui::ViewportCommand::Close);
                        }

                        ui.add_space(button_width);

                        let is_maximized = ui.input(|i| i.viewport().maximized.unwrap_or(false));

                        let maximize_rectangle = Rect::from_min_size(
                            title_bar_rectangle.right_top() - vec2(button_width * 2.0, 0.0),
                            egui::vec2(button_width, title_bar_height),
                        );

                        let maximize_response =
                            ui.interact(maximize_rectangle, ui.id().with("maximize"), Sense::click());

                        if maximize_response.hovered() {
                            let painter = ui.painter().with_clip_rect(title_bar_rectangle);
                            painter.rect_filled(hover_rectangle(maximize_rectangle, title_bar_rectangle), 0.0, context.global_style().visuals.widgets.hovered.weak_bg_fill);
                        }

                        {
                            let icon = if is_maximized { TitleIcon::Restore } else { TitleIcon::Maximize };
                            let painter = ui.painter().with_clip_rect(title_bar_rectangle);
                            draw_title_icon(&painter, maximize_rectangle, icon, ui.style().visuals.text_color());
                        }

                        if maximize_response.clicked() {
                            context.send_viewport_cmd(egui::ViewportCommand::Maximized(!is_maximized));
                        }

                        ui.add_space(button_width);

                        let minimize_rectangle = Rect::from_min_size(
                            title_bar_rectangle.right_top() - vec2(button_width * 3.0, 0.0),
                            egui::vec2(button_width, title_bar_height),
                        );

                        let minimize_response =
                            ui.interact(minimize_rectangle, ui.id().with("minimize"), Sense::click());

                        if minimize_response.hovered() {
                            let painter = ui.painter().with_clip_rect(title_bar_rectangle);
                            painter.rect_filled(hover_rectangle(minimize_rectangle, title_bar_rectangle), 0.0, context.global_style().visuals.widgets.hovered.weak_bg_fill);
                        }

                        {
                            let painter = ui.painter().with_clip_rect(title_bar_rectangle);
                            draw_title_icon(&painter, minimize_rectangle, TitleIcon::Minimize, ui.style().visuals.text_color());
                        }

                        if minimize_response.clicked() {
                            context.send_viewport_cmd(egui::ViewportCommand::Minimized(true));
                        }
                    });
                });
            });
    }
}
