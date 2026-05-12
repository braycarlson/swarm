use std::sync::Arc;

use crate::app::message::{Command, Copy};
use crate::app::state::{Model, UiState};

pub fn handle(model: &mut Model, ui: &mut UiState, message: Copy) -> Command {
    match message {
        Copy::Requested => handle_copy_requested(model, ui),
        Copy::Started => handle_copy_started(ui),
        Copy::Completed(output) => handle_copy_completed(model, ui, output),
        Copy::Failed(error) => handle_copy_failed(ui, error),
    }
}

fn handle_copy_requested(model: &mut Model, ui: &mut UiState) -> Command {
    if ui.copy_in_progress {
        return Command::None;
    }

    for node in &mut model.tree.nodes {
        node.expand_all_checked(&model.options);
    }

    model.refresh_git_status();

    let paths = model.tree.gather_checked_paths_with_git(&model.search, Some(&model.git_service));

    if paths.is_empty() {
        return Command::None;
    }

    ui.copy_in_progress = true;
    model.tree.output.clear();

    let query = model.search.parsed().into_owned();

    Command::GatherFiles {
        paths,
        options: Arc::clone(&model.options),
        git: model.git_service.clone(),
        query,
    }
}

fn handle_copy_started(ui: &mut UiState) -> Command {
    ui.copy_in_progress = true;
    Command::None
}

fn handle_copy_completed(model: &mut Model, ui: &mut UiState, message: String) -> Command {
    model.tree.output = String::new();
    ui.copy_in_progress = false;

    ui.toast.success(message);

    Command::None
}

fn handle_copy_failed(ui: &mut UiState, error: String) -> Command {
    ui.copy_in_progress = false;
    eprintln!("Copy failed: {}", error);

    ui.toast.error(format!("Copy failed: {}", error));

    Command::None
}
