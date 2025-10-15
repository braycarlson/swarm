use std::sync::Arc;

use crate::app::message::{Cmd, Copy};
use crate::app::state::{Model, UiState};

pub fn handle(model: &mut Model, ui: &mut UiState, msg: Copy) -> Cmd {
    match msg {
        Copy::Requested => handle_copy_requested(model, ui),
        Copy::Started => handle_copy_started(ui),
        Copy::Completed(output) => handle_copy_completed(model, ui, output),
        Copy::Failed(error) => handle_copy_failed(ui, error),
    }
}

fn handle_copy_requested(model: &mut Model, ui: &mut UiState) -> Cmd {
    if ui.copy_in_progress {
        return Cmd::None;
    }

    let paths = model.tree.gather_checked_paths(&model.search);

    if paths.is_empty() {
        return Cmd::None;
    }

    ui.copy_in_progress = true;
    model.tree.output.clear();

    Cmd::GatherFiles {
        paths,
        options: Arc::clone(&model.options),
    }
}

fn handle_copy_started(ui: &mut UiState) -> Cmd {
    ui.copy_in_progress = true;
    Cmd::None
}

fn handle_copy_completed(model: &mut Model, ui: &mut UiState, output: String) -> Cmd {
    model.tree.output = output;
    ui.copy_in_progress = false;

    ui.toast.success("Copied to clipboard");

    Cmd::None
}

fn handle_copy_failed(ui: &mut UiState, error: String) -> Cmd {
    ui.copy_in_progress = false;
    eprintln!("Copy failed: {}", error);

    Cmd::None
}
