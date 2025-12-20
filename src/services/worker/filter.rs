use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::Sender;
use std::sync::Arc;

use crate::app::state::search::{FileMetadata, ParsedQuery};
use crate::model::node::FileNode;
use crate::services::filesystem::git::GitService;

use super::core::{Worker, WorkerTask};

pub enum FilterCommand {
    Filter {
        nodes: Vec<FileNode>,
        query: ParsedQuery,
        git: GitService,
    },
    Cancel,
}

pub enum FilterResult {
    Started,
    Progress(usize, usize),
    Complete(HashSet<PathBuf>),
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

    fn filter_nodes(
        nodes: &[FileNode],
        query: &ParsedQuery,
        git: &GitService,
        result_sender: &Sender<FilterResult>,
        is_running: &Arc<AtomicBool>,
    ) -> HashSet<PathBuf> {
        let mut matching_paths = HashSet::new();
        let mut processed: usize = 0;
        let total = Self::count_files(nodes);

        Self::filter_recursive(
            nodes,
            query,
            git,
            0,
            &mut matching_paths,
            result_sender,
            &mut processed,
            total,
            is_running,
        );

        matching_paths
    }

    fn filter_recursive(
        nodes: &[FileNode],
        query: &ParsedQuery,
        git: &GitService,
        depth: usize,
        matching_paths: &mut HashSet<PathBuf>,
        result_sender: &Sender<FilterResult>,
        processed: &mut usize,
        total: usize,
        is_running: &Arc<AtomicBool>,
    ) {
        for node in nodes {
            if !is_running.load(Ordering::Relaxed) {
                return;
            }

            if node.is_directory() {
                if query.has_depth_filter() && !query.matches_depth(depth) {
                    continue;
                }

                Self::filter_recursive(
                    &node.children,
                    query,
                    git,
                    depth + 1,
                    matching_paths,
                    result_sender,
                    processed,
                    total,
                    is_running,
                );

                let has_matching_child = node.children.iter().any(|c| matching_paths.contains(&c.path));

                if has_matching_child {
                    matching_paths.insert(node.path.clone());
                }
            } else {
                *processed += 1;

                if *processed % 100 == 0 {
                    let _ = result_sender.send(FilterResult::Progress(*processed, total));
                }

                if Self::file_matches(node, query, git) {
                    matching_paths.insert(node.path.clone());
                }
            }
        }
    }

    fn file_matches(node: &FileNode, query: &ParsedQuery, git: &GitService) -> bool {
        let name = node.file_name().unwrap_or_default();
        let path = node.path.to_string_lossy();
        let git_status = Some(git.get_status(&node.path));

        let metadata = Self::get_metadata(node, query);

        query.matches_full(&name, &path, git_status, false, metadata.as_ref())
    }

    fn get_metadata(node: &FileNode, query: &ParsedQuery) -> Option<FileMetadata> {
        if !query.needs_metadata() {
            return None;
        }

        if let Some(ref cached) = node.metadata {
            if query.needs_content() && cached.content.is_none() {
                return FileMetadata::from_path(&node.path, true);
            }

            if query.needs_lines() && cached.lines.is_none() {
                return FileMetadata::from_path_with_lines(&node.path);
            }

            return Some(cached.clone());
        }

        if query.needs_content() {
            return FileMetadata::from_path(&node.path, true);
        }

        if query.needs_lines() {
            return FileMetadata::from_path_with_lines(&node.path);
        }

        FileMetadata::from_path_basic(&node.path)
    }

    fn count_files(nodes: &[FileNode]) -> usize {
        let mut count: usize = 0;

        for node in nodes {
            if node.is_file() {
                count += 1;
            } else {
                count += Self::count_files(&node.children);
            }
        }

        count
    }
}

impl WorkerTask for FilterTask {
    type Command = FilterCommand;
    type Result = FilterResult;

    fn process(&mut self, command: Self::Command, result_sender: &Sender<Self::Result>) {
        match command {
            FilterCommand::Filter { nodes, query, git } => {
                self.is_running.store(true, Ordering::Relaxed);
                let _ = result_sender.send(FilterResult::Started);

                let matching = Self::filter_nodes(
                    &nodes,
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

    pub fn start_filter(&self, nodes: Vec<FileNode>, query: ParsedQuery, git: GitService) -> bool {
        if self.is_running.load(Ordering::Relaxed) {
            let _ = self.worker.send(FilterCommand::Cancel);
        }

        self.worker.send(FilterCommand::Filter { nodes, query, git })
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
