use eframe::egui;

use crate::app::message::MessageSender;
use crate::app::state::{Model, UiState};

use super::panel::{bottom, central, menu};
use super::window::{about, options, preset};

pub fn render(ui: &mut egui::Ui, model: &Model, ui_state: &mut UiState, sender: &MessageSender) {
    menu::render(ui, model, ui_state, sender);
    bottom::render(ui, model, ui_state, sender);
    central::render(ui, model, ui_state, sender);

    let context = ui.ctx();

    if ui_state.options_show {
        options::render(context, model, ui_state, sender);
    }

    if ui_state.about_show {
        about::render(context, sender);
    }

    if ui_state.preset_save_show {
        preset::render_save(context, ui_state, sender);
    }

    if ui_state.preset_load_show {
        preset::render_load(context, ui_state, sender);
    }
}
