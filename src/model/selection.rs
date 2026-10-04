use std::path::PathBuf;

use rustc_hash::FxHashSet;

use crate::model::node::{self, FileNode, TREE_DEPTH_MAX, Visit};

const STACK_CAPACITY: usize = 32;

struct CheckedFrame<'tree> {
    child_index: usize,
    descendant_checked: bool,
    node: &'tree FileNode,
}

impl<'tree> CheckedFrame<'tree> {
    fn new(node: &'tree FileNode) -> Self {
        Self {
            child_index: 0,
            descendant_checked: false,
            node,
        }
    }
}

fn check_ancestors(nodes: &mut [FileNode], ancestors: &[u32]) {
    assert_ne!(ancestors.len(), 0);

    let mut current = nodes;

    for &index in ancestors {
        let Some(node) = current.get_mut(index as usize) else {
            return;
        };

        node.checked = true;
        current = &mut node.children;
    }
}

pub fn collect_checked_paths(nodes: &[FileNode]) -> FxHashSet<PathBuf> {
    let mut paths = FxHashSet::default();

    node::visit(nodes, |node, _| {
        if node.checked {
            let inserted = paths.insert(node.path.clone());

            debug_assert!(inserted);
        }
    });

    paths
}

fn directories_with_checked_descendants(nodes: &[FileNode]) -> FxHashSet<PathBuf> {
    let mut marked = FxHashSet::default();
    let mut stack: Vec<CheckedFrame<'_>> = Vec::with_capacity(STACK_CAPACITY);

    for root in nodes {
        stack.push(CheckedFrame::new(root));

        while !stack.is_empty() {
            assert!(stack.len() <= TREE_DEPTH_MAX as usize);

            let top = stack.len() - 1;
            let frame = &mut stack[top];
            let node = frame.node;

            if let Some(child) = node.children.get(frame.child_index) {
                frame.child_index += 1;
                stack.push(CheckedFrame::new(child));

                continue;
            }

            let descendant_checked = frame.descendant_checked;

            let _ = stack.pop();

            if node.is_directory() {
                if descendant_checked {
                    let inserted = marked.insert(node.path.clone());

                    debug_assert!(inserted);
                }
            }

            if let Some(parent) = stack.last_mut() {
                parent.descendant_checked |= node.checked || descendant_checked;
            }
        }
    }

    marked
}

pub fn find_node<'tree>(nodes: &'tree [FileNode], path: &[u32]) -> Option<&'tree FileNode> {
    debug_assert!(path.len() <= TREE_DEPTH_MAX as usize);

    let (&last, ancestors) = path.split_last()?;
    let mut current = nodes;

    for &index in ancestors {
        current = &current.get(index as usize)?.children;
    }

    current.get(last as usize)
}

pub fn find_node_mut<'tree>(
    nodes: &'tree mut [FileNode],
    path: &[u32],
) -> Option<&'tree mut FileNode> {
    debug_assert!(path.len() <= TREE_DEPTH_MAX as usize);

    let (&last, ancestors) = path.split_last()?;
    let mut current = nodes;

    for &index in ancestors {
        current = &mut current.get_mut(index as usize)?.children;
    }

    current.get_mut(last as usize)
}

fn has_selected_child(nodes: &[FileNode], path: &[u32]) -> bool {
    assert_ne!(path.len(), 0);

    find_node(nodes, path).is_some_and(|node| node::any(&node.children, |child| child.checked))
}

pub(crate) fn restore_checked_paths(nodes: &mut [FileNode], paths: &FxHashSet<PathBuf>) {
    node::visit_mut(nodes, |node, _| node.checked = paths.contains(&node.path));
}

pub fn select_paths_by_suffix(nodes: &mut [FileNode], suffixes: &[String]) -> u32 {
    assert_ne!(suffixes.len(), 0);
    debug_assert!(suffixes.iter().all(|suffix| !suffix.contains('\\')));

    let mut selected: u32 = 0;
    let mut normalized = String::new();

    node::visit_mut(nodes, |node, _| {
        if !node.is_file() {
            return;
        }

        normalized.clear();
        normalized.push_str(&node.path.to_string_lossy());

        if normalized.contains('\\') {
            normalized = normalized.replace('\\', "/");
        }

        if suffixes
            .iter()
            .any(|suffix| normalized.ends_with(suffix.as_str()))
        {
            node.checked = true;
            selected += 1;
        }
    });

    let marked = directories_with_checked_descendants(nodes);

    node::walk_mut(nodes, |node, _| {
        if !node.is_directory() {
            return Visit::Skip;
        }

        if marked.contains(&node.path) {
            node.checked = true;
        }

        Visit::Descend
    });

    selected
}

