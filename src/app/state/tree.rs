use std::collections::HashMap;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};

use crate::model::node::FileNode;

use super::SearchModel;

#[derive(Clone, Default, Deserialize, Serialize)]
pub struct TreeModel {
    pub nodes: Vec<FileNode>,
    pub output: String,
    pub load_status: LoadStatus,
    #[serde(skip)]
    pub states: Option<HashMap<PathBuf, bool>>,
    #[serde(skip)]
    pub file_count: usize,
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
            file_count: 0,
        }
    }

    pub fn collect_checkbox_states(&self) -> HashMap<PathBuf, bool> {
        let mut states = HashMap::new();
        for node in &self.nodes {
            node.collect_checkbox_states_recursive(&mut states);
        }
        states
    }

    pub fn count_files(&self) -> usize {
        self.nodes.iter().map(count_files_recursive).sum()
    }

    pub fn restore_checkbox_states(&mut self, states: &HashMap<PathBuf, bool>) {
        for node in &mut self.nodes {
            node.restore_checkbox_states_recursive(states);
        }
    }

    pub fn gather_checked_paths(&self, search: &SearchModel) -> Vec<String> {
        let mut results = Vec::new();
        let query = if search.has_query() { &search.query } else { "" };

        for node in &self.nodes {
            node.gather_checked_paths_recursive(&mut results, query);
        }
        results
    }

    pub fn create_filtered_tree(&self, search: &SearchModel) -> Vec<FileNode> {
        let query = if search.has_query() { &search.query } else { "" };
        self.nodes.iter().filter_map(|n| n.filter_selected(query)).collect()
    }

    pub fn update_file_count(&mut self) {
        self.file_count = self.count_files();
    }
}

fn count_files_recursive(node: &FileNode) -> usize {
    if node.is_file() {
        1
    } else {
        node.children.iter().map(count_files_recursive).sum()
    }
}
