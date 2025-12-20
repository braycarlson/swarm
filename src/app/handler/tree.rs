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
        Tree::PropagateStarted => handle_propagate_started(model),
        Tree::PropagateCompleted(nodes) => handle_propagate_completed(model, nodes),
        Tree::PropagateFailed(error) => handle_propagate_failed(error),
        Tree::BackgroundLoadProgress { loaded, total } => { handle_background_load_progress(model, loaded, total) }
        Tree::BackgroundLoadCompleted(nodes) => { handle_background_load_completed(model, nodes) }
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

    let builder = CmdBuilder::new()
        .add(Cmd::RefreshTree {
            nodes: model.tree.nodes.clone(),
            options: Arc::clone(&model.options),
        });

    builder.build()
}

fn handle_tree_node_toggled(model: &mut Model, path: Vec<usize>, checked: bool, propagate: bool) -> Cmd {
    let path_u32: Vec<u32> = path.iter().map(|&x| x as u32).collect();

    if !propagate {
        toggle_node(&mut model.tree.nodes, &path_u32, checked, false, &model.options);
        sync_to_active_session(model);
        return Cmd::None;
    }

    if let Some(result) = model.background_loader.check_results()
        && let crate::services::worker::BackgroundLoadResult::NodesUpdated(nodes) = result {
            let current_states = model.tree.collect_checkbox_states();

            model.tree.nodes = nodes;
            model.tree.restore_checkbox_states(&current_states);

            sync_to_active_session(model);
        }

    let mut current = &model.tree.nodes;
    let mut target_is_dir = false;
    let mut has_unloaded = false;

    for &index in &path_u32 {
        if let Some(node) = current.get(index as usize) {
            if path_u32.last() == Some(&index) {
                target_is_dir = node.is_directory();

                if target_is_dir {
                    has_unloaded = check_has_unloaded(node);
                }

                break;
            }

            current = &node.children;
        }
    }

    if target_is_dir && has_unloaded {
        model.tree.load_status = LoadStatus::Loading {
            message: "Loading directories...".to_string(),
            progress: (0, 0),
        };

        return Cmd::PropagateCheckedWithLoad {
            nodes: model.tree.nodes.clone(),
            path: path_u32,
            checked,
            options: Arc::clone(&model.options),
        };
    }

    toggle_node(&mut model.tree.nodes, &path_u32, checked, propagate, &model.options);
    sync_to_active_session(model);

    Cmd::None
}

fn check_has_unloaded(node: &FileNode) -> bool {
    if node.is_directory() && !node.loaded {
        return true;
    }

    for child in &node.children {
        if check_has_unloaded(child) {
            return true;
        }
    }

    false
}

fn handle_tree_node_expanded(model: &mut Model, path: Vec<usize>) -> Cmd {
    let path_u32: Vec<u32> = path.iter().map(|&x| x as u32).collect();
    load_node_children(&mut model.tree.nodes, &path_u32, &model.options);
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

    model.tree.nodes = nodes.clone();
    model.tree.restore_checkbox_states(&states_to_restore);
    model.tree.load_status = LoadStatus::Loaded;
    model.tree.update_file_count();

    model.refresh_git_status();

    sync_to_active_session(model);

    model.background_loader.start_loading(nodes, (*model.options).clone());

    Cmd::None
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

fn handle_propagate_started(model: &mut Model) -> Cmd {
    model.tree.load_status = LoadStatus::Loading {
        message: "Loading directories...".to_string(),
        progress: (0, 0),
    };

    Cmd::None
}

fn handle_propagate_completed(model: &mut Model, nodes: Vec<FileNode>) -> Cmd {
    model.tree.nodes = nodes;
    model.tree.load_status = LoadStatus::Loaded;
    model.tree.update_file_count();
    sync_to_active_session(model);

    Cmd::None
}

fn handle_propagate_failed(error: String) -> Cmd {
    eprintln!("Propagate check with load failed: {}", error);
    Cmd::None
}

fn handle_background_load_progress(_model: &mut Model, _loaded: usize, _total: usize) -> Cmd {
    Cmd::None
}

fn handle_background_load_completed(model: &mut Model, nodes: Vec<FileNode>) -> Cmd {
    let current_states = model.tree.collect_checkbox_states();
    model.tree.nodes = nodes;
    model.tree.restore_checkbox_states(&current_states);
    model.tree.update_file_count();
    sync_to_active_session(model);

    Cmd::None
}
