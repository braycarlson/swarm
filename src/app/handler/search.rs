use crate::app::message::{Cmd, Search};
use crate::app::state::Model;

use super::sync_to_active_session;

pub fn handle(model: &mut Model, msg: Search) -> Cmd {
    match msg {
        Search::QueryChanged(query) => handle_search_query_changed(model, query),
        Search::Activated => handle_search_activated(model),
        Search::Cleared => handle_search_cleared(model),
    }
}

fn handle_search_query_changed(model: &mut Model, query: String) -> Cmd {
    model.search.set_query(query);
    sync_to_active_session(model);

    Cmd::None
}

fn handle_search_activated(model: &mut Model) -> Cmd {
    model.search.activate();
    sync_to_active_session(model);

    Cmd::None
}

fn handle_search_cleared(model: &mut Model) -> Cmd {
    model.search.clear();
    sync_to_active_session(model);

    Cmd::None
}
