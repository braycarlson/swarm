use std::path::PathBuf;

use crate::app::message::Command;
use crate::app::state::{FilterStatus, LoadStatus, Model, SearchModel, TreeModel, UiState};
use crate::services::tree::traversal;
use crate::services::worker::SessionLoadRequest;

pub const SESSION_NAME_DEFAULT: &str = "Session";

pub fn finish_session_switch(model: &mut Model, ui: &mut UiState) -> Command {
    model.tree.load_status = if model.tree.nodes.is_empty() {
        LoadStatus::NotStarted
    } else {
        LoadStatus::Loaded
    };

    model.refresh_git_status();
    model.search.ensure_parsed();
    model.tree.update_files_count();
    ui.sync_search_text(&model.search);
    ui.filter_status = FilterStatus::Idle;

    let search = refresh_matches(model, ui);

    let needs_loading = model
        .tree
        .nodes
        .iter()
        .any(|node| node.has_unloaded_directory());

    let background = if needs_loading {
        model.background_start()
    } else {
        Command::StopBackground
    };

    Command::batch(vec![background, search])
}

pub fn open_paths(model: &mut Model, ui: &mut UiState, paths: Vec<PathBuf>) -> Command {
    assert_ne!(paths.len(), 0);

    let reusable = model.sessions.active_identifier().is_some() && model.tree.nodes.is_empty();

    if !reusable {
        model.flush_session();

        let created = model
            .sessions
            .create_session(SESSION_NAME_DEFAULT.to_owned());

        debug_assert_eq!(model.sessions.active_identifier(), Some(created.as_str()));
    }

    let message = match paths.as_slice() {
        [single] => format!("Loading {}", single.display()),
        _ => format!("Loading {} paths", paths.len()),
    };

    model.tree = TreeModel::default();
    model.tree.load_status = LoadStatus::Loading(message);
    model.search = SearchModel::default();
    model.git_service.refresh(&paths[0]);
    ui.sync_search_text(&model.search);
    ui.filter_status = FilterStatus::Idle;

    let session = model
        .sessions
        .active_identifier()
        .expect("a session is active once one was created")
        .to_owned();

    Command::batch(vec![
        Command::StopBackground,
        Command::CancelFilter,
        Command::LoadSession(SessionLoadRequest { paths, session }),
    ])
}

pub fn prepare_selection(model: &mut Model, ui: &mut UiState) {
    let failed_count = traversal::expand_checked(&mut model.tree.nodes, &model.filter);

    if failed_count > 0 {
        ui.toast
            .error(format!("{failed_count} selected directories could not be read"));
    }

    model.tree.update_files_count();
    model.refresh_git_status();
    model.mark_session_dirty();
}

pub fn refresh_matches(model: &mut Model, ui: &mut UiState) -> Command {
    let command = model.refresh_search_matches();

    if matches!(command, Command::StartFilter(_)) {
        ui.filter_status = FilterStatus::Filtering;

        return command;
    }

    if ui.filter_status == FilterStatus::Filtering {
        ui.filter_status = FilterStatus::Idle;
    }

    Command::batch(vec![Command::CancelFilter, command])
}

pub fn switch_to(model: &mut Model, ui: &mut UiState, identifier: &str) -> Command {
    let Some(session) = model.sessions.select_session(identifier) else {
        return Command::None;
    };

    let tree = session.tree_state.clone();
    let search = session.search_state.clone();

    model.tree = tree;
    model.search = search;

    debug_assert_eq!(model.sessions.active_identifier(), Some(identifier));

    finish_session_switch(model, ui)
}
