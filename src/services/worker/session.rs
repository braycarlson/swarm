use std::path::PathBuf;
use std::sync::mpsc::SyncSender;

use rustc_hash::FxHashSet;

use crate::constants::IPC_PATH_COUNT_MAX;
use crate::model::node::FileNode;
use crate::model::path;

use super::core::{Worker, WorkerTask};

pub struct SessionLoadRequest {
    pub paths: Vec<PathBuf>,
    pub session: String,
}

pub struct SessionLoadResult {
    pub missing: Vec<PathBuf>,
    pub nodes: Vec<FileNode>,
    pub session: String,
}

struct SessionLoadTask;

impl SessionLoadTask {
    fn roots_for(paths: Vec<PathBuf>) -> (Vec<FileNode>, Vec<PathBuf>) {
        assert!(paths.len() <= IPC_PATH_COUNT_MAX as usize);

        let mut nodes = Vec::with_capacity(paths.len());
        let mut missing = Vec::new();
        let mut seen = FxHashSet::default();

        for requested in paths {
            if !requested.exists() {
                missing.push(requested);

                continue;
            }

            let directory = path::directory_of(&requested);

            if seen.insert(directory.clone()) {
                nodes.push(FileNode::new(directory));
            }
        }

        debug_assert!(nodes.len() <= IPC_PATH_COUNT_MAX as usize);

        (nodes, missing)
    }
}

impl WorkerTask for SessionLoadTask {
    type Command = SessionLoadRequest;
    type Result = SessionLoadResult;

    fn process(&mut self, request: Self::Command, results: &SyncSender<Self::Result>) {
        let (nodes, missing) = Self::roots_for(request.paths);

        let _ = results.send(SessionLoadResult {
            missing,
            nodes,
            session: request.session,
        });
    }
}

pub struct SessionLoader {
    worker: Worker<SessionLoadTask>,
}

impl SessionLoader {
    pub fn check_results(&self) -> Option<SessionLoadResult> {
        self.worker.try_recv()
    }

    pub fn new() -> Self {
        Self {
            worker: Worker::spawn("session-loader", SessionLoadTask),
        }
    }

    pub fn start(&self, request: SessionLoadRequest) -> bool {
        assert_ne!(request.paths.len(), 0);
        assert_ne!(request.session, "");

        self.worker.send(request)
    }
}

impl Default for SessionLoader {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_paths_are_reported_and_duplicates_collapse() {
        let directory = std::env::temp_dir();
        let missing = PathBuf::from("/nonexistent/swarm/session");

        let (nodes, absent) =
            SessionLoadTask::roots_for(vec![directory.clone(), missing.clone(), directory.clone()]);

        assert_eq!(nodes.len(), 1);
        assert_eq!(nodes[0].path, directory);
        assert_eq!(absent, vec![missing]);
    }
}
