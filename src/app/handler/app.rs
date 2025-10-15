use std::path::PathBuf;
use std::sync::Arc;

use crate::app::message::{App, Cmd, CmdBuilder};
use crate::app::state::{IndexStatus, LoadStatus, Model, UiState};

use super::sync_to_active_session;

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

    let mut builder = CmdBuilder::new()
        .add(Cmd::SwitchIndexSession(session_id));

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
    let should_create_new = if let Some(session) = model.sessions.active_session() {
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
        model.sessions.active_id.clone().unwrap_or_default()
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

    let mut cmd_builder = CmdBuilder::new()
        .add(Cmd::SwitchIndexSession(session_id));

    model.tree.load_status = LoadStatus::Loading {
        message: format!("Loading {} paths", paths.len()),
        progress: (0, paths.len()),
    };

    for path in paths {
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
