use std::sync::Arc;

use crate::app::message::{Command, Render};
use crate::app::state::{Model, UiState};

pub fn handle(model: &mut Model, ui: &mut UiState, message: Render) -> Command {
    match message {
        Render::Requested => handle_render_requested(model, ui),
        Render::Started => handle_render_started(ui),
        Render::Generated(output) => handle_render_generated(model, ui, output),
        Render::Failed(error) => handle_render_failed(ui, error),
    }
}

fn handle_render_requested(model: &mut Model, ui: &mut UiState) -> Command {
    if ui.tree_generate_in_progress {
        return Command::None;
    }

    model.refresh_git_status();

    let filtered = model.tree.create_filtered_tree_with_git(&model.search, Some(&model.git));

    if filtered.is_empty() {
        return Command::None;
    }

    ui.tree_generate_in_progress = true;
    model.tree.output.clear();

    Command::RenderTree {
        nodes: filtered,
        options: Arc::clone(&model.options),
    }
}

fn handle_render_started(ui: &mut UiState) -> Command {
    ui.tree_generate_in_progress = true;
    Command::None
}

fn handle_render_generated(model: &mut Model, ui: &mut UiState, output: String) -> Command {
    model.tree.output = output;
    ui.tree_generate_in_progress = false;

    ui.toast.success("Copied to clipboard");

    Command::None
}

fn handle_render_failed(ui: &mut UiState, error: String) -> Command {
    ui.tree_generate_in_progress = false;
    eprintln!("Tree rendering failed: {}", error);

    ui.toast.error(format!("Tree rendering failed: {}", error));

    Command::None
}
