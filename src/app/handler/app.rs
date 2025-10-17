use std::path::PathBuf;
use std::sync::Arc;

use crate::app::message::{App, Cmd, CmdBuilder};
use crate::app::state::{IndexStatus, LoadStatus, Model, UiState};

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
    }
}

fn handle_app_initialized(model: &mut Model) -> Cmd {
    let session_name = format!("Session");
    let session_id = model.sessions.create_session(session_name);

    model.tree = Default::default();
    model.search = Default::default();

    model.index.status = IndexStatus::Idle;
    model.index.statistics = None;
    model.index.extensions.clear();

    let mut builder = CmdBuilder::new();
    builder = builder.add(Cmd::SwitchIndexSession(session_id));

    if model.options.auto_index_on_startup {
        model.index.status = IndexStatus::Running { paused: false };

        builder = builder.add(Cmd::StartIndexing {
            paths: vec![],
            options: Arc::clone(&model.options),
        });
    }

    builder.build()
}

fn handle_path_selected(model: &mut Model, ui: &mut UiState, path: PathBuf) -> Cmd {
    let active_session = model.sessions.active_session();

    let should_create_new = if active_session.is_some() {
        let session = active_session.unwrap();
        !session.tree_state.nodes.is_empty()
    } else {
        true
    };

    let mut cmd_builder = CmdBuilder::new();

    let session_id = if should_create_new {
        sync_to_active_session(model);

        let session_name = format!("Session");
        let session_id = model.sessions.create_session(session_name);

        model.tree = Default::default();
        model.search = Default::default();

        session_id
    } else {
        let active_id = model.sessions.active_id.clone();

        if active_id.is_none() {
            return Cmd::None;
        }

        active_id.unwrap()
    };

    cmd_builder = cmd_builder.add(Cmd::SwitchIndexSession(session_id));

    ui.file_dialog_pending = true;

    model.tree.load_status = LoadStatus::Loading {
        message: format!("Loading {}", path.display()),
        progress: (0, 0),
    };

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

    let session_name = format!("Session");
    let session_id = model.sessions.create_session(session_name);

    model.tree = Default::default();
    model.search = Default::default();

    let mut cmd_builder = CmdBuilder::new();
    cmd_builder = cmd_builder.add(Cmd::SwitchIndexSession(session_id));

    let path_count = paths.len();

    model.tree.load_status = LoadStatus::Loading {
        message: format!("Loading {} paths", path_count),
        progress: (0, path_count),
    };

    let mut processed_count: u32 = 0;

    for path in paths {
        if processed_count >= MAX_IPC_PATHS {
            break;
        }

        processed_count = processed_count + 1;

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
