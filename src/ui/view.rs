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
        ui: &mut egui::Ui,
        model: &Model,
        ui_state: &UiState,
        sender: &Sender<Msg>,
    ) {
        menu::render(ui, model, ui_state, sender);
        bottom::render(ui, model, ui_state, sender);
        central::render(ui, model, ui_state, sender);

        let ctx = ui.ctx();

        if ui_state.show_options {
            options::render(ctx, model, ui_state, sender);
        }

        if ui_state.show_about {
            about::render(ctx, sender);
        }

        if ui_state.show_save_preset {
            preset::render_save(ctx, ui_state, sender);
        }

        if ui_state.show_load_preset {
            preset::render_load(ctx, model, sender);
        }
    }
}
