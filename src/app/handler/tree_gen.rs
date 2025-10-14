use std::sync::Arc;

use crate::app::message::{Cmd, TreeGen};
use crate::app::state::{Model, UiState};

pub fn handle(model: &mut Model, ui: &mut UiState, msg: TreeGen) -> Cmd {
    match msg {
        TreeGen::Requested => handle_generate_tree_requested(model, ui),
        TreeGen::Started => handle_tree_generation_started(ui),
        TreeGen::Generated(output) => handle_tree_generated(model, ui, output),
        TreeGen::Failed(error) => handle_tree_generation_failed(ui, error),
    }
}

fn handle_generate_tree_requested(model: &mut Model, ui: &mut UiState) -> Cmd {
    if ui.tree_gen_in_progress {
        return Cmd::None;
    }

    let filtered = model.tree.create_filtered_tree(&model.search);

    if filtered.is_empty() {
        return Cmd::None;
    }

    ui.tree_gen_in_progress = true;
    model.tree.output.clear();

    Cmd::GenerateTree {
        nodes: filtered,
        options: Arc::clone(&model.options),
    }
}

fn handle_tree_generation_started(ui: &mut UiState) -> Cmd {
    ui.tree_gen_in_progress = true;
    Cmd::None
}

fn handle_tree_generated(model: &mut Model, ui: &mut UiState, output: String) -> Cmd {
    model.tree.output = output;
    ui.tree_gen_in_progress = false;
    Cmd::None
}

fn handle_tree_generation_failed(ui: &mut UiState, error: String) -> Cmd {
    ui.tree_gen_in_progress = false;
    eprintln!("Tree generation failed: {}", error);
    Cmd::None
}
