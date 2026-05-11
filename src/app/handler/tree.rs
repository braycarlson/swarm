use std::sync::Arc;

use crate::app::message::{Command, CommandBuilder, Tree};
use crate::app::state::{LoadStatus, Model, UiState};
use crate::app::state::SearchModel;
use crate::model::node::FileNode;
use crate::services::filesystem::git::GitService;

use super::{load_node_children, sync_to_active_session, toggle_node};

pub fn handle(model: &mut Model, ui: &mut UiState, message: Tree) -> Command {
    match message {
        Tree::RefreshRequested => handle_tree_refresh_requested(model, ui),
        Tree::NodeToggled { path, checked, propagate } => { handle_tree_node_toggled(model, path, checked, propagate) }
        Tree::NodeExpanded { path } => handle_tree_node_expanded(model, path),
        Tree::SelectAll => handle_select_all(model),
        Tree::DeselectAll => handle_deselect_all(model),
        Tree::Loaded(nodes) => handle_tree_loaded(model, ui, nodes),
        Tree::LoadProgress { current, processed, total } => { handle_tree_load_progress(model, current, processed, total) }
        Tree::LoadFailed(error) => handle_tree_load_failed(model, error),
        Tree::PropagateStarted => handle_propagate_started(model),
        Tree::PropagateCompleted(nodes) => handle_propagate_completed(model, nodes),
        Tree::PropagateFailed(error) => handle_propagate_failed(error),
        Tree::BackgroundLoadProgress { loaded, total } => { handle_background_load_progress(model, loaded, total) }
        Tree::BackgroundLoadCompleted(nodes) => { handle_background_load_completed(model, nodes) }
        Tree::BulkSelectToggled => handle_bulk_select_toggled(ui),
        Tree::BulkSelectTextChanged(text) => handle_bulk_select_text_changed(ui, text),
        Tree::BulkSelectApplied => handle_bulk_select_applied(model, ui),
    }
}

fn handle_tree_refresh_requested(model: &mut Model, _ui: &mut UiState) -> Command {
    if matches!(model.tree.load_status, LoadStatus::Loading { .. }) {
        return Command::None;
    }

    model.tree.states = Some(model.tree.collect_checkbox_states());

    model.tree.load_status = LoadStatus::Loading {
        message: "Starting refresh...".to_string(),
        progress: (0, 0),
    };

    let nodes = std::mem::take(&mut model.tree.nodes);

    let builder = CommandBuilder::new()
        .add(Command::RefreshTree {
            nodes,
            options: Arc::clone(&model.options),
        });

    builder.build()
}

fn handle_tree_node_toggled(model: &mut Model, path: Vec<usize>, checked: bool, propagate: bool) -> Command {
    let path_u32: Vec<u32> = path.iter().map(|&x| x as u32).collect();

    if !propagate {
        toggle_node(&mut model.tree.nodes, &path_u32, checked, false, &model.options);
        sync_to_active_session(model);
        return Command::None;
    }

    if let Some(result) = model.background_loader.check_results()
        && let crate::services::worker::BackgroundLoadResult::NodesUpdated(nodes) = result {
            let current_states = model.tree.collect_checkbox_states();

            model.tree.nodes = nodes;
            model.tree.restore_checkbox_states(&current_states);

            sync_to_active_session(model);
        }

    let mut current = &model.tree.nodes;
    let mut target_is_directory = false;
    let mut has_unloaded = false;

    for &index in &path_u32 {
        if let Some(node) = current.get(index as usize) {
            if path_u32.last() == Some(&index) {
                target_is_directory = node.is_directory();

                if target_is_directory {
                    has_unloaded = check_has_unloaded(node);
                }

                break;
            }

            current = &node.children;
        }
    }

    if target_is_directory && has_unloaded {
        model.tree.load_status = LoadStatus::Loading {
            message: "Loading directories...".to_string(),
            progress: (0, 0),
        };

        let nodes = std::mem::take(&mut model.tree.nodes);

        return Command::PropagateCheckedWithLoad {
            nodes,
            path: path_u32,
            checked,
            options: Arc::clone(&model.options),
            search: model.search.clone(),
            git: model.git_service.clone(),
        };
    }

    if model.search.has_query() {
        toggle_node_filtered(&mut model.tree.nodes, &path_u32, checked, &model.options, &model.search, &model.git_service);
    } else {
        toggle_node(&mut model.tree.nodes, &path_u32, checked, propagate, &model.options);
    }

    sync_to_active_session(model);

    Command::None
}

