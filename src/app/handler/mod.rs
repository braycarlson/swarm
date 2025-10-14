pub mod app;
pub mod copy;
pub mod filter;
pub mod index;
pub mod options;
pub mod search;
pub mod session;
pub mod tree;
pub mod tree_gen;

use crate::app::state::Model;
use crate::model::node::FileNode;
use crate::model::options::Options;
use crate::services::tree::operations::TreeOperations;

pub fn sync_to_active_session(model: &mut Model) {
    model.sessions.sync_from_tree_and_search(model.tree.clone(), model.search.clone());
}

pub fn load_node_children(nodes: &mut [FileNode], path: &[usize], options: &Options) {
    if path.is_empty() {
        return;
    }

    let mut current = nodes;

    for (i, &index) in path.iter().enumerate() {
        if i == path.len() - 1 {
            if let Some(node) = current.get_mut(index) {
                if !node.loaded && node.is_directory() {
                    use crate::services::tree::loader::load_children;
                    let _ = load_children(node, options);
                }
            }

            break;
        } else if let Some(node) = current.get_mut(index) {
            current = &mut node.children;
        } else {
            break;
        }
    }
}

pub fn toggle_node(nodes: &mut [FileNode], path: &[usize], checked: bool, propagate: bool) {
    if path.is_empty() {
        return;
    }

    let mut current = nodes;

    for (i, &index) in path.iter().enumerate() {
        if i == path.len() - 1 {
            if let Some(node) = current.get_mut(index) {
                node.checked = checked;

                if propagate && node.is_directory() {
                    node.propagate_checked(checked);
                }
            }

            break;
        } else if let Some(node) = current.get_mut(index) {
            current = &mut node.children;
        } else {
            break;
        }
    }
}
