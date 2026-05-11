use std::path::PathBuf;

use rustc_hash::FxHashSet;

use crate::app::message::{Command, Search};
use crate::app::state::{FilterStatus, Model, UiState};
use crate::services::worker::filter::{add_ancestor_directories, build_filter_entries};

use super::sync_to_active_session;

const SEARCH_DEBOUNCE_MS: u64 = 150;

pub fn handle(model: &mut Model, ui: &mut UiState, message: Search) -> Command {
    match message {
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

fn handle_search_query_changed(ui: &mut UiState, query: String) -> Command {
    ui.set_search_pending(query);
    Command::None
}

fn handle_search_activated(model: &mut Model) -> Command {
    model.search.activate();
    sync_to_active_session(model);
    Command::None
}

fn handle_search_cleared(model: &mut Model, ui: &mut UiState) -> Command {
    model.search.clear();
    model.clear_filter_cache();
    ui.search_pending = None;
    ui.search_debounce = None;
    ui.filter_status = FilterStatus::Idle;
    sync_to_active_session(model);
    Command::None
}

fn handle_debounce_tick(model: &mut Model, ui: &mut UiState) -> Command {
    if let Some(query) = ui.take_debounced_search(SEARCH_DEBOUNCE_MS) {
        model.search.set_query(query);
        model.clear_filter_cache();

        let parsed = model.search.parsed();

        if parsed.is_expensive() && !model.tree.nodes.is_empty() {
            ui.filter_status = FilterStatus::Filtering;

            let entries = build_filter_entries(&model.tree.nodes, parsed.as_ref());

            return Command::StartExpensiveFilter {
                entries,
                query: parsed.into_owned(),
                git: model.git_service.clone(),
            };
        }

        sync_to_active_session(model);
    }
    Command::None
}

fn handle_filter_started(ui: &mut UiState) -> Command {
    ui.filter_status = FilterStatus::Filtering;
    Command::None
}

fn handle_filter_progress(_ui: &mut UiState, _current: usize, _total: usize) -> Command {
    Command::None
}

fn handle_filter_complete(model: &mut Model, ui: &mut UiState, mut matching: FxHashSet<PathBuf>) -> Command {
    model.filtered_nodes = None;

    let parsed = model.search.parsed();
    add_ancestor_directories(&model.tree.nodes, &mut matching, parsed.as_ref());

    model.search.matching_paths = Some(matching);
    ui.filter_status = FilterStatus::Complete;
    sync_to_active_session(model);
    Command::None
}

fn handle_filter_cancelled(ui: &mut UiState) -> Command {
    ui.filter_status = FilterStatus::Idle;
    Command::None
}
