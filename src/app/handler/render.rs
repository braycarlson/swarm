use std::sync::Arc;

use crate::app::message::{Cmd, Render};
use crate::app::state::{Model, UiState};

pub fn handle(model: &mut Model, ui: &mut UiState, msg: Render) -> Cmd {
    match msg {
        Render::Requested => handle_render_requested(model, ui),
        Render::Started => handle_render_started(ui),
        Render::Generated(output) => handle_render_generated(model, ui, output),
        Render::Failed(error) => handle_render_failed(ui, error),
    }
}

fn handle_render_requested(model: &mut Model, ui: &mut UiState) -> Cmd {
    if ui.tree_gen_in_progress {
        return Cmd::None;
    }

    model.refresh_git_status();

    let filtered = model.tree.create_filtered_tree_with_git(&model.search, Some(&model.git));

    if filtered.is_empty() {
        return Cmd::None;
    }

    ui.tree_gen_in_progress = true;
    model.tree.output.clear();

    Cmd::RenderTree {
        nodes: filtered,
        options: Arc::clone(&model.options),
    }
}

fn handle_render_started(ui: &mut UiState) -> Cmd {
    ui.tree_gen_in_progress = true;
    Cmd::None
}

fn handle_render_generated(model: &mut Model, ui: &mut UiState, output: String) -> Cmd {
    model.tree.output = output;
    ui.tree_gen_in_progress = false;

    ui.toast.success("Copied to clipboard");

    Cmd::None
}

fn handle_render_failed(ui: &mut UiState, error: String) -> Cmd {
    ui.tree_gen_in_progress = false;
    eprintln!("Tree rendering failed: {}", error);

    ui.toast.error(format!("Tree rendering failed: {}", error));

    Cmd::None
}
