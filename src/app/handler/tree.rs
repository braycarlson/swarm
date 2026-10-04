use alloc::sync::Arc;
use std::path::PathBuf;

use rustc_hash::FxHashSet;

use crate::app::message::{Command, ToggleScope, Tree};
use crate::app::state::{LoadStatus, Model, UiState};
use crate::model::query::ParsedQuery;
use crate::model::selection;
use crate::services::tree::loader::{self, LoadStop};
use crate::services::tree::traversal::{self, PropagateRequest};
use crate::services::worker::{
    BackgroundOutcome,
    PropagateJob,
    PropagateOutcome,
    SessionLoadResult,
    TreeRefreshRequest,
    TreeRefreshResult,
};

use super::common::refresh_matches;

pub fn handle(model: &mut Model, ui: &mut UiState, message: Tree) -> Command {
    match message {
        Tree::BackgroundLoadCompleted(outcome) => handle_background_completed(model, ui, *outcome),
        Tree::BulkSelectApplied => handle_bulk_select_applied(model, ui),
        Tree::BulkSelectToggled => {
            ui.bulk_select_show = !ui.bulk_select_show;

            if !ui.bulk_select_show {
                ui.bulk_select_text.clear();
            }

            Command::None
        }
        Tree::DeselectAll => handle_set_all(model, false),
        Tree::NodeExpanded { path } => handle_node_expanded(model, ui, &path),
        Tree::NodeToggled {
            checked,
            path,
            scope,
        } => handle_node_toggled(model, ui, path, checked, scope),
        Tree::PropagateCompleted(outcome) => handle_propagate_completed(model, ui, *outcome),
        Tree::Refreshed(result) => handle_refreshed(model, ui, *result),
        Tree::RefreshRequested => handle_refresh_requested(model),
        Tree::SelectAll => handle_set_all(model, true),
        Tree::SessionLoaded(result) => handle_session_loaded(model, ui, *result),
    }
}

fn depth_of(path: &[u32]) -> u32 {
    assert_ne!(path.len(), 0);

    u32::try_from(path.len() - 1).expect("a node path is bounded by the tree depth")
}

fn handle_background_completed(
    model: &mut Model,
    ui: &mut UiState,
    outcome: BackgroundOutcome,
) -> Command {
    let BackgroundOutcome {
        generation,
        nodes,
        session,
        summary,
    } = outcome;

    debug_assert!(generation > 0);

    if model.sessions.active_identifier() != Some(session.as_str()) {
        if let Some(stored) = model.sessions.get_mut(&session) {
            stored.tree_state.replace_nodes(nodes);
            stored.tree_state.background_loading = false;
        }

        return Command::None;
    }

    model.tree.replace_nodes(nodes);
    model.tree.background_loading = false;
    model.mark_session_dirty();

    if summary.stop == LoadStop::Bound {
        ui.toast.error(format!(
            "Stopped scanning after {} entries; deeper directories load when expanded",
            summary.progress.nodes,
        ));
    }

    refresh_matches(model, ui)
}

fn handle_bulk_select_applied(model: &mut Model, ui: &mut UiState) -> Command {
    let suffixes: Vec<String> = ui
        .bulk_select_text
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(|line| line.replace('\\', "/"))
        .collect();

    if suffixes.is_empty() {
        return Command::None;
    }

    selection::set_all_checked(&mut model.tree.nodes, false);

    let count = selection::select_paths_by_suffix(&mut model.tree.nodes, &suffixes);

    model.tree.update_files_count();
    model.mark_session_dirty();
    ui.toast.success(format!("{count} files selected"));
    ui.bulk_select_show = false;
    ui.bulk_select_text.clear();

    Command::None
}

fn handle_node_expanded(model: &mut Model, ui: &mut UiState, path: &[u32]) -> Command {
    let depth = depth_of(path);

    let Some(node) = selection::find_node_mut(&mut model.tree.nodes, path) else {
        return Command::None;
    };

    if node.loaded {
        return Command::None;
    }

    if !node.is_directory() {
        return Command::None;
    }

    if let Err(error) = loader::load_children(node, depth, &model.filter) {
        eprintln!("Failed to load {}: {error}", node.path.display());
    }

    debug_assert!(node.loaded);

    model.tree.update_files_count();
    model.mark_session_dirty();

    refresh_matches(model, ui)
}

fn handle_node_toggled(
    model: &mut Model,
    ui: &mut UiState,
    path: Vec<u32>,
    checked: bool,
    scope: ToggleScope,
) -> Command {
    if scope == ToggleScope::Node {
        toggle_node(model, &path, checked);

        return Command::None;
    }

    let Some(target) = selection::find_node(&model.tree.nodes, &path) else {
        return Command::None;
    };

    if !target.is_directory() {
        toggle_node(model, &path, checked);

        return Command::None;
    }

    if target.has_unloaded_directory() {
        return start_propagate(model, path, checked);
    }

    let parsed = model
        .search
        .has_query()
        .then(|| model.search.parsed().into_owned());
    let failed_count = propagate_in_place(model, &path, checked, parsed.as_ref());

    if failed_count > 0 {
        ui.toast
            .error(format!("{failed_count} directories could not be read"));
    }

    Command::None
}

