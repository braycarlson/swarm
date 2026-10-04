use crate::app::message::{Command, Session};
use crate::app::state::{FilterStatus, LoadStatus, Model, SearchModel, TreeModel, UiState};

use super::common::switch_to;

pub fn handle(model: &mut Model, ui: &mut UiState, message: Session) -> Command {
    match message {
        Session::Created(name) => handle_created(model, ui, name),
        Session::Deleted(identifier) => handle_deleted(model, ui, identifier),
        Session::EditCancelled => {
            ui.cancel_session_edit();

            Command::None
        }
        Session::EditStarted(identifier) => {
            if let Some(session) = model.sessions.get(&identifier) {
                ui.start_session_edit(session);
            }

            Command::None
        }
        Session::Renamed { identifier, name } => {
            model.sessions.rename_session(&identifier, name);
            ui.cancel_session_edit();

            Command::None
        }
        Session::Selected(identifier) => handle_selected(model, ui, &identifier),
    }
}

fn clear_workspace(model: &mut Model, ui: &mut UiState) -> Command {
    model.tree = TreeModel::default();
    model.search = SearchModel::default();
    ui.filter_status = FilterStatus::Idle;
    ui.sync_search_text(&model.search);

    debug_assert_eq!(model.tree.load_status, LoadStatus::NotStarted);

    Command::batch(vec![Command::StopBackground, Command::CancelFilter])
}

fn handle_created(model: &mut Model, ui: &mut UiState, name: String) -> Command {
    model.flush_session();

    let created = model.sessions.create_session(name);

    debug_assert_eq!(model.sessions.active_identifier(), Some(created.as_str()));

    clear_workspace(model, ui)
}

fn handle_deleted(model: &mut Model, ui: &mut UiState, identifier: String) -> Command {
    let was_active = model.sessions.active_identifier() == Some(identifier.as_str());

    if was_active {
        model.flush_session();
    }

    let next = model.sessions.delete_session(&identifier);

    let switch = match (was_active, next) {
        (true, Some(identifier_next)) => switch_to(model, ui, &identifier_next),
        (true, None) => clear_workspace(model, ui),
        (false, _) => Command::None,
    };

    Command::batch(vec![Command::DeleteSessionData(identifier), switch])
}

fn handle_selected(model: &mut Model, ui: &mut UiState, identifier: &str) -> Command {
    if model.sessions.active_identifier() == Some(identifier) {
        return Command::None;
    }

    model.flush_session();

    switch_to(model, ui, identifier)
}
