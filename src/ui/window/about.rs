use std::sync::mpsc::Sender;

use eframe::egui;

use crate::app::message::{App, Msg};
use crate::constants::{APP_NAME, APP_VERSION};

pub fn render(ctx: &egui::Context, sender: &Sender<Msg>) {
    let center = ctx.content_rect().center();

    egui::Window::new(egui::RichText::new("About").size(14.0))
        .resizable(false)
        .fixed_size([420.0, 280.0])
        .collapsible(false)
        .pivot(egui::Align2::CENTER_CENTER)
        .current_pos(center)
        .show(ctx, |ui| {
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

                        ui.add_space(ui.available_height() - 35.0);

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.button("Close").clicked() {
                                sender.send(Msg::App(App::AboutClosed)).ok();
                            }
                        });

                        ui.add_space(3.0);
                    });
                });
        });
}
