use std::path::PathBuf;

use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};

use crate::model::node::FileNode;
use crate::services::filesystem::git::GitService;

use super::SearchModel;

#[derive(Clone, Default, Deserialize, Serialize)]
pub struct TreeModel {
    pub nodes: Vec<FileNode>,
    pub output: String,
    pub load_status: LoadStatus,
    #[serde(skip)]
    pub states: Option<FxHashMap<PathBuf, bool>>,
    #[serde(skip)]
    pub files_count: usize,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum LoadStatus {
    NotStarted,
    Loading { message: String, progress: (usize, usize) },
    Loaded,
    Failed(String),
}

impl Default for LoadStatus {
    fn default() -> Self {
        Self::NotStarted
    }
}

impl TreeModel {
    pub fn new(paths: Vec<String>) -> Self {
        let nodes = paths.into_iter()
            .map(|p| FileNode::new(PathBuf::from(p)))
            .collect();

        Self {
            nodes,
            output: String::new(),
            load_status: LoadStatus::NotStarted,
            states: None,
            files_count: 0,
        }
    }

    pub fn collect_checkbox_states(&self) -> FxHashMap<PathBuf, bool> {
        let mut states = FxHashMap::default();
        for node in &self.nodes {
            node.collect_checkbox_states_recursive(&mut states);
        }
        states
    }

    pub fn count_files(&self) -> usize {
        self.nodes.iter().map(count_files_recursive).sum()
    }

    pub fn restore_checkbox_states(&mut self, states: &FxHashMap<PathBuf, bool>) {
        for node in &mut self.nodes {
            node.restore_checkbox_states_recursive(states);
        }
    }

    pub fn gather_checked_paths(&self, search: &SearchModel) -> Vec<String> {
        self.gather_checked_paths_with_git(search, None)
    }

    pub fn gather_checked_paths_with_git(&self, search: &SearchModel, git: Option<&GitService>) -> Vec<String> {
        let mut results = Vec::new();
        let query = search.parsed();

        for node in &self.nodes {
            node.gather_checked_paths_with_git(&mut results, &query, git);
        }
        results
    }

    pub fn create_filtered_tree(&self, search: &SearchModel) -> Vec<FileNode> {
        self.create_filtered_tree_with_git(search, None)
    }

    pub fn create_filtered_tree_with_git(&self, search: &SearchModel, git: Option<&GitService>) -> Vec<FileNode> {
        let query = search.parsed();
        self.nodes.iter().filter_map(|n| n.filter_selected_with_git(&query, git)).collect()
    }

    pub fn update_files_count(&mut self) {
        self.files_count = self.count_files();
    }
}

fn count_files_recursive(node: &FileNode) -> usize {
    if node.is_file() {
        1
    } else {
        node.children.iter().map(count_files_recursive).sum()
    }
}
