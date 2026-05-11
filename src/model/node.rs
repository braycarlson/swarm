use std::path::{Path, PathBuf};
use std::sync::{Arc, OnceLock};

use rayon::prelude::*;
use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};

use crate::app::state::search::{FileMetadata, ParsedQuery, TypeFilter};
use crate::model::options::Options;
use crate::model::path::PathExtensions;
use crate::services::filesystem::git::GitService;
use crate::services::tree::traversal::Traversable;

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
    #[serde(skip)]
    pub metadata: Option<FileMetadata>,
    #[serde(skip)]
    pub offset_name: u32,
    pub path: PathBuf,
    #[serde(skip, default = "empty_lowercase_path")]
    pub lowercase_path: Arc<str>,
}

fn empty_lowercase_path() -> Arc<str> {
    static EMPTY: OnceLock<Arc<str>> = OnceLock::new();
    EMPTY.get_or_init(|| Arc::from("")).clone()
}

fn compute_path_cache(path: &Path) -> (Arc<str>, u32) {
    let lowercase_path = path.to_string_lossy().to_ascii_lowercase();
    let byte_length_name = path.file_name()
        .map(|n| n.to_string_lossy().len())
        .unwrap_or(0);
    let offset_name = (lowercase_path.len() - byte_length_name) as u32;
    (Arc::from(lowercase_path), offset_name)
}

impl FileNode {
    pub fn new(path: PathBuf) -> Self {
        let kind = if path.is_dir() {
            NodeKind::Directory
        } else {
            NodeKind::File
        };

        let (lowercase_path, offset_name) = compute_path_cache(&path);

        Self {
            checked: false,
            children: Vec::new(),
            kind,
            loaded: false,
            metadata: None,
            offset_name,
            path,
            lowercase_path,
        }
    }

    pub fn with_kind(path: PathBuf, kind: NodeKind) -> Self {
        let (lowercase_path, offset_name) = compute_path_cache(&path);

        Self {
            checked: false,
            children: Vec::new(),
            kind,
            loaded: false,
            metadata: None,
            offset_name,
            path,
            lowercase_path,
        }
    }

    pub fn builder() -> FileNodeBuilder {
        FileNodeBuilder::default()
    }

    pub fn file_name(&self) -> Option<String> {
        self.path.file_name_string()
    }

    pub fn name_lowercase(&self) -> &str {
        &self.lowercase_path[self.offset_name as usize..]
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
        self.name_lowercase().to_owned()
    }

    pub fn collect_checkbox_states_recursive(&self, states: &mut FxHashMap<PathBuf, bool>) {
        states.insert(self.path.clone(), self.checked);
        for child in &self.children {
            child.collect_checkbox_states_recursive(states);
        }
    }

    pub fn restore_checkbox_states_recursive(&mut self, states: &FxHashMap<PathBuf, bool>) {
        if let Some(&checked) = states.get(&self.path) {
            self.checked = checked;
        }
        for child in &mut self.children {
            child.restore_checkbox_states_recursive(states);
        }
    }

    pub fn gather_checked_paths_recursive(&self, paths: &mut Vec<String>, query: &str) {
        let parsed = ParsedQuery::parse(query);
        self.gather_checked_paths_with_git(paths, &parsed, None);
    }

    pub fn gather_checked_paths_with_git(&self, paths: &mut Vec<String>, query: &ParsedQuery, git: Option<&GitService>) {
        if !self.matches_query_with_git(query, git) {
            return;
        }

        if self.checked {
            match self.kind {
                NodeKind::File => {
                    paths.push(self.path.to_string_lossy().into_owned());
                }
                NodeKind::Directory => {
                    for child in &self.children {
                        child.gather_checked_paths_with_git(paths, query, git);
                    }
                }
            }
        } else {
            for child in &self.children {
                child.gather_checked_paths_with_git(paths, query, git);
            }
        }
    }

    pub fn matches_query_with_git(&self, query: &ParsedQuery, git: Option<&GitService>) -> bool {
        self.matches_query_recursive(query, git, 0)
    }

