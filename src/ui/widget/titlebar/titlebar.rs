use eframe::egui::{
    self,
    Align,
    Color32,
    Context,
    Frame,
    Layout,
    Rect,
    RichText,
    Sense,
    vec2,
};

use crate::constants::APP_NAME;
use crate::ui::widget::titlebar::icon::{TitleIcon, draw_title_icon, hover_rect};

pub struct TitleBar;

impl TitleBar {
    pub fn render(ctx: &Context) {
        let title_bar_height = 32.0;

        egui::TopBottomPanel::top("custom_title_bar")
            .frame(
                Frame::default()
                    .fill(ctx.style().visuals.window_fill)
                    .inner_margin(0.0),
            )
            .exact_height(title_bar_height)
            .show(ctx, |ui| {
                let title_bar_rect = ui.max_rect();

                let button_width = 46.0;
                let num_buttons  = 3;
                let total_button_width = button_width * num_buttons as f32;

                let drag_rect = Rect::from_min_size(
                    title_bar_rect.min,
                    egui::vec2(title_bar_rect.width() - total_button_width, title_bar_height),
                );

                let drag_response = ui.interact(drag_rect, ui.id().with("title_drag"), Sense::drag());

                if drag_response.dragged() {
                    ctx.send_viewport_cmd(egui::ViewportCommand::StartDrag);
                }

                ui.horizontal_centered(|ui| {
                    ui.add_space(10.0);

                    ui.allocate_ui_with_layout(
                        egui::vec2(drag_rect.width() - 20.0, title_bar_height),
                        Layout::left_to_right(Align::Center),
                        |ui| {
                            ui.label(RichText::new(APP_NAME).size(13.0));
                        },
                    );

                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        let close_rect = Rect::from_min_size(
                            title_bar_rect.right_top() - vec2(button_width, 0.0),
                            egui::vec2(button_width, title_bar_height),
                        );

                        let close_response = ui.interact(close_rect, ui.id().with("close"), Sense::click());

                        if close_response.hovered() {
                            let p = ui.painter().with_clip_rect(title_bar_rect);
                            p.rect_filled(hover_rect(close_rect, title_bar_rect), 0.0, Color32::from_rgb(232, 17, 35));
                        }

                        {
                            let fg = if close_response.hovered() {
                                Color32::WHITE
                            } else {
                                ui.style().visuals.text_color()
                            };
                            let p = ui.painter().with_clip_rect(title_bar_rect);
                            draw_title_icon(&p, close_rect, TitleIcon::Close, fg);
                        }

                        if close_response.clicked() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }

                        ui.add_space(button_width);

                        let is_maximized = ctx.input(|i| i.viewport().maximized.unwrap_or(false));

                        let maximize_rect = Rect::from_min_size(
                            title_bar_rect.right_top() - vec2(button_width * 2.0, 0.0),
                            egui::vec2(button_width, title_bar_height),
                        );

                        let maximize_response =
                            ui.interact(maximize_rect, ui.id().with("maximize"), Sense::click());

                        if maximize_response.hovered() {
                            let p = ui.painter().with_clip_rect(title_bar_rect);
                            p.rect_filled(hover_rect(maximize_rect, title_bar_rect), 0.0, ctx.style().visuals.widgets.hovered.weak_bg_fill);
                        }

                        {
                            let icon = if is_maximized { TitleIcon::Restore } else { TitleIcon::Maximize };
                            let p = ui.painter().with_clip_rect(title_bar_rect);
                            draw_title_icon(&p, maximize_rect, icon, ui.style().visuals.text_color());
                        }

                        if maximize_response.clicked() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Maximized(!is_maximized));
                        }

                        ui.add_space(button_width);

                        let minimize_rect = Rect::from_min_size(
                            title_bar_rect.right_top() - vec2(button_width * 3.0, 0.0),
                            egui::vec2(button_width, title_bar_height),
                        );

                        let minimize_response =
                            ui.interact(minimize_rect, ui.id().with("minimize"), Sense::click());

                        if minimize_response.hovered() {
                            let p = ui.painter().with_clip_rect(title_bar_rect);
                            p.rect_filled(hover_rect(minimize_rect, title_bar_rect), 0.0, ctx.style().visuals.widgets.hovered.weak_bg_fill);
                        }

                        {
                            let p = ui.painter().with_clip_rect(title_bar_rect);
                            draw_title_icon(&p, minimize_rect, TitleIcon::Minimize, ui.style().visuals.text_color());
                        }

                        if minimize_response.clicked() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(true));
                        }
                    });
                });
            });
    }
}
