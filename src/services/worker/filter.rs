use std::path::PathBuf;
use std::sync::mpsc::SyncSender;

use rayon::iter::{IntoParallelRefIterator as _, ParallelIterator as _};
use rustc_hash::FxHashSet;

use crate::model::node::{FileNode, TREE_DEPTH_MAX};
use crate::model::query::ParsedQuery;
use crate::services::filesystem::git::GitService;
use crate::services::search::SearchScratch;
use crate::services::tree::query::{self, FileTarget, MatchContext};

use super::core::{Worker, WorkerTask};
use super::generation::Generation;

const STACK_CAPACITY: usize = 32;

struct AncestorFrame<'tree> {
    child_index: usize,
    child_matched: bool,
    depth: u32,
    node: &'tree FileNode,
}

pub struct FilterComplete {
    pub generation: u64,
    pub matching: FxHashSet<PathBuf>,
    pub session: String,
}

pub struct FilterEntry {
    name_offset: u32,
    path: PathBuf,
    path_lowercase: Box<str>,
}

impl FilterEntry {
    fn name_lowercase(&self) -> &str {
        debug_assert!(self.name_offset as usize <= self.path_lowercase.len());

        self.path_lowercase
            .get(self.name_offset as usize..)
            .unwrap_or("")
    }

    fn new(node: &FileNode, path_lowercase: &str) -> Self {
        debug_assert!(path_lowercase.ends_with(&*node.name_lowercase));

        let offset = path_lowercase.len() - node.name_lowercase.len();
        let name_offset = u32::try_from(offset).expect("a path is shorter than four gigabytes");

        Self {
            name_offset,
            path: node.path.clone(),
            path_lowercase: Box::from(path_lowercase),
        }
    }
}

pub struct FilterRequest {
    pub entries: Vec<FilterEntry>,
    pub generation: u64,
    pub git: GitService,
    pub query: ParsedQuery,
    pub session: String,
}

struct FilterTask {
    generation: Generation,
}

impl FilterTask {
    fn filter_entries(&self, request: &FilterRequest) -> Option<FxHashSet<PathBuf>> {
        let context = MatchContext::new(&request.query, Some(&request.git));

        let matching: FxHashSet<PathBuf> = request
            .entries
            .par_iter()
            .map_init(SearchScratch::default, |scratch, entry| {
                if !self.generation.is_current(request.generation) {
                    return None;
                }

                let target = FileTarget {
                    name_lowercase: entry.name_lowercase(),
                    path: &entry.path,
                    path_lowercase: &entry.path_lowercase,
                };

                context
                    .file_matches(&target, scratch)
                    .then(|| entry.path.clone())
            })
            .flatten()
            .collect();

        if !self.generation.is_current(request.generation) {
            return None;
        }

        assert!(matching.len() <= request.entries.len());

        Some(matching)
    }
}

impl WorkerTask for FilterTask {
    type Command = Box<FilterRequest>;
    type Result = FilterComplete;

    fn process(&mut self, request: Self::Command, results: &SyncSender<Self::Result>) {
        if !self.generation.is_current(request.generation) {
            return;
        }

        let Some(matching) = self.filter_entries(&request) else {
            return;
        };

        let complete = FilterComplete {
            generation: request.generation,
            matching,
            session: request.session,
        };

        let _ = results.send(complete);
    }
}

pub struct FilterWorker {
    generation: Generation,
    worker: Worker<FilterTask>,
}

impl FilterWorker {
    pub fn cancel(&self) {
        let _ = self.generation.advance();
    }

    pub fn check_results(&self) -> Option<FilterComplete> {
        self.worker.try_recv()
    }

    pub fn is_current(&self, generation: u64) -> bool {
        self.generation.is_current(generation)
    }

    pub fn new() -> Self {
        let generation = Generation::default();

        let task = FilterTask {
            generation: generation.clone(),
        };

        Self {
            generation,
            worker: Worker::spawn("filter", task),
        }
    }

    pub fn next_generation(&self) -> u64 {
        self.generation.advance()
    }

    pub fn start(&self, request: Box<FilterRequest>) -> bool {
        debug_assert!(self.generation.is_current(request.generation));

        self.worker.send(request)
    }
}

impl Default for FilterWorker {
    fn default() -> Self {
        Self::new()
    }
}

pub(crate) fn add_ancestor_directories(
    nodes: &[FileNode],
    matching: &mut FxHashSet<PathBuf>,
    query: &ParsedQuery,
) {
    let mut stack: Vec<AncestorFrame<'_>> = Vec::with_capacity(STACK_CAPACITY);

    for root in nodes {
        stack.push(AncestorFrame {
            child_index: 0,
            child_matched: false,
            depth: 0,
            node: root,
        });

        while !stack.is_empty() {
            assert!(stack.len() <= TREE_DEPTH_MAX as usize);

            let top = stack.len() - 1;
            let frame = &mut stack[top];
            let node = frame.node;

            if let Some(child) = node.children.get(frame.child_index) {
                let depth = frame.depth;

                frame.child_index += 1;

                if query::descends_into(child, query, depth + 1) {
                    stack.push(AncestorFrame {
                        child_index: 0,
                        child_matched: false,
                        depth: depth + 1,
                        node: child,
                    });
                }

                continue;
            }

            let child_matched = frame.child_matched;

            let _ = stack.pop();

            let matched = if node.is_directory() {
                child_matched
            } else {
                matching.contains(&node.path)
            };

            if node.is_directory() {
                if matched {
                    let _ = matching.insert(node.path.clone());
                }
            }

            if let Some(parent) = stack.last_mut() {
                parent.child_matched |= matched;
            }
        }
    }
}

pub fn build_filter_entries(nodes: &[FileNode], query: &ParsedQuery) -> Vec<FilterEntry> {
    let mut entries = Vec::new();

    query::walk_with_path(nodes, |node, depth, path_lowercase| {
        if node.is_directory() {
            return query::descends_into(node, query, depth);
        }

        entries.push(FilterEntry::new(node, path_lowercase));

        false
    });

    entries
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::node::NodeKind;
    use std::path::MAIN_SEPARATOR;

    fn directory(path: &str, children: Vec<FileNode>) -> FileNode {
        let mut node = FileNode::with_kind(PathBuf::from(path), NodeKind::Directory);

        node.children = children;
        node.loaded = true;

        node
    }

    fn file(path: &str) -> FileNode {
        FileNode::with_kind(PathBuf::from(path), NodeKind::File)
    }

    fn tree() -> Vec<FileNode> {
        vec![directory(
            "/Root",
            vec![
                file("/Root/Alpha.rs"),
                directory("/Root/nested", vec![file("/Root/nested/beta.py")]),
            ],
        )]
    }

    #[test]
    fn entries_carry_the_lowercase_path_and_name() {
        let entries = build_filter_entries(&tree(), &ParsedQuery::default());

        assert_eq!(entries.len(), 2);
        assert_eq!(&*entries[0].path_lowercase, format!("/root{MAIN_SEPARATOR}alpha.rs"));
        assert_eq!(entries[0].name_lowercase(), "alpha.rs");
    }

    #[test]
    fn ancestors_of_matches_are_added() {
        let nodes = tree();
        let mut matching = FxHashSet::default();

        let _ = matching.insert(PathBuf::from("/Root/nested/beta.py"));

        add_ancestor_directories(&nodes, &mut matching, &ParsedQuery::default());

        assert!(matching.contains(&PathBuf::from("/Root/nested")));
        assert!(matching.contains(&PathBuf::from("/Root")));
        assert!(!matching.contains(&PathBuf::from("/Root/Alpha.rs")));
    }
}
