use std::path::PathBuf;

use rustc_hash::FxHashSet;
use serde::{Deserialize, Serialize};

use crate::model::node::{self, FileNode};
use crate::model::selection;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub enum LoadStatus {
    Failed,
    Loaded,
    Loading(String),
    #[default]
    NotStarted,
}

#[derive(Clone, Default, Deserialize, Serialize)]
pub struct TreeModel {
    #[serde(skip)]
    pub background_loading: bool,
    #[serde(skip)]
    pub files_count: u32,
    #[serde(skip)]
    pub files_label: String,
    #[serde(skip)]
    pub load_status: LoadStatus,
    pub nodes: Vec<FileNode>,
    #[serde(skip)]
    pub states: Option<FxHashSet<PathBuf>>,
}

impl TreeModel {
    pub fn collect_checked_paths(&self) -> FxHashSet<PathBuf> {
        selection::collect_checked_paths(&self.nodes)
    }

    pub fn is_loading(&self) -> bool {
        matches!(self.load_status, LoadStatus::Loading(_))
    }

    pub fn new(paths: Vec<PathBuf>) -> Self {
        let nodes: Vec<FileNode> = paths.into_iter().map(FileNode::new).collect();

        Self {
            background_loading: false,
            files_count: 0,
            files_label: String::new(),
            load_status: LoadStatus::NotStarted,
            nodes,
            states: None,
        }
    }

    pub fn replace_nodes(&mut self, nodes: Vec<FileNode>) {
        let checked = self.collect_checked_paths();

        self.nodes = nodes;

        selection::restore_checked_paths(&mut self.nodes, &checked);
        self.update_files_count();
    }

    pub fn restore_checked_paths(&mut self, paths: &FxHashSet<PathBuf>) {
        selection::restore_checked_paths(&mut self.nodes, paths);
    }

    pub fn update_files_count(&mut self) {
        let count = node::count_files(&self.nodes);

        if count == self.files_count {
            if !self.files_label.is_empty() {
                return;
            }
        }

        self.files_count = count;
        self.files_label = format!("{count} files");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::node::NodeKind;

    fn tree() -> TreeModel {
        let mut root = FileNode::with_kind(PathBuf::from("/root"), NodeKind::Directory);

        root.children = vec![
            FileNode::with_kind(PathBuf::from("/root/a.rs"), NodeKind::File),
            FileNode::with_kind(PathBuf::from("/root/b.rs"), NodeKind::File),
        ];

        root.loaded = true;

        let mut model = TreeModel::new(Vec::new());

        model.nodes = vec![root];

        model
    }

    #[test]
    fn the_files_label_follows_the_count() {
        let mut model = tree();

        model.update_files_count();

        assert_eq!(model.files_count, 2);
        assert_eq!(model.files_label, "2 files");
    }

    #[test]
    fn replacing_nodes_keeps_the_checked_state() {
        let mut model = tree();

        model.nodes[0].children[1].checked = true;

        let fresh = tree().nodes;

        model.replace_nodes(fresh);

        assert!(model.nodes[0].children[1].checked);
        assert!(!model.nodes[0].children[0].checked);
        assert_eq!(model.files_count, 2);
    }
}
