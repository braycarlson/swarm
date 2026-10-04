use crate::app::message::{Command, Search};
use crate::app::state::{FilterStatus, Model, UiState};
use crate::services::worker::FilterComplete;
use crate::services::worker::filter::add_ancestor_directories;

use super::common::refresh_matches;

const SEARCH_DEBOUNCE_MS: u64 = 150;

pub fn handle(model: &mut Model, ui: &mut UiState, message: Search) -> Command {
    match message {
        Search::Cleared => handle_cleared(model, ui),
        Search::DebounceTick => handle_debounce_tick(model, ui),
        Search::FilterCompleted(complete) => handle_filter_completed(model, ui, *complete),
        Search::QueryEdited => {
            ui.start_search_debounce();

            Command::None
        }
    }
}

fn handle_cleared(model: &mut Model, ui: &mut UiState) -> Command {
    model.search.clear();
    model.mark_session_dirty();
    ui.filter_status = FilterStatus::Idle;
    ui.search_debounce = None;
    ui.search_text.clear();

    debug_assert!(!model.search.has_query());

    Command::CancelFilter
}

fn handle_debounce_tick(model: &mut Model, ui: &mut UiState) -> Command {
    if !ui.take_debounced_search(SEARCH_DEBOUNCE_MS) {
        return Command::None;
    }

    model.search.set_query(ui.search_text.clone());
    model.mark_session_dirty();

    refresh_matches(model, ui)
}

fn handle_filter_completed(
    model: &mut Model,
    ui: &mut UiState,
    complete: FilterComplete,
) -> Command {
    if model.sessions.active_identifier() != Some(complete.session.as_str()) {
        return Command::None;
    }

    if !model.search.has_query() {
        ui.filter_status = FilterStatus::Idle;

        return Command::None;
    }

    let mut matching = complete.matching;
    let parsed = model.search.parsed().into_owned();

    add_ancestor_directories(&model.tree.nodes, &mut matching, &parsed);

    model.search.set_matching_paths(Some(matching));
    model.mark_session_dirty();
    ui.filter_status = FilterStatus::Complete;

    Command::None
}
