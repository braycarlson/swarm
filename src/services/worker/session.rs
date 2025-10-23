use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::sync::mpsc::Sender;

use crate::model::node::FileNode;
use crate::model::options::Options;

use super::core::{Worker, WorkerTask};

pub enum SessionLoadCommand {
    Cancel,
    Load(PathBuf, Options),
}

pub enum SessionLoadResult {
    Error(String),
    Loaded(Vec<FileNode>),
    Loading(String),
}

pub struct SessionLoadTask {
    loading: Arc<AtomicBool>,
}

impl SessionLoadTask {
    fn new() -> Self {
        Self {
            loading: Arc::new(AtomicBool::new(false)),
        }
    }

    fn load_path(path: PathBuf) -> Result<Vec<FileNode>, String> {
        let actual_path = if path.is_file() {
            path.parent()
                .map(|parent| parent.to_path_buf())
                .unwrap_or(path)
        } else {
            path
        };

        let node = FileNode::new(actual_path);
        Ok(vec![node])
    }
}

impl WorkerTask for SessionLoadTask {
    type Command = SessionLoadCommand;
    type Result = SessionLoadResult;

    fn process(&mut self, command: Self::Command, result_sender: &Sender<Self::Result>) {
        match command {
            SessionLoadCommand::Load(path, _options) => {
                self.loading.store(true, Ordering::Relaxed);

                let loading_msg = format!("Loading {}", path.display());
                let _ = result_sender.send(SessionLoadResult::Loading(loading_msg));

                match Self::load_path(path.clone()) {
                    Ok(nodes) => {
                        self.loading.store(false, Ordering::Relaxed);
                        let _ = result_sender.send(SessionLoadResult::Loaded(nodes));
                    }
                    Err(error) => {
                        self.loading.store(false, Ordering::Relaxed);
                        let error_msg = format!("Failed to load path {}: {}", path.display(), error);
                        let _ = result_sender.send(SessionLoadResult::Error(error_msg));
                    }
                }
            }
            SessionLoadCommand::Cancel => {
                self.loading.store(false, Ordering::Relaxed);
            }
        }
    }
}

pub struct SessionLoader {
    loading: Arc<AtomicBool>,
    worker: Worker<SessionLoadTask>,
}

impl Default for SessionLoader {
    fn default() -> Self {
        Self::new()
    }
}

impl SessionLoader {
    pub fn new() -> Self {
        let task = SessionLoadTask::new();
        let loading = Arc::clone(&task.loading);
        let worker = Worker::spawn(task);

        Self { loading, worker }
    }

    pub fn check_results(&self) -> Option<SessionLoadResult> {
        self.worker.try_recv()
    }

    pub fn start_loading(&self, path: PathBuf, options: Options) -> bool {
        if self.loading.load(Ordering::Relaxed) {
            return false;
        }

        let command = SessionLoadCommand::Load(path, options);

        if self.worker.send(command) {
            self.loading.store(true, Ordering::Relaxed);
            true
        } else {
            false
        }
    }
}

impl Clone for SessionLoader {
    fn clone(&self) -> Self {
        Self {
            loading: Arc::clone(&self.loading),
            worker: self.worker.clone(),
        }
    }
}

impl Drop for SessionLoader {
    fn drop(&mut self) {
        let _ = self.worker.send(SessionLoadCommand::Cancel);
    }
}
