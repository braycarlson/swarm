pub mod app;
pub mod copy;
pub mod filter;
pub mod options;
pub mod render;
pub mod search;
pub mod session;
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
}
