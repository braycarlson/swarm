use std::sync::Arc;

use crate::app::message::{Cmd, CmdBuilder, Tree};
use crate::app::state::{LoadStatus, Model, UiState};
use crate::model::node::FileNode;

use super::{load_node_children, sync_to_active_session, toggle_node};

pub fn handle(model: &mut Model, ui: &mut UiState, msg: Tree) -> Cmd {
    match msg {
        Tree::RefreshRequested => handle_tree_refresh_requested(model, ui),
        Tree::NodeToggled { path, checked, propagate } => { handle_tree_node_toggled(model, path, checked, propagate) }
        Tree::NodeExpanded { path } => handle_tree_node_expanded(model, path),
        Tree::Loaded(nodes) => handle_tree_loaded(model, ui, nodes),
        Tree::LoadProgress { current, processed, total } => { handle_tree_load_progress(model, current, processed, total) }
        Tree::LoadFailed(error) => handle_tree_load_failed(model, error),
    }
}

fn handle_tree_refresh_requested(model: &mut Model, _ui: &mut UiState) -> Cmd {
    if matches!(model.tree.load_status, LoadStatus::Loading { .. }) {
        return Cmd::None;
    }

    model.tree.states = Some(model.tree.collect_checkbox_states());

    model.tree.load_status = LoadStatus::Loading {
        message: "Starting refresh...".to_string(),
        progress: (0, 0),
    };

    let mut builder = CmdBuilder::new()
        .add(Cmd::RefreshTree {
            nodes: model.tree.nodes.clone(),
            options: Arc::clone(&model.options),
        });

    if let Some(session_id) = &model.sessions.active_id {
        builder = builder.add(Cmd::SwitchIndexSession(session_id.clone()));
    }

    builder.build()
}

fn handle_tree_node_toggled(model: &mut Model, path: Vec<usize>, checked: bool, propagate: bool) -> Cmd {
    toggle_node(&mut model.tree.nodes, &path, checked, propagate);
    sync_to_active_session(model);
    Cmd::None
}

fn handle_tree_node_expanded(model: &mut Model, path: Vec<usize>) -> Cmd {
    load_node_children(&mut model.tree.nodes, &path, &model.options);
    sync_to_active_session(model);
    Cmd::None
}

fn handle_tree_loaded(model: &mut Model, _ui: &mut UiState, nodes: Vec<FileNode>) -> Cmd {
    let current_states = model.tree.collect_checkbox_states();

    let states_to_restore = if let Some(saved_states) = model.tree.states.take() {
        let mut merged = saved_states;

        for (path, checked) in current_states {
            merged.insert(path, checked);
        }

        merged
    } else {
        current_states
    };

    model.tree.nodes = nodes;
    model.tree.restore_checkbox_states(&states_to_restore);
    model.tree.load_status = LoadStatus::Loaded;

    sync_to_active_session(model);

    if !model.options.auto_index_on_startup {
        Cmd::None
    } else {
        Cmd::StartIndexing {
            paths: model.tree.nodes.iter().map(|n| n.path.clone()).collect(),
            options: Arc::clone(&model.options),
        }
    }
}

fn handle_tree_load_progress(model: &mut Model, current: String, processed: usize, total: usize) -> Cmd {
    model.tree.load_status = LoadStatus::Loading {
        message: current,
        progress: (processed, total),
    };

    Cmd::None
}

fn handle_tree_load_failed(model: &mut Model, error: String) -> Cmd {
    model.tree.load_status = LoadStatus::Failed(error);
    Cmd::None
}
