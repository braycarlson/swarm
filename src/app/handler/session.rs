use crate::app::message::{Command, CommandBuilder, Session};
use crate::app::state::{Model, UiState};

use super::sync_to_active_session;

pub fn handle(model: &mut Model, ui: &mut UiState, message: Session) -> Command {
    match message {
        Session::Created(name) => handle_session_created(model, name),
        Session::Selected(identifier) => handle_session_selected(model, ui, identifier),
        Session::Deleted(identifier) => handle_session_deleted(model, identifier),
        Session::NameEdited(name) => handle_session_name_edited(ui, name),
        Session::Renamed { identifier, name } => handle_session_renamed(model, ui, identifier, name),
        Session::EditStarted(identifier) => handle_session_edit_started(model, ui, identifier),
        Session::EditCancelled => handle_session_edit_cancelled(ui),
    }
}

fn handle_session_created(model: &mut Model, name: String) -> Command {
    sync_to_active_session(model);

    model.sessions.create_session(name);
    model.tree = Default::default();
    model.search = Default::default();

    let builder = CommandBuilder::new();
    builder.build()
}

fn handle_session_selected(model: &mut Model, _ui: &mut UiState, identifier: String) -> Command {
    if model.sessions.active_identifier.as_ref() == Some(&identifier) {
        return Command::None;
    }

    sync_to_active_session(model);

    if let Some(session) = model.sessions.select_session(identifier.clone()) {
        model.tree = session.tree_state.clone();
        model.search = session.search_state.clone();

        model.refresh_git_status();

        let builder = CommandBuilder::new();
        builder.build()
    } else {
        Command::None
    }
}

fn handle_session_deleted(model: &mut Model, identifier: String) -> Command {
    if model.sessions.active_identifier.as_deref() == Some(&identifier) {
        sync_to_active_session(model);
    }

    if let Some(new_active_identifier) = model.sessions.delete_session(&identifier) {
        if let Some(session) = model.sessions.sessions.get(&new_active_identifier) {
            model.tree = session.tree_state.clone();
            model.search = session.search_state.clone();

            model.refresh_git_status();

            let builder = CommandBuilder::new()
                .add(Command::DeleteSessionData(identifier));

            return builder.build();
        }
    } else {
        model.tree = Default::default();
        model.search = Default::default();
    }

    Command::DeleteSessionData(identifier)
}

fn handle_session_renamed(model: &mut Model, ui: &mut UiState, identifier: String, name: String) -> Command {
    model.sessions.rename_session(&identifier, name);
    ui.cancel_session_edit();

    Command::None
}

fn handle_session_name_edited(ui: &mut UiState, name: String) -> Command {
    ui.editing_session_name = name;
    Command::None
}

fn handle_session_edit_started(model: &mut Model, ui: &mut UiState, identifier: String) -> Command {
    if let Some(session) = model.sessions.sessions.get(&identifier) {
        ui.start_session_edit(identifier, session.name.clone());
    }

    Command::None
}

fn handle_session_edit_cancelled(ui: &mut UiState) -> Command {
    ui.cancel_session_edit();
    Command::None
}
