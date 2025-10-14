use std::sync::mpsc::Sender;

use eframe::egui;

use crate::app::message::Msg;
use crate::app::state::{Model, UiState};

use super::panel::menu;
use super::panel::central;
use super::panel::bottom;
use super::window::options;
use super::window::about;

pub struct View;

impl View {
    pub fn render(
        ctx: &egui::Context,
        model: &Model,
        ui: &UiState,
        sender: &Sender<Msg>,
    ) {
        menu::render(ctx, model, ui, sender);
        bottom::render(ctx, model, ui, sender);
        central::render(ctx, model, ui, sender);

        if ui.show_options {
            options::render(ctx, model, ui, sender);
        }

        if ui.show_about {
            about::render(ctx, sender);
        }
    }
}
