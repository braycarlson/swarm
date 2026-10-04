use std::path::PathBuf;

use crate::app::message::{App, Command};
use crate::app::state::{Model, SearchModel, TreeModel, UiState};
use crate::platform;

use super::common::{open_paths, switch_to};

pub fn handle(model: &mut Model, ui: &mut UiState, message: App) -> Command {
    match message {
        App::AboutClosed => {
            ui.about_show = false;

            Command::None
        }
        App::AboutOpened => {
            ui.about_show = true;

            Command::None
        }
        App::FileDialogClosed => {
            ui.file_dialog_open = false;

            Command::None
        }
        App::FileDialogOpened => handle_file_dialog_opened(ui),
        App::Initialized => handle_initialized(model, ui),
        App::OpenInExplorer => handle_open_in_explorer(model, ui),
        App::PathSelected(path) => {
            ui.file_dialog_open = false;

            open_paths(model, ui, vec![path])
        }
        App::PathsReceivedFromIpc(paths) => handle_paths_from_ipc(model, ui, paths),
        App::RestoreLastSession => handle_restore_last_session(model, ui),
    }
}

fn handle_file_dialog_opened(ui: &mut UiState) -> Command {
    if ui.file_dialog_open {
        return Command::None;
    }

    ui.file_dialog_open = true;

    Command::OpenFileDialog
}

fn handle_initialized(model: &mut Model, ui: &mut UiState) -> Command {
    model.tree = TreeModel::default();
    model.search = SearchModel::default();
    ui.sync_search_text(&model.search);

    debug_assert_eq!(model.tree.nodes.len(), 0);

    Command::None
}

fn handle_open_in_explorer(model: &Model, ui: &mut UiState) -> Command {
    let Some(node) = model.tree.nodes.first() else {
        return Command::None;
    };

    if let Err(error) = platform::path_open(&node.path) {
        ui.toast
            .error(format!("Failed to open {}: {error}", node.path.display()));
    }

    Command::None
}

fn handle_paths_from_ipc(model: &mut Model, ui: &mut UiState, paths: Vec<PathBuf>) -> Command {
    if paths.is_empty() {
        return Command::None;
    }

    open_paths(model, ui, paths)
}

fn handle_restore_last_session(model: &mut Model, ui: &mut UiState) -> Command {
    model.flush_session();

    if let Some(closed) = model.sessions.take_last_closed() {
        let identifier = closed.identifier.clone();

        model.sessions.add_session(closed);
        model.sessions.rebuild_order();

        return switch_to(model, ui, &identifier);
    }

    let Some(identifier) = model.sessions.most_recent_with_tree() else {
        return Command::None;
    };

    switch_to(model, ui, &identifier)
}