fn toggle_node_filtered(
    nodes: &mut [FileNode],
    path: &[u32],
    checked: bool,
    options: &crate::model::options::Options,
    search: &SearchModel,
    git_service: &GitService,
) {
    use crate::services::tree::traversal::Traversable;

    if path.is_empty() {
        return;
    }

    let mut current = &mut *nodes;
    let path_length = path.len() as u32;
    let mut depth: u32 = 0;

    while depth < path_length {
        if depth >= super::PATH_DEPTH_MAX {
            break;
        }

        let index_value = path[depth as usize];
        let is_last = (depth + 1) == path_length;

        if is_last {
            let node_option = current.get_mut(index_value as usize);

            if node_option.is_none() {
                break;
            }

            let node = node_option.unwrap();
            node.checked = checked;

            if node.is_directory() {
                node.propagate_checked_filtered(checked, options, search, Some(git_service));
            }

            break;
        } else {
            let node_option = current.get_mut(index_value as usize);

            if node_option.is_none() {
                break;
            }

            let node = node_option.unwrap();
            current = &mut node.children;
        }

        depth += 1;
    }

    super::update_ancestors(nodes, path, checked);
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

fn handle_tree_node_expanded(model: &mut Model, path: Vec<usize>) -> Command {
    let path_u32: Vec<u32> = path.iter().map(|&x| x as u32).collect();
    load_node_children(&mut model.tree.nodes, &path_u32, &model.options);
    sync_to_active_session(model);

    Command::None
}

fn handle_tree_loaded(model: &mut Model, _ui: &mut UiState, nodes: Vec<FileNode>) -> Command {
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
    model.tree.update_files_count();

    model.refresh_git_status();

    sync_to_active_session(model);

    model.background_loader.start_loading(nodes, (*model.options).clone());

    Command::None
}

fn handle_tree_load_progress(model: &mut Model, current: String, processed: usize, total: usize) -> Command {
    model.tree.load_status = LoadStatus::Loading {
        message: current,
        progress: (processed, total),
    };

    Command::None
}

fn handle_tree_load_failed(model: &mut Model, error: String) -> Command {
    model.tree.load_status = LoadStatus::Failed(error);
    Command::None
}

fn handle_propagate_started(model: &mut Model) -> Command {
    model.tree.load_status = LoadStatus::Loading {
        message: "Loading directories...".to_string(),
        progress: (0, 0),
    };

    Command::None
}

fn handle_propagate_completed(model: &mut Model, nodes: Vec<FileNode>) -> Command {
    model.tree.nodes = nodes;
    model.tree.load_status = LoadStatus::Loaded;
    model.tree.update_files_count();
    sync_to_active_session(model);

    Command::None
}

fn handle_propagate_failed(error: String) -> Command {
    eprintln!("Propagate check with load failed: {}", error);
    Command::None
}

fn handle_background_load_progress(_model: &mut Model, _loaded: usize, _total: usize) -> Command {
    Command::None
}

fn handle_background_load_completed(model: &mut Model, nodes: Vec<FileNode>) -> Command {
    let current_states = model.tree.collect_checkbox_states();
    model.tree.nodes = nodes;
    model.tree.restore_checkbox_states(&current_states);
    model.tree.update_files_count();
    sync_to_active_session(model);

    Command::None
}

fn handle_select_all(model: &mut Model) -> Command {
    set_all_checked(&mut model.tree.nodes, true);
    model.tree.update_files_count();
    sync_to_active_session(model);
    Command::None
}

fn handle_deselect_all(model: &mut Model) -> Command {
    set_all_checked(&mut model.tree.nodes, false);
    model.tree.update_files_count();
    sync_to_active_session(model);
    Command::None
}

fn set_all_checked(nodes: &mut [FileNode], checked: bool) {
    for node in nodes {
        node.checked = checked;
        set_all_checked(&mut node.children, checked);
    }
}

fn handle_bulk_select_toggled(ui: &mut UiState) -> Command {
    ui.bulk_select_show = !ui.bulk_select_show;

    if !ui.bulk_select_show {
        ui.bulk_select_text.clear();
    }

    Command::None
}

fn handle_bulk_select_text_changed(ui: &mut UiState, text: String) -> Command {
    ui.bulk_select_text = text;
    Command::None
}

fn handle_bulk_select_applied(model: &mut Model, ui: &mut UiState) -> Command {
    let paths: Vec<String> = ui.bulk_select_text
        .lines()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty())
        .map(|line| line.replace('\\', "/"))
        .collect();

    if paths.is_empty() {
        return Command::None;
    }

    set_all_checked(&mut model.tree.nodes, false);
    bulk_select_recursive(&mut model.tree.nodes, &paths);
    model.tree.update_files_count();
    sync_to_active_session(model);

    let count = checked_count(&model.tree.nodes);
    ui.toast.success(format!("{} files selected", count));
    ui.bulk_select_show = false;
    ui.bulk_select_text.clear();

    Command::None
}

fn bulk_select_recursive(nodes: &mut [FileNode], paths: &[String]) {
    for node in nodes {
        let node_path = node.path.to_string_lossy().replace('\\', "/");

        if node.is_file() && paths.iter().any(|p| node_path.ends_with(p)) {
            node.checked = true;
        }

        if node.is_directory() {
            bulk_select_recursive(&mut node.children, paths);

            if node.children.iter().any(|c| c.checked) {
                node.checked = true;
            }
        }
    }
}

fn checked_count(nodes: &[FileNode]) -> usize {
    let mut count = 0;

    for node in nodes {
        if node.is_file() && node.checked {
            count += 1;
        }

        count += checked_count(&node.children);
    }

    count
}
