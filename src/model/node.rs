use core::slice;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

pub const TREE_DEPTH_MAX: u32 = 256;
const STACK_CAPACITY: usize = 32;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct FileNode {
    pub checked: bool,
    pub children: Vec<Self>,
    pub kind: NodeKind,
    #[serde(skip)]
    pub load_failed: bool,
    pub loaded: bool,
    #[serde(skip)]
    pub name: Box<str>,
    #[serde(skip)]
    pub name_lowercase: Box<str>,
    pub path: PathBuf,
}

impl FileNode {
    pub fn has_unloaded_directory(&self) -> bool {
        if self.is_directory() {
            if !self.loaded {
                return true;
            }
        }

        any(&self.children, |node| node.is_directory() && !node.loaded)
    }

    pub fn is_directory(&self) -> bool {
        self.kind == NodeKind::Directory
    }

    pub fn is_file(&self) -> bool {
        self.kind == NodeKind::File
    }

    pub fn is_selected(&self) -> bool {
        if self.checked {
            return true;
        }

        any(&self.children, |node| node.checked)
    }

    pub fn new(path: PathBuf) -> Self {
        let kind = if path.is_dir() {
            NodeKind::Directory
        } else {
            NodeKind::File
        };

        Self::with_kind(path, kind)
    }

    pub fn recompute_name_cache(&mut self) {
        let (name, name_lowercase) = names_from_path(&self.path);

        self.name = name;
        self.name_lowercase = name_lowercase;

        debug_assert_eq!(self.name.len(), self.name_lowercase.len());
    }