    pub(crate) fn matches_query_recursive(&self, query: &ParsedQuery, git: Option<&GitService>, depth: usize) -> bool {
        if query.is_empty() {
            return true;
        }

        if self.is_directory() {
            if query.has_depth_filter() && !query.matches_depth(depth) {
                return false;
            }

            if matches!(query.filter_type, Some(TypeFilter::Directory)) {
                let self_matches = query.matches_full(self.name_lowercase(), &self.lowercase_path, None, true, None);

                let has_matching_children = self.children.iter().any(|child| {
                    child.matches_query_recursive(query, git, depth + 1)
                });

                return self_matches || has_matching_children;
            }

            if query.requires_file_match() {
                let has_matching_children = self.children.iter().any(|child| {
                    child.matches_query_recursive(query, git, depth + 1)
                });
                return has_matching_children;
            }

            let has_matching_children = self.children.iter().any(|child| {
                child.matches_query_recursive(query, git, depth + 1)
            });

            if has_matching_children {
                return true;
            }

            return query.matches_full(self.name_lowercase(), &self.lowercase_path, None, true, None);
        }

        let git_status = git.map(|g| g.get_status(&self.path));
        let metadata = self.get_metadata_for_query(query);

        if !query.matches_full(self.name_lowercase(), &self.lowercase_path, git_status, false, metadata.as_ref()) {
            return false;
        }

        if !query.patterns_content.is_empty() {
            let matchers = crate::services::search::build_content_matchers(&query.patterns_content);

            if !crate::services::search::content_matches(&self.path, &matchers) {
                return false;
            }
        }

        if !query.patterns_symbol.is_empty()
            && !crate::services::search::symbol_matches(&self.path, &query.patterns_symbol)
        {
            return false;
        }

        true
    }

    fn get_metadata_for_query(&self, query: &ParsedQuery) -> Option<FileMetadata> {
        if !query.needs_metadata() {
            return None;
        }

        if let Some(ref cached) = self.metadata {
            if query.needs_content() && cached.content.is_none() {
                return FileMetadata::from_path(&self.path, true);
            }

            if query.needs_lines() && cached.lines.is_none() {
                return FileMetadata::from_path_with_lines(&self.path);
            }

            return Some(cached.clone());
        }

        if query.needs_content() {
            return FileMetadata::from_path(&self.path, true);
        }

        if query.needs_lines() {
            return FileMetadata::from_path_with_lines(&self.path);
        }

        FileMetadata::from_path_basic(&self.path)
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
        let parsed = ParsedQuery::parse(query);
        self.filter_selected_with_git(&parsed, None)
    }

    pub fn filter_selected_with_git(&self, query: &ParsedQuery, git: Option<&GitService>) -> Option<FileNode> {
        if !self.matches_query_with_git(query, git) {
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
            if let Some(filtered_child) = child.filter_selected_with_git(query, git) {
                filtered.children.push(filtered_child);
            }
        }

        Some(filtered)
    }

    pub fn load_metadata(&mut self, include_lines: bool, include_content: bool) {
        if self.is_directory() {
            return;
        }

        if include_content {
            self.metadata = FileMetadata::from_path(&self.path, true);
        } else if include_lines {
            self.metadata = FileMetadata::from_path_with_lines(&self.path);
        } else {
            self.metadata = FileMetadata::from_path_basic(&self.path);
        }
    }

    pub fn load_metadata_recursive(&mut self, include_lines: bool, include_content: bool) {
        self.load_metadata(include_lines, include_content);

        for child in &mut self.children {
            child.load_metadata_recursive(include_lines, include_content);
        }
    }

    pub fn recompute_cache(&mut self) {
        let (lowercase_path, offset_name) = compute_path_cache(&self.path);
        self.lowercase_path = lowercase_path;
        self.offset_name = offset_name;

        self.children.par_iter_mut().for_each(|child| {
            child.recompute_cache();
        });
    }
}

#[derive(Default)]
pub struct FileNodeBuilder {
    checked: bool,
    children: Vec<FileNode>,
    kind: Option<NodeKind>,
    loaded: bool,
    metadata: Option<FileMetadata>,
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

    pub fn metadata(mut self, metadata: Option<FileMetadata>) -> Self {
        self.metadata = metadata;
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

        let (lowercase_path, offset_name) = compute_path_cache(&path);

        FileNode {
            checked: self.checked,
            children: self.children,
            kind,
            loaded: self.loaded,
            metadata: self.metadata,
            offset_name,
            path,
            lowercase_path,
        }
    }
}
