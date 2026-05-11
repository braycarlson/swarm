use std::path::PathBuf;
use std::sync::Arc;

use crate::app::message::{App, Command, CommandBuilder};
use crate::app::state::{LoadStatus, Model, UiState};

use super::sync_to_active_session;

const IPC_PATH_COUNT_MAX: u32 = 100;

pub fn handle(model: &mut Model, ui: &mut UiState, message: App) -> Command {
    match message {
        App::Initialized => handle_app_initialized(model),
        App::RestoreLastSession => handle_restore_last_session(model),
        App::FileDialogOpened => Command::None,
        App::PathSelected(path) => handle_path_selected(model, ui, path),
        App::PathsReceivedFromIpc(paths) => handle_paths_from_ipc(model, ui, paths),
        App::AboutOpened => handle_about_opened(ui),
        App::AboutClosed => handle_about_closed(ui),
        App::Tick => Command::None,
        App::OpenInExplorer => handle_open_in_explorer(model),
    }
}

fn handle_app_initialized(model: &mut Model) -> Command {
    model.tree = Default::default();
    model.search = Default::default();

    let builder = CommandBuilder::new();
    builder.build()
}

fn handle_restore_last_session(model: &mut Model) -> Command {
    if let Some(closed) = model.sessions.last_closed_session.take() {
        let identifier = closed.identifier.clone();

        model.tree = closed.tree_state.clone();
        model.search = closed.search_state.clone();
        model.tree.load_status = LoadStatus::Loaded;

        model.sessions.sessions.insert(identifier.clone(), closed);
        model.sessions.active_identifier = Some(identifier);

        model.refresh_git_status();

        return Command::None;
    }

    let most_recent_identifier = model.sessions.sessions.values()
        .filter(|s| !s.tree_state.nodes.is_empty())
        .max_by_key(|s| s.last_modified)
        .map(|s| s.identifier.clone());

    if let Some(identifier) = most_recent_identifier {
        if let Some(session) = model.sessions.select_session(identifier) {
            model.tree = session.tree_state.clone();
            model.search = session.search_state.clone();
            model.tree.load_status = LoadStatus::Loaded;

            model.refresh_git_status();
        }
    }

    Command::None
}

fn handle_path_selected(model: &mut Model, ui: &mut UiState, path: PathBuf) -> Command {
    let active_session = model.sessions.active_session();

    let should_create_new = if let Some(session) = active_session {
        !session.tree_state.nodes.is_empty()
    } else {
        true
    };

    let mut command_builder = CommandBuilder::new();

    if should_create_new {
        sync_to_active_session(model);

        let session_name = "Session".to_string();
        model.sessions.create_session(session_name);

        model.tree = Default::default();
        model.search = Default::default();
    } else {
        let active_identifier = model.sessions.active_identifier.clone();

        if active_identifier.is_none() {
            return Command::None;
        }
    }

    ui.file_dialog_pending = true;

    model.tree.load_status = LoadStatus::Loading {
        message: format!("Loading {}", path.display()),
        progress: (0, 0),
    };

    model.git_service.refresh(&path);

    command_builder = command_builder.add(Command::LoadSession {
        path,
        options: Arc::clone(&model.options),
    });

    command_builder.build()
}

fn handle_paths_from_ipc(model: &mut Model, _ui: &mut UiState, paths: Vec<PathBuf>) -> Command {
    if paths.is_empty() {
        return Command::None;
    }

    sync_to_active_session(model);

    let session_name = "Session".to_string();
    model.sessions.create_session(session_name);

    model.tree = Default::default();
    model.search = Default::default();

    let mut command_builder = CommandBuilder::new();

    let path_count = paths.len();

    model.tree.load_status = LoadStatus::Loading {
        message: format!("Loading {} paths", path_count),
        progress: (0, path_count),
    };

    if let Some(first_path) = paths.first() {
        model.git_service.refresh(first_path);
    }

    for path in paths.into_iter().take(IPC_PATH_COUNT_MAX as usize) {
        command_builder = command_builder.add(Command::LoadSession {
            path,
            options: Arc::clone(&model.options),
        });
    }

    command_builder.build()
}

fn handle_about_opened(ui: &mut UiState) -> Command {
    ui.about_show = true;
    Command::None
}

fn handle_about_closed(ui: &mut UiState) -> Command {
    ui.about_show = false;
    Command::None
}

fn handle_open_in_explorer(model: &Model) -> Command {
    if model.tree.nodes.is_empty() {
        return Command::None;
    }

    let path = &model.tree.nodes[0].path;

    #[cfg(target_os = "windows")]
    {
        let _ = std::process::Command::new("explorer")
            .arg(path)
            .spawn();
    }

    #[cfg(target_os = "macos")]
    {
        let _ = std::process::Command::new("open")
            .arg(path)
            .spawn();
    }

    #[cfg(target_os = "linux")]
    {
        let _ = std::process::Command::new("xdg-open")
            .arg(path)
            .spawn();
    }

    Command::None
}
