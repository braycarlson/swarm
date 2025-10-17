use std::collections::HashSet;
use std::sync::Arc;

use crate::app::message::{Cmd, CmdBuilder, Session};
use crate::app::state::{IndexStatus, Model, UiState};
use crate::services::filesystem::IndexStatistics;

use super::sync_to_active_session;

pub fn handle(model: &mut Model, ui: &mut UiState, msg: Session) -> Cmd {
    match msg {
        Session::Created(name) => handle_session_created(model, name),
        Session::Selected(id) => handle_session_selected(model, ui, id),
        Session::IndexDataLoaded { statistics, extensions } => { handle_session_index_data_loaded(model, statistics, extensions) }
        Session::Deleted(id) => handle_session_deleted(model, id),
        Session::NameEdited(name) => handle_session_name_edited(ui, name),
        Session::Renamed { id, name } => handle_session_renamed(model, ui, id, name),
        Session::EditStarted(id) => handle_session_edit_started(model, ui, id),
        Session::EditCancelled => handle_session_edit_cancelled(ui),
    }
}

fn handle_session_created(model: &mut Model, name: String) -> Cmd {
    sync_to_active_session(model);

    let session_id = model.sessions.create_session(name);
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

fn handle_session_selected(model: &mut Model, _ui: &mut UiState, id: String) -> Cmd {
    if model.sessions.active_id.as_ref() == Some(&id) {
        return Cmd::None;
    }

    sync_to_active_session(model);

    if let Some(session) = model.sessions.select_session(id.clone()) {
        model.tree = session.tree_state.clone();
        model.search = session.search_state.clone();

        if !matches!(model.index.status, IndexStatus::Running { .. }) {
            if session.has_been_indexed {
                model.index.status = IndexStatus::Completed;
            } else {
                model.index.status = IndexStatus::Idle;
            }
        }

        let needs_indexing = model.options.auto_index_on_startup
            && !session.has_been_indexed
            && !model.tree.nodes.is_empty();

        let mut builder = CmdBuilder::new()
            .add(Cmd::SwitchIndexSession(id.clone()))
            .add(Cmd::LoadSessionIndexData(id));

        if needs_indexing {
            model.index.status = IndexStatus::Running { paused: false };
            builder = builder.add(Cmd::StartIndexing {
                paths: model.tree.nodes.iter().map(|n| n.path.clone()).collect(),
                options: Arc::clone(&model.options),
            });
        }

        builder.build()
    } else {
        Cmd::None
    }
}

fn handle_session_deleted(model: &mut Model, id: String) -> Cmd {
    if let Some(new_active_id) = model.sessions.delete_session(&id) {
        if let Some(session) = model.sessions.sessions.get(&new_active_id) {
            model.tree = session.tree_state.clone();
            model.search = session.search_state.clone();

            if !matches!(model.index.status, IndexStatus::Running { .. }) {
                if session.has_been_indexed {
                    model.index.status = IndexStatus::Completed;
                } else {
                    model.index.status = IndexStatus::Idle;
                }
            }

            let needs_indexing = model.options.auto_index_on_startup
                && !session.has_been_indexed
                && !model.tree.nodes.is_empty();

            let mut builder = CmdBuilder::new()
                .add(Cmd::DeleteSessionData(id))
                .add(Cmd::SwitchIndexSession(new_active_id.clone()))
                .add(Cmd::LoadSessionIndexData(new_active_id));

            if needs_indexing {
                model.index.status = IndexStatus::Running { paused: false };

                builder = builder.add(Cmd::StartIndexing {
                    paths: model.tree.nodes.iter().map(|n| n.path.clone()).collect(),
                    options: Arc::clone(&model.options),
                });
            }

            return builder.build();
        }
    } else {
        model.tree = Default::default();
        model.search = Default::default();
        model.index.status = IndexStatus::Idle;
        model.index.statistics = None;
        model.index.extensions.clear();
    }

    Cmd::DeleteSessionData(id)
}

fn handle_session_renamed(model: &mut Model, ui: &mut UiState, id: String, name: String) -> Cmd {
    model.sessions.rename_session(&id, name);
    ui.cancel_session_edit();

    Cmd::None
}

fn handle_session_name_edited(ui: &mut UiState, name: String) -> Cmd {
    ui.edit_name = name;
    Cmd::None
}

fn handle_session_edit_started(model: &mut Model, ui: &mut UiState, id: String) -> Cmd {
    if let Some(session) = model.sessions.sessions.get(&id) {
        ui.start_session_edit(id, session.name.clone());
    }

    Cmd::None
}

fn handle_session_edit_cancelled(ui: &mut UiState) -> Cmd {
    ui.cancel_session_edit();
    Cmd::None
}

fn handle_session_index_data_loaded(
    model: &mut Model,
    statistics: Option<IndexStatistics>,
    extensions: HashSet<String>,
) -> Cmd {
    let should_update = if let Some(active_id) = &model.sessions.active_id {
        if let Some(session) = model.sessions.sessions.get(active_id) {
            if session.has_been_indexed && statistics.is_none() {
                if model.options.auto_index_on_startup && !model.tree.nodes.is_empty() {
                    model.index.status = IndexStatus::Running { paused: false };

                    return Cmd::StartIndexing {
                        paths: model.tree.nodes.iter().map(|n| n.path.clone()).collect(),
                        options: Arc::clone(&model.options),
                    };
                } else {
                    return Cmd::None;
                }
            }

            true
        } else {
            false
        }
    } else {
        false
    };

    if should_update {
        model.index.statistics = statistics;
        model.index.extensions = extensions;
    }

    Cmd::None
}
