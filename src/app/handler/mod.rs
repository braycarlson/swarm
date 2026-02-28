pub mod app;
pub mod copy;
pub mod filter;
pub mod options;
pub mod render;
pub mod search;
pub mod session;
pub mod skeleton;
pub mod tree;

use crate::app::state::Model;
use crate::model::node::FileNode;
use crate::model::options::Options;
use crate::services::tree::traversal::Traversable;

const MAX_PATH_DEPTH: u32 = 100;

pub fn sync_to_active_session(model: &mut Model) {
    model.sessions.sync_from_tree_and_search(model.tree.clone(), model.search.clone());
}

pub fn load_node_children(nodes: &mut [FileNode], path: &[u32], options: &Options) {
    if path.is_empty() {
        return;
    }

    let mut current = nodes;
    let path_len = path.len() as u32;

    let mut depth: u32 = 0;

    while depth < path_len {
        if depth >= MAX_PATH_DEPTH {
            break;
        }

        let index_value = path[depth as usize];
        let is_last = (depth + 1) == path_len;

        if is_last {
            let node_option = current.get_mut(index_value as usize);

            if node_option.is_none() {
                break;
            }

            let node = node_option.unwrap();
            let should_load = !node.loaded && node.is_directory();

            if should_load {
                use crate::services::tree::loader::load_children;
                let _ = load_children(node, options);
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
}

pub fn toggle_node(nodes: &mut [FileNode], path: &[u32], checked: bool, propagate: bool, options: &Options) {
    if path.is_empty() {
        return;
    }

    let mut current = &mut *nodes;
    let path_len = path.len() as u32;

    let mut depth: u32 = 0;

    while depth < path_len {
        if depth >= MAX_PATH_DEPTH {
            break;
        }

        let index_value = path[depth as usize];
        let is_last = (depth + 1) == path_len;

        if is_last {
            let node_option = current.get_mut(index_value as usize);

            if node_option.is_none() {
                break;
            }

            let node = node_option.unwrap();
            node.checked = checked;

            if propagate && node.is_directory() {
                node.propagate_checked_with_load(checked, options);
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

    update_ancestors(nodes, path, checked);
}

fn update_ancestors(nodes: &mut [FileNode], path: &[u32], checked: bool) {
    if path.len() <= 1 {
        return;
    }

    let ancestor_count = path.len() - 1;

    if checked {
        let mut current = &mut *nodes;

        for &index in &path[..ancestor_count] {
            if let Some(node) = current.get_mut(index as usize) {
                node.checked = true;
                current = &mut node.children;
            } else {
                break;
            }
        }
    } else {
        for depth in (0..ancestor_count).rev() {
            let has_selected = has_selected_child(&*nodes, &path[..=depth]);

            if has_selected {
                break;
            }

            let mut current = &mut *nodes;

            for &index in &path[..depth] {
                if let Some(node) = current.get_mut(index as usize) {
                    current = &mut node.children;
                } else {
                    return;
                }
            }

            if let Some(node) = current.get_mut(path[depth] as usize) {
                node.checked = false;
            }
        }
    }
}

fn has_selected_child(nodes: &[FileNode], path: &[u32]) -> bool {
    let mut current = nodes;
    let last = path.len() - 1;

    for (i, &index) in path.iter().enumerate() {
        if let Some(node) = current.get(index as usize) {
            if i == last {
                return node.children.iter().any(FileNode::is_selected);
            }

            current = &node.children;
        } else {
            return false;
        }
    }

    false
}
