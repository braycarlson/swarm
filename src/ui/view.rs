use std::sync::mpsc::Sender;

use eframe::egui;

use crate::app::message::Msg;
use crate::app::state::{Model, UiState};

use super::panel::menu;
use super::panel::central;
use super::panel::bottom;
use super::window::options;
use super::window::about;
use super::window::preset;

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

        if ui.show_save_preset {
            preset::render_save(ctx, ui, sender);
        }

        if ui.show_load_preset {
            preset::render_load(ctx, model, sender);
        }
    }
}
