use std::sync::mpsc::Sender;

use eframe::egui;

use crate::app::message::Message;
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
        sender: &Sender<Message>,
    ) {
        menu::render(ui, model, ui_state, sender);
        bottom::render(ui, model, ui_state, sender);
        central::render(ui, model, ui_state, sender);

        let ctx = ui.ctx();

        if ui_state.options_show {
            options::render(ctx, model, ui_state, sender);
        }

        if ui_state.about_show {
            about::render(ctx, sender);
        }

        if ui_state.preset_save_show {
            preset::render_save(ctx, ui_state, sender);
        }

        if ui_state.preset_load_show {
            preset::render_load(ctx, model, sender);
        }
    }
}
