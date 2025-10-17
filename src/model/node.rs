use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::model::options::Options;
use crate::model::path::PathExtensions;
use crate::services::tree::operations::TreeOperations;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum NodeKind {
    Directory,
    File,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct FileNode {
    pub checked: bool,
    pub children: Vec<FileNode>,
    pub kind: NodeKind,
    pub loaded: bool,
    pub path: PathBuf,
}

impl FileNode {
    pub fn new(path: PathBuf) -> Self {
        let kind = if path.is_dir() {
            NodeKind::Directory
        } else {
            NodeKind::File
        };

        Self {
            checked: false,
            children: Vec::new(),
            kind,
            loaded: false,
            path,
        }
    }

    pub fn builder() -> FileNodeBuilder {
        FileNodeBuilder::default()
    }

    pub fn file_name(&self) -> Option<String> {
        self.path.file_name_string()
    }

    pub fn has_children(&self) -> bool {
        !self.children.is_empty()
    }

    pub fn is_directory(&self) -> bool {
        matches!(self.kind, NodeKind::Directory)
    }

    pub fn is_file(&self) -> bool {
        matches!(self.kind, NodeKind::File)
    }

    pub fn is_hidden(&self) -> bool {
        self.path.is_hidden()
    }

    pub fn is_selected(&self) -> bool {
        self.checked || self.children.iter().any(FileNode::is_selected)
    }

    pub fn is_fully_selected(&self) -> bool {
        if !self.checked {
            return false;
        }
        if self.is_directory() {
            self.children.iter().all(FileNode::is_fully_selected)
        } else {
            true
        }
    }

    pub fn lowercase_name(&self) -> String {
        self.path.lowercase_name()
    }

    pub fn collect_checkbox_states_recursive(&self, states: &mut std::collections::HashMap<PathBuf, bool>) {
        states.insert(self.path.clone(), self.checked);
        for child in &self.children {
            child.collect_checkbox_states_recursive(states);
        }
    }

    pub fn restore_checkbox_states_recursive(&mut self, states: &std::collections::HashMap<PathBuf, bool>) {
        if let Some(&checked) = states.get(&self.path) {
            self.checked = checked;
        }
        for child in &mut self.children {
            child.restore_checkbox_states_recursive(states);
        }
    }

    pub fn gather_checked_paths_recursive(&self, out: &mut Vec<String>, query: &str) {
        if !query.is_empty() && !self.matches_search(query) {
            return;
        }

        if self.checked {
            match self.kind {
                NodeKind::File => {
                    out.push(self.path.display().to_string());
                }
                NodeKind::Directory => {
                    for child in &self.children {
                        if query.is_empty() || child.matches_search(query) {
                            child.gather_checked_paths_recursive(out, query);
                        }
                    }
                }
            }
        } else {
            for child in &self.children {
                child.gather_checked_paths_recursive(out, query);
            }
        }
    }

    pub fn expand_all_checked(&mut self, options: &Options) {
        if !self.is_selected() {
            return;
        }

        if self.is_directory() {
            if !self.loaded {
                let _ = self.load_children(options);
            }

            for child in &mut self.children {
                child.expand_all_checked(options);
            }
        }
    }

    pub fn filter_selected(&self, query: &str) -> Option<FileNode> {
        if !query.is_empty() && !self.matches_search(query) {
            return None;
        }

        if !self.is_selected() {
            return None;
        }

        let mut filtered = FileNode::builder()
            .path(self.path.clone())
            .checked(self.checked)
            .loaded(self.loaded)
            .build();

        for child in &self.children {
            if let Some(filtered_child) = child.filter_selected(query) {
                filtered.children.push(filtered_child);
            }
        }

        Some(filtered)
    }
}

#[derive(Default)]
pub struct FileNodeBuilder {
    checked: bool,
    children: Vec<FileNode>,
    kind: Option<NodeKind>,
    loaded: bool,
    path: Option<PathBuf>,
}

impl FileNodeBuilder {
    pub fn checked(mut self, checked: bool) -> Self {
        self.checked = checked;
        self
    }

    pub fn loaded(mut self, loaded: bool) -> Self {
        self.loaded = loaded;
        self
    }

    pub fn path(mut self, path: PathBuf) -> Self {
        self.kind = Some(if path.is_dir() {
            NodeKind::Directory
        } else {
            NodeKind::File
        });

        self.path = Some(path);
        self
    }

    pub fn build(self) -> FileNode {
        let path = self.path.expect("Path is required for FileNode");

        let kind = self.kind.unwrap_or_else(|| {
            if path.is_dir() {
                NodeKind::Directory
            } else {
                NodeKind::File
            }
        });

        FileNode {
            checked: self.checked,
            children: self.children,
            kind,
            loaded: self.loaded,
            path,
        }
    }
}
