use std::collections::HashSet;
use std::path::PathBuf;

use crate::app::message::{Cmd, Search};
use crate::app::state::{FilterStatus, Model, UiState};

use super::sync_to_active_session;

const SEARCH_DEBOUNCE_MS: u64 = 150;

pub fn handle(model: &mut Model, ui: &mut UiState, msg: Search) -> Cmd {
    match msg {
        Search::QueryChanged(query) => handle_search_query_changed(ui, query),
        Search::Activated => handle_search_activated(model),
        Search::Cleared => handle_search_cleared(model, ui),
        Search::DebounceTick => handle_debounce_tick(model, ui),
        Search::FilterStarted => handle_filter_started(ui),
        Search::FilterProgress(current, total) => handle_filter_progress(ui, current, total),
        Search::FilterComplete(matching) => handle_filter_complete(model, ui, matching),
        Search::FilterCancelled => handle_filter_cancelled(ui),
    }
}

fn handle_search_query_changed(ui: &mut UiState, query: String) -> Cmd {
    ui.set_search_pending(query);
    Cmd::None
}

fn handle_search_activated(model: &mut Model) -> Cmd {
    model.search.activate();
    sync_to_active_session(model);
    Cmd::None
}

fn handle_search_cleared(model: &mut Model, ui: &mut UiState) -> Cmd {
    model.search.clear();
    model.clear_filter_cache();
    ui.search_pending = None;
    ui.search_debounce = None;
    ui.filter_status = FilterStatus::Idle;
    sync_to_active_session(model);
    Cmd::None
}

fn handle_debounce_tick(model: &mut Model, ui: &mut UiState) -> Cmd {
    if let Some(query) = ui.take_debounced_search(SEARCH_DEBOUNCE_MS) {
        model.search.set_query(query);
        model.clear_filter_cache();

        let parsed = model.search.parsed();

        if parsed.is_expensive() && !model.tree.nodes.is_empty() {
            ui.filter_status = FilterStatus::Filtering;

            return Cmd::StartExpensiveFilter {
                nodes: model.tree.nodes.clone(),
                query: parsed,
                git: model.git.clone(),
            };
        }

        sync_to_active_session(model);
    }
    Cmd::None
}

fn handle_filter_started(ui: &mut UiState) -> Cmd {
    ui.filter_status = FilterStatus::Filtering;
    Cmd::None
}

fn handle_filter_progress(_ui: &mut UiState, _current: usize, _total: usize) -> Cmd {
    Cmd::None
}

fn handle_filter_complete(model: &mut Model, ui: &mut UiState, matching: HashSet<PathBuf>) -> Cmd {
    model.filtered_nodes = None;
    model.search.matching_paths = Some(matching);
    ui.filter_status = FilterStatus::Complete;
    sync_to_active_session(model);
    Cmd::None
}

fn handle_filter_cancelled(ui: &mut UiState) -> Cmd {
    ui.filter_status = FilterStatus::Idle;
    Cmd::None
}
