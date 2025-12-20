use std::path::PathBuf;
use std::sync::Arc;

use crate::app::message::{App, Cmd, CmdBuilder};
use crate::app::state::{LoadStatus, Model, UiState};

use super::sync_to_active_session;

const MAX_IPC_PATHS: u32 = 100;

pub fn handle(model: &mut Model, ui: &mut UiState, msg: App) -> Cmd {
    match msg {
        App::Initialized => handle_app_initialized(model),
        App::FileDialogOpened => Cmd::None,
        App::PathSelected(path) => handle_path_selected(model, ui, path),
        App::PathsReceivedFromIpc(paths) => handle_paths_from_ipc(model, ui, paths),
        App::AboutOpened => handle_about_opened(ui),
        App::AboutClosed => handle_about_closed(ui),
        App::Tick => Cmd::None,
        App::OpenInExplorer => handle_open_in_explorer(model),
    }
}

fn handle_app_initialized(model: &mut Model) -> Cmd {
    model.tree = Default::default();
    model.search = Default::default();

    let builder = CmdBuilder::new();
    builder.build()
}

fn handle_path_selected(model: &mut Model, ui: &mut UiState, path: PathBuf) -> Cmd {
    let active_session = model.sessions.active_session();

    let should_create_new = if let Some(session) = active_session {
        !session.tree_state.nodes.is_empty()
    } else {
        true
    };

    let mut cmd_builder = CmdBuilder::new();

    if should_create_new {
        sync_to_active_session(model);

        let session_name = "Session".to_string();
        model.sessions.create_session(session_name);

        model.tree = Default::default();
        model.search = Default::default();
    } else {
        let active_id = model.sessions.active_id.clone();

        if active_id.is_none() {
            return Cmd::None;
        }
    }

    ui.file_dialog_pending = true;

    model.tree.load_status = LoadStatus::Loading {
        message: format!("Loading {}", path.display()),
        progress: (0, 0),
    };

    model.git.refresh(&path);

    cmd_builder = cmd_builder.add(Cmd::LoadSession {
        path,
        options: Arc::clone(&model.options),
    });

    cmd_builder.build()
}

fn handle_paths_from_ipc(model: &mut Model, _ui: &mut UiState, paths: Vec<PathBuf>) -> Cmd {
    if paths.is_empty() {
        return Cmd::None;
    }

    sync_to_active_session(model);

    let session_name = "Session".to_string();
    model.sessions.create_session(session_name);

    model.tree = Default::default();
    model.search = Default::default();

    let mut cmd_builder = CmdBuilder::new();

    let path_count = paths.len();

    model.tree.load_status = LoadStatus::Loading {
        message: format!("Loading {} paths", path_count),
        progress: (0, path_count),
    };

    if let Some(first_path) = paths.first() {
        model.git.refresh(first_path);
    }

    for path in paths.into_iter().take(MAX_IPC_PATHS as usize) {
        cmd_builder = cmd_builder.add(Cmd::LoadSession {
            path,
            options: Arc::clone(&model.options),
        });
    }

    cmd_builder.build()
}

fn handle_about_opened(ui: &mut UiState) -> Cmd {
    ui.show_about = true;
    Cmd::None
}

fn handle_about_closed(ui: &mut UiState) -> Cmd {
    ui.show_about = false;
    Cmd::None
}

fn handle_open_in_explorer(model: &Model) -> Cmd {
    if model.tree.nodes.is_empty() {
        return Cmd::None;
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

    Cmd::None
}