pub fn set_all_checked(nodes: &mut [FileNode], checked: bool) {
    node::visit_mut(nodes, |node, _| node.checked = checked);
}

fn uncheck_empty_ancestors(nodes: &mut [FileNode], path: &[u32]) {
    assert!(path.len() > 1);

    for length in (1..path.len()).rev() {
        let ancestor = &path[..length];

        if has_selected_child(nodes, ancestor) {
            return;
        }

        if let Some(node) = find_node_mut(nodes, ancestor) {
            node.checked = false;
        }
    }
}

pub fn update_ancestors(nodes: &mut [FileNode], path: &[u32], checked: bool) {
    debug_assert_ne!(path.len(), 0);

    if path.len() <= 1 {
        return;
    }

    if checked {
        check_ancestors(nodes, &path[..path.len() - 1]);

        return;
    }

    uncheck_empty_ancestors(nodes, path);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::node::NodeKind;

    fn directory(name: &str, children: Vec<FileNode>) -> FileNode {
        let mut node = FileNode::with_kind(PathBuf::from(name), NodeKind::Directory);

        node.children = children;
        node.loaded = true;

        node
    }

    fn file(name: &str) -> FileNode {
        FileNode::with_kind(PathBuf::from(name), NodeKind::File)
    }

    fn tree() -> Vec<FileNode> {
        vec![directory(
            "/root",
            vec![
                file("/root/a.rs"),
                directory("/root/nested", vec![file("/root/nested/b.rs")]),
            ],
        )]
    }

    fn count_checked_files(nodes: &[FileNode]) -> u32 {
        let mut count: u32 = 0;

        node::visit(nodes, |node, _| {
            if node.is_file() {
                if node.checked {
                    count += 1;
                }
            }
        });

        count
    }

    #[test]
    fn find_node_resolves_a_deep_path() {
        let mut nodes = tree();

        assert_eq!(&*find_node(&nodes, &[0, 1, 0]).expect("the node exists").name, "b.rs");
        assert_eq!(&*find_node_mut(&mut nodes, &[0, 1]).expect("the node exists").name, "nested");
    }

    #[test]
    fn find_node_rejects_an_empty_path() {
        let mut nodes = tree();

        assert!(find_node(&nodes, &[]).is_none());
        assert!(find_node_mut(&mut nodes, &[]).is_none());
    }

    #[test]
    fn find_node_rejects_an_out_of_range_index() {
        let nodes = tree();

        assert!(find_node(&nodes, &[0, 9]).is_none());
        assert!(find_node(&nodes, &[9]).is_none());
        assert!(find_node(&nodes, &[0, 0, 0]).is_none());
    }

    #[test]
    fn checking_a_leaf_checks_its_ancestors() {
        let mut nodes = tree();

        update_ancestors(&mut nodes, &[0, 1, 0], true);

        assert!(nodes[0].checked);
        assert!(nodes[0].children[1].checked);
    }

    #[test]
    fn unchecking_the_last_leaf_unchecks_its_ancestors() {
        let mut nodes = tree();

        update_ancestors(&mut nodes, &[0, 1, 0], true);
        update_ancestors(&mut nodes, &[0, 1, 0], false);

        assert!(!nodes[0].checked);
        assert!(!nodes[0].children[1].checked);
    }

    #[test]
    fn a_remaining_sibling_keeps_the_ancestor_checked() {
        let mut nodes = tree();

        nodes[0].children[0].checked = true;
        update_ancestors(&mut nodes, &[0, 0], true);
        update_ancestors(&mut nodes, &[0, 1, 0], false);

        assert!(nodes[0].checked);
    }

    #[test]
    fn checked_paths_round_trip() {
        let mut nodes = tree();

        set_all_checked(&mut nodes, true);

        let paths = collect_checked_paths(&nodes);

        assert_eq!(paths.len(), 4);

        set_all_checked(&mut nodes, false);
        restore_checked_paths(&mut nodes, &paths);

        assert_eq!(count_checked_files(&nodes), 2);
    }

    #[test]
    fn restore_clears_nodes_absent_from_the_set() {
        let mut nodes = tree();

        set_all_checked(&mut nodes, true);
        restore_checked_paths(&mut nodes, &FxHashSet::default());

        assert_eq!(count_checked_files(&nodes), 0);
    }

    #[test]
    fn selection_marks_ancestors() {
        let mut nodes = tree();
        let selected = select_paths_by_suffix(&mut nodes, &["nested/b.rs".to_owned()]);

        assert_eq!(selected, 1);
        assert!(nodes[0].checked);
        assert!(nodes[0].children[1].checked);
        assert!(!nodes[0].children[0].checked);
    }
}