    pub fn with_kind(path: PathBuf, kind: NodeKind) -> Self {
        let (name, name_lowercase) = names_from_path(&path);

        debug_assert_eq!(name.len(), name_lowercase.len());

        Self {
            checked: false,
            children: Vec::new(),
            kind,
            load_failed: false,
            loaded: false,
            name,
            name_lowercase,
            path,
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum NodeKind {
    Directory,
    File,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Visit {
    Descend,
    Skip,
    Stop,
}

pub fn any(nodes: &[FileNode], mut predicate: impl FnMut(&FileNode) -> bool) -> bool {
    let mut found = false;

    walk(nodes, |node, _| {
        if predicate(node) {
            found = true;

            return Visit::Stop;
        }

        Visit::Descend
    });

    found
}

pub fn count_files(nodes: &[FileNode]) -> u32 {
    let mut count: u32 = 0;

    visit(nodes, |node, _| {
        if node.is_file() {
            count += 1;
        }
    });

    count
}

pub fn depth_is_bounded(nodes: &[FileNode]) -> bool {
    let mut bounded = true;

    walk(nodes, |node, depth| {
        if node.children.is_empty() {
            return Visit::Skip;
        }

        if depth + 1 < TREE_DEPTH_MAX {
            return Visit::Descend;
        }

        bounded = false;

        Visit::Stop
    });

    bounded
}

fn names_from_path(path: &Path) -> (Box<str>, Box<str>) {
    let name = path.file_name().map_or_else(
        || path.to_string_lossy().into_owned(),
        |name| name.to_string_lossy().into_owned(),
    );

    let name_lowercase = name.to_ascii_lowercase();

    debug_assert_eq!(name_lowercase.len(), name.len());

    (name.into_boxed_str(), name_lowercase.into_boxed_str())
}

pub fn visit(nodes: &[FileNode], mut visitor: impl FnMut(&FileNode, u32)) {
    walk(nodes, |node, depth| {
        visitor(node, depth);

        Visit::Descend
    });
}

pub fn visit_mut(nodes: &mut [FileNode], mut visitor: impl FnMut(&mut FileNode, u32)) {
    walk_mut(nodes, |node, depth| {
        visitor(node, depth);

        Visit::Descend
    });
}

pub fn walk(nodes: &[FileNode], mut visitor: impl FnMut(&FileNode, u32) -> Visit) {
    let mut stack: Vec<slice::Iter<'_, FileNode>> = Vec::with_capacity(STACK_CAPACITY);
    let mut depth: u32 = 0;

    stack.push(nodes.iter());

    while !stack.is_empty() {
        debug_assert_eq!(stack.len(), depth as usize + 1);

        let Some(node) = stack.last_mut().and_then(Iterator::next) else {
            let _ = stack.pop();
            depth = depth.saturating_sub(1);

            continue;
        };

        match visitor(node, depth) {
            Visit::Descend => {}
            Visit::Skip => continue,
            Visit::Stop => return,
        }

        if node.children.is_empty() {
            continue;
        }

        assert!(depth + 1 < TREE_DEPTH_MAX);

        stack.push(node.children.iter());
        depth += 1;
    }
}

pub fn walk_mut(nodes: &mut [FileNode], mut visitor: impl FnMut(&mut FileNode, u32) -> Visit) {
    let mut stack: Vec<slice::IterMut<'_, FileNode>> = Vec::with_capacity(STACK_CAPACITY);
    let mut depth: u32 = 0;

    stack.push(nodes.iter_mut());

    while !stack.is_empty() {
        debug_assert_eq!(stack.len(), depth as usize + 1);

        let Some(node) = stack.last_mut().and_then(Iterator::next) else {
            let _ = stack.pop();
            depth = depth.saturating_sub(1);

            continue;
        };

        match visitor(node, depth) {
            Visit::Descend => {}
            Visit::Skip => continue,
            Visit::Stop => return,
        }

        if node.children.is_empty() {
            continue;
        }

        assert!(depth + 1 < TREE_DEPTH_MAX);

        stack.push(node.children.iter_mut());
        depth += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn directory(name: &str, children: Vec<FileNode>) -> FileNode {
        let mut node = FileNode::with_kind(PathBuf::from(name), NodeKind::Directory);

        node.children = children;
        node.loaded = true;

        node
    }

    fn file(name: &str) -> FileNode {
        FileNode::with_kind(PathBuf::from(name), NodeKind::File)
    }

    fn sample() -> Vec<FileNode> {
        vec![directory(
            "root",
            vec![
                file("root/a.rs"),
                directory("root/nested", vec![file("root/nested/b.rs")]),
            ],
        )]
    }

    fn chain(length: u32) -> Vec<FileNode> {
        let mut node = file("leaf");

        for index in 0..length {
            node = directory(&format!("level_{index}"), vec![node]);
        }

        vec![node]
    }

    #[test]
    fn visit_reaches_every_node_once() {
        let nodes = sample();
        let mut count: u32 = 0;

        visit(&nodes, |_, _| count += 1);

        assert_eq!(count, 4);
    }

    #[test]
    fn visit_reports_depth() {
        let nodes = sample();
        let mut depths = Vec::new();

        visit(&nodes, |node, depth| {
            depths.push((node.name.to_string(), depth));
        });

        assert_eq!(depths[0], ("root".to_owned(), 0));
        assert_eq!(depths[1], ("a.rs".to_owned(), 1));
        assert_eq!(depths[3], ("b.rs".to_owned(), 2));
    }

    #[test]
    fn walk_honours_skip() {
        let nodes = sample();
        let mut count: u32 = 0;

        walk(&nodes, |_, depth| {
            count += 1;

            if depth < 1 {
                Visit::Descend
            } else {
                Visit::Skip
            }
        });

        assert_eq!(count, 3);
    }

    #[test]
    fn walk_honours_stop() {
        let nodes = sample();
        let mut count: u32 = 0;

        walk(&nodes, |_, _| {
            count += 1;

            Visit::Stop
        });

        assert_eq!(count, 1);
    }

    #[test]
    fn any_stops_at_the_first_match() {
        let nodes = sample();
        let mut visited: u32 = 0;

        let found = any(&nodes, |node| {
            visited += 1;

            node.is_file()
        });

        assert!(found);
        assert_eq!(visited, 2);
    }

    #[test]
    fn any_reports_false_on_an_empty_forest() {
        assert!(!any(&[], |_| true));
    }

    #[test]
    fn files_are_counted() {
        assert_eq!(count_files(&sample()), 2);
        assert_eq!(count_files(&[]), 0);
    }

    #[test]
    fn is_selected_sees_descendants() {
        let mut nodes = sample();

        assert!(!nodes[0].is_selected());

        nodes[0].children[1].children[0].checked = true;

        assert!(nodes[0].is_selected());
    }

    #[test]
    fn unloaded_directories_are_detected() {
        let mut nodes = sample();

        assert!(!nodes[0].has_unloaded_directory());

        nodes[0].children[1].loaded = false;

        assert!(nodes[0].has_unloaded_directory());
    }

    #[test]
    fn recompute_names_repopulates_skipped_caches() {
        let mut nodes = sample();

        visit_mut(&mut nodes, |node, _| {
            node.name = Box::default();
            node.name_lowercase = Box::default();
        });

        visit_mut(&mut nodes, |node, _| node.recompute_name_cache());

        assert_eq!(&*nodes[0].name, "root");
        assert_eq!(&*nodes[0].children[1].children[0].name_lowercase, "b.rs");
    }

    #[test]
    fn depth_bound_accepts_a_tree_at_the_limit() {
        assert!(depth_is_bounded(&chain(TREE_DEPTH_MAX - 1)));
    }

    #[test]
    fn depth_bound_rejects_a_tree_past_the_limit() {
        assert!(!depth_is_bounded(&chain(TREE_DEPTH_MAX)));
    }
}
