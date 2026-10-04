use crate::app::message::{Command, Render};
use crate::app::state::{Model, UiState};

use super::common::prepare_selection;

pub fn handle(model: &mut Model, ui: &mut UiState, message: Render) -> Command {
    match message {
        Render::Failed(error) => {
            debug_assert!(ui.tree_generate_in_progress);

            ui.tree_generate_in_progress = false;
            ui.toast.error(format!("Tree rendering failed: {error}"));

            Command::None
        }
        Render::Generated(output) => {
            debug_assert!(ui.tree_generate_in_progress);

            ui.tree_generate_in_progress = false;
            ui.clipboard_pending = Some(output);
            ui.toast.success("Copied to clipboard");

            Command::None
        }
        Render::Requested => handle_requested(model, ui),
    }
}

fn handle_requested(model: &mut Model, ui: &mut UiState) -> Command {
    if ui.tree_generate_in_progress {
        return Command::None;
    }

    prepare_selection(model, ui);

    let nodes = model.selected_tree();

    if nodes.is_empty() {
        return Command::None;
    }

    ui.tree_generate_in_progress = true;

    Command::RenderTree {
        nodes,
        use_icons: model.options.use_icon,
    }
}
