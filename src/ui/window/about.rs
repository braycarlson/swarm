use eframe::egui;

use crate::app::message::{App, Message, MessageSender};
use crate::constants::{APP_NAME, APP_VERSION};
use crate::ui::widget::window::{Modal, render_modal};

const ABOUT_SIZE: [f32; 2] = [420.0, 320.0];
const DESCRIPTION: &str = "swarm is a developer tool for generating project context. It allows \
                           you browse and select file(s) from a directory tree, then copy the \
                           structure or content in multiple formats, such as: plain text, \
                           Markdown, JSON, or XML.";

pub fn render(context: &egui::Context, sender: &MessageSender) {
    let dialog = Modal {
        identifier: "about",
        size: ABOUT_SIZE,
        title: "About",
    };

    let closed = render_modal(context, &dialog, |modal| {
        egui::Frame::dark_canvas(modal.style())
            .fill(modal.visuals().extreme_bg_color)
            .inner_margin(8.0)
            .stroke(egui::Stroke::NONE)
            .show(modal, render_body);
    });

    if closed {
        sender.send(Message::App(App::AboutClosed));
    }
}

fn render_body(ui: &mut egui::Ui) {
    ui.vertical(|column| {
        column.vertical_centered(|centered| {
            centered.heading(format!("{APP_NAME} [v{APP_VERSION}]"));
        });

        column.add_space(60.0);
        column.add(egui::Label::new(DESCRIPTION).wrap());
        column.add_space(15.0);
        column.hyperlink_to("Homepage", "https://github.com/braycarlson/swarm");

        column.hyperlink_to(
            "Editor Extension",
            "https://github.com/braycarlson/swarm_extension/",
        );

        column.label("License: MIT");
        column.add_space(3.0);
    });
}
