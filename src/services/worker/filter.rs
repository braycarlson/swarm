use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::Sender;
use std::sync::Arc;

use rayon::prelude::*;
use rustc_hash::FxHashSet;

use crate::app::state::search::{FileMetadata, ParsedQuery};
use crate::model::node::FileNode;
use crate::services::filesystem::git::GitService;
use crate::services::search::{build_content_matchers, content_matches, symbol_matches};

use super::core::{Worker, WorkerTask};

pub struct FilterEntry {
    pub path: PathBuf,
    pub path_lower: Arc<str>,
    pub name_offset: u32,
    pub metadata: Option<FileMetadata>,
}

impl FilterEntry {
    pub fn name_lower(&self) -> &str {
        &self.path_lower[self.name_offset as usize..]
    }
}

pub enum FilterCommand {
    Filter {
        entries: Vec<FilterEntry>,
        query: ParsedQuery,
        git: GitService,
    },
    Cancel,
}

pub enum FilterResult {
    Started,
    Progress(usize, usize),
    Complete(FxHashSet<PathBuf>),
    Cancelled,
}

pub struct FilterTask {
    is_running: Arc<AtomicBool>,
}

impl FilterTask {
    fn new() -> Self {
        Self {
            is_running: Arc::new(AtomicBool::new(false)),
        }
    }

    fn filter_entries(
        entries: &[FilterEntry],
        query: &ParsedQuery,
        git: &GitService,
        result_sender: &Sender<FilterResult>,
        is_running: &Arc<AtomicBool>,
    ) -> FxHashSet<PathBuf> {
        let total = entries.len();
        let _ = result_sender.send(FilterResult::Progress(0, total));

        let content_matchers = build_content_matchers(&query.content_patterns);

        let matching_files: FxHashSet<PathBuf> = entries
            .par_iter()
            .filter(|entry| {
                if !is_running.load(Ordering::Relaxed) {
                    return false;
                }

                Self::entry_matches(entry, query, git, &content_matchers)
            })
            .map(|entry| entry.path.clone())
            .collect();

        if !is_running.load(Ordering::Relaxed) {
            return FxHashSet::default();
        }

        let _ = result_sender.send(FilterResult::Progress(total, total));

        matching_files
    }

    fn entry_matches(
        entry: &FilterEntry,
        query: &ParsedQuery,
        git: &GitService,
        content_matchers: &[grep_regex::RegexMatcher],
    ) -> bool {
        let git_status = Some(git.get_status(&entry.path));
        let metadata = Self::resolve_metadata(entry, query);

        if !query.matches_full(entry.name_lower(), &entry.path_lower, git_status, false, metadata.as_ref()) {
            return false;
        }

        if !content_matchers.is_empty() && !content_matches(&entry.path, content_matchers) {
            return false;
        }

        if !query.symbol_patterns.is_empty()
            && !symbol_matches(&entry.path, &query.symbol_patterns)
        {
            return false;
        }

        true
    }

    fn resolve_metadata(entry: &FilterEntry, query: &ParsedQuery) -> Option<FileMetadata> {
        if !query.needs_metadata() {
            return None;
        }

        if let Some(ref cached) = entry.metadata {
            if query.needs_content() && cached.content.is_none() {
                return FileMetadata::from_path(&entry.path, true);
            }

            if query.needs_lines() && cached.lines.is_none() {
                return FileMetadata::from_path_with_lines(&entry.path);
            }

            return Some(cached.clone());
        }

        if query.needs_content() {
            return FileMetadata::from_path(&entry.path, true);
        }

        if query.needs_lines() {
            return FileMetadata::from_path_with_lines(&entry.path);
        }

        FileMetadata::from_path_basic(&entry.path)
    }
}

impl WorkerTask for FilterTask {
    type Command = FilterCommand;
    type Result = FilterResult;

    fn process(&mut self, command: Self::Command, result_sender: &Sender<Self::Result>) {
        match command {
            FilterCommand::Filter { entries, query, git } => {
                self.is_running.store(true, Ordering::Relaxed);
                let _ = result_sender.send(FilterResult::Started);

                let matching = Self::filter_entries(
                    &entries,
                    &query,
                    &git,
                    result_sender,
                    &self.is_running,
                );

                if self.is_running.load(Ordering::Relaxed) {
                    self.is_running.store(false, Ordering::Relaxed);
                    let _ = result_sender.send(FilterResult::Complete(matching));
                } else {
                    let _ = result_sender.send(FilterResult::Cancelled);
                }
            }
            FilterCommand::Cancel => {
                self.is_running.store(false, Ordering::Relaxed);
            }
        }
    }
}

pub struct FilterWorker {
    is_running: Arc<AtomicBool>,
    worker: Worker<FilterTask>,
}

impl Default for FilterWorker {
    fn default() -> Self {
        Self::new()
    }
}

impl FilterWorker {
    pub fn new() -> Self {
        let task = FilterTask::new();
        let is_running = Arc::clone(&task.is_running);
        let worker = Worker::spawn(task);

        Self { is_running, worker }
    }

    pub fn start_filter(&self, entries: Vec<FilterEntry>, query: ParsedQuery, git: GitService) -> bool {
        if self.is_running.load(Ordering::Relaxed) {
            let _ = self.worker.send(FilterCommand::Cancel);
        }

        self.worker.send(FilterCommand::Filter { entries, query, git })
    }

    pub fn cancel(&self) {
        let _ = self.worker.send(FilterCommand::Cancel);
    }

    pub fn check_results(&self) -> Option<FilterResult> {
        self.worker.try_recv()
    }

    pub fn is_running(&self) -> bool {
        self.is_running.load(Ordering::Relaxed)
    }
}

impl Clone for FilterWorker {
    fn clone(&self) -> Self {
        Self {
            is_running: Arc::clone(&self.is_running),
            worker: self.worker.clone(),
        }
    }
}

pub fn build_filter_entries(nodes: &[FileNode], query: &ParsedQuery) -> Vec<FilterEntry> {
    let mut entries = Vec::new();
    collect_entries(nodes, query, 0, &mut entries);
    entries
}

fn collect_entries(
    nodes: &[FileNode],
    query: &ParsedQuery,
    depth: usize,
    out: &mut Vec<FilterEntry>,
) {
    for node in nodes {
        if node.is_directory() {
            if query.has_depth_filter() && !query.matches_depth(depth) {
                continue;
            }

            collect_entries(&node.children, query, depth + 1, out);
        } else {
            out.push(FilterEntry {
                path: node.path.clone(),
                path_lower: Arc::clone(&node.path_lower),
                name_offset: node.name_offset,
                metadata: node.metadata.clone(),
            });
        }
    }
}

pub fn add_ancestor_directories(
    nodes: &[FileNode],
    matching: &mut FxHashSet<PathBuf>,
    query: &ParsedQuery,
) {
    add_ancestors_recursive(nodes, matching, 0, query);
}

fn add_ancestors_recursive(
    nodes: &[FileNode],
    matching: &mut FxHashSet<PathBuf>,
    depth: usize,
    query: &ParsedQuery,
) -> bool {
    let mut any_match = false;

    for node in nodes {
        if node.is_directory() {
            if query.has_depth_filter() && !query.matches_depth(depth) {
                continue;
            }

            let child_matches = add_ancestors_recursive(
                &node.children,
                matching,
                depth + 1,
                query,
            );

            if child_matches {
                matching.insert(node.path.clone());
                any_match = true;
            }
        } else if matching.contains(&node.path) {
            any_match = true;
        }
    }

    any_match
}