fn handle_propagate_completed(
    model: &mut Model,
    ui: &mut UiState,
    outcome: PropagateOutcome,
) -> Command {
    let PropagateOutcome {
        failed_count,
        nodes,
        session,
    } = outcome;

    if failed_count > 0 {
        ui.toast
            .error(format!("{failed_count} directories could not be read"));
    }

    if model.sessions.active_identifier() != Some(session.as_str()) {
        if let Some(stored) = model.sessions.get_mut(&session) {
            stored.tree_state.nodes = nodes;
            stored.tree_state.load_status = LoadStatus::Loaded;
            stored.tree_state.update_files_count();
        }

        return Command::None;
    }

    model.tree.nodes = nodes;
    model.tree.load_status = LoadStatus::Loaded;
    model.tree.update_files_count();
    model.mark_session_dirty();

    refresh_matches(model, ui)
}

fn handle_refresh_requested(model: &mut Model) -> Command {
    if model.tree.is_loading() {
        return Command::None;
    }

    let Some(session) = model.sessions.active_identifier().map(str::to_owned) else {
        return Command::None;
    };

    if model.tree.nodes.is_empty() {
        return Command::None;
    }

    model.tree.states = Some(model.tree.collect_checked_paths());
    model.tree.load_status = LoadStatus::Loading("Refreshing...".to_owned());

    let nodes = core::mem::take(&mut model.tree.nodes);

    Command::batch(vec![
        Command::StopBackground,
        Command::RefreshTree(Box::new(TreeRefreshRequest {
            filter: Arc::clone(&model.filter),
            nodes,
            session,
        })),
    ])
}

fn handle_refreshed(model: &mut Model, ui: &mut UiState, result: TreeRefreshResult) -> Command {
    let TreeRefreshResult {
        failed,
        nodes,
        session,
    } = result;

    if let Some(first) = failed.first() {
        ui.toast.error(format!("Failed to refresh {first}"));
    }

    if model.sessions.active_identifier() != Some(session.as_str()) {
        if let Some(stored) = model.sessions.get_mut(&session) {
            stored.tree_state.nodes = nodes;
            stored.tree_state.load_status = LoadStatus::Loaded;
        }

        return Command::None;
    }

    let restored: FxHashSet<PathBuf> = model.tree.states.take().unwrap_or_default();

    model.tree.nodes = nodes;
    model.tree.restore_checked_paths(&restored);
    model.tree.load_status = LoadStatus::Loaded;
    model.tree.update_files_count();
    model.refresh_git_status();
    model.mark_session_dirty();

    let search = refresh_matches(model, ui);
    let background = model.background_start();

    Command::batch(vec![search, background])
}

fn handle_session_loaded(
    model: &mut Model,
    ui: &mut UiState,
    result: SessionLoadResult,
) -> Command {
    let SessionLoadResult {
        missing,
        nodes,
        session,
    } = result;

    for path in &missing {
        ui.toast
            .error(format!("{} no longer exists", path.display()));
    }

    if model.sessions.active_identifier() != Some(session.as_str()) {
        if let Some(stored) = model.sessions.get_mut(&session) {
            stored.tree_state.nodes = nodes;
            stored.tree_state.load_status = LoadStatus::Loaded;
        }

        return Command::None;
    }

    if nodes.is_empty() {
        model.tree.load_status = LoadStatus::Failed;

        return Command::None;
    }

    model.tree.nodes = nodes;
    model.tree.load_status = LoadStatus::Loaded;
    model.tree.update_files_count();
    model.refresh_git_status();
    model.mark_session_dirty();

    let search = refresh_matches(model, ui);
    let background = model.background_start();

    Command::batch(vec![search, background])
}

fn handle_set_all(model: &mut Model, checked: bool) -> Command {
    selection::set_all_checked(&mut model.tree.nodes, checked);
    model.tree.update_files_count();
    model.mark_session_dirty();

    Command::None
}

fn propagate_in_place(
    model: &mut Model,
    path: &[u32],
    checked: bool,
    parsed: Option<&ParsedQuery>,
) -> u32 {
    let depth = depth_of(path);
    let matches = model.search.matching_paths();

    let Some(node) = selection::find_node_mut(&mut model.tree.nodes, path) else {
        return 0;
    };

    let request = PropagateRequest {
        checked,
        depth,
        filter: &model.filter,
        git: Some(&model.git_service),
        matches,
        query: parsed,
    };

    let failed_count = traversal::propagate_checked(node, &request);

    selection::update_ancestors(&mut model.tree.nodes, path, checked);
    model.tree.update_files_count();
    model.mark_session_dirty();

    failed_count
}

fn start_propagate(model: &mut Model, path: Vec<u32>, checked: bool) -> Command {
    let Some(session) = model.sessions.active_identifier().map(str::to_owned) else {
        return Command::None;
    };

    model.tree.load_status = LoadStatus::Loading("Loading directories...".to_owned());

    let query = model
        .search
        .has_query()
        .then(|| model.search.parsed().into_owned());

    let nodes = core::mem::take(&mut model.tree.nodes);

    Command::PropagateChecked(Box::new(PropagateJob {
        checked,
        filter: Arc::clone(&model.filter),
        git: model.git_service.clone(),
        nodes,
        path,
        query,
        session,
    }))
}

fn toggle_node(model: &mut Model, path: &[u32], checked: bool) {
    if let Some(node) = selection::find_node_mut(&mut model.tree.nodes, path) {
        node.checked = checked;
    }

    selection::update_ancestors(&mut model.tree.nodes, path, checked);
    model.tree.update_files_count();
    model.mark_session_dirty();
}
