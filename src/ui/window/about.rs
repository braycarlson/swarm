use std::sync::mpsc::Sender;

use eframe::egui;

use crate::app::message::{App, Message};
use crate::constants::{APP_NAME, APP_VERSION};
use crate::ui::widget::titlebar::icon::{TitleIcon, draw_title_icon, hover_rectangle};

pub fn render(context: &egui::Context, sender: &Sender<Message>) {
    let center = context.content_rect().center();
    let title_bar_height = 32.0;
    let button_width = 46.0;

    egui::Window::new("about")
        .title_bar(false)
        .resizable(false)
        .fixed_size([420.0, 320.0])
        .collapsible(false)
        .pivot(egui::Align2::CENTER_CENTER)
        .current_pos(center)
        .show(context, |ui| {
            let content_rect = ui.max_rect();

            let title_rectangle = egui::Rect::from_min_size(
                content_rect.min,
                egui::vec2(content_rect.width(), title_bar_height),
            );

            ui.allocate_ui_with_layout(
                egui::vec2(ui.available_width(), title_bar_height),
                egui::Layout::left_to_right(egui::Align::Center),
                |ui| {
                    ui.set_min_height(title_bar_height);
                    ui.add_space(8.0);
                    ui.label(egui::RichText::new("About").size(14.0));
                }
            );

            let close_rectangle = egui::Rect::from_min_size(
                title_rectangle.right_top() - egui::vec2(button_width, 0.0),
                egui::vec2(button_width, title_bar_height),
            );

            let close_response = ui.interact(
                close_rectangle,
                ui.id().with("about_close"),
                egui::Sense::click(),
            );

            if close_response.hovered() {
                let painter = ui.painter().with_clip_rect(title_rectangle);
                painter.rect_filled(
                    hover_rectangle(close_rectangle, title_rectangle),
                    0.0,
                    egui::Color32::from_rgb(232, 17, 35),
                );
            }

            {
                let foreground = if close_response.hovered() {
                    egui::Color32::WHITE
                } else {
                    ui.visuals().text_color()
                };

                let painter = ui.painter().with_clip_rect(title_rectangle);
                draw_title_icon(&painter, close_rectangle, TitleIcon::Close, foreground);
            }

            if close_response.clicked() {
                sender.send(Message::App(App::AboutClosed)).ok();
            }

            ui.separator();

            egui::Frame::dark_canvas(ui.style())
                .fill(ui.visuals().extreme_bg_color)
                .inner_margin(8.0)
                .stroke(egui::Stroke::NONE)
                .show(ui, |ui| {
                    ui.vertical(|ui| {
                        ui.vertical_centered(|ui| {
                            ui.heading(format!("{} [v{}]", APP_NAME, APP_VERSION));
                        });

                        ui.add_space(60.0);

                        ui.add(egui::Label::new(
                            "swarm is a developer tool for generating project context. \
                             It allows you browse and select file(s) from a directory tree, then copy \
                             the structure or content in multiple formats, such as: plain text, Markdown, JSON, or XML."
                        ).wrap());

                        ui.add_space(15.0);

                        ui.hyperlink_to("Homepage", "https://github.com/braycarlson/swarm");
                        ui.hyperlink_to("Editor Extension", "https://github.com/braycarlson/swarm_extension/");
                        ui.label("License: MIT");

                        ui.add_space(3.0);
                    });
                });
        });
}
