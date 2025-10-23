use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::sync::mpsc::Sender;

use crate::model::node::FileNode;
use crate::model::options::Options;
use crate::services::tree::traversal::Traversable;

use super::core::{Worker, WorkerTask};

pub enum BackgroundLoadCommand {
    Start(Vec<FileNode>, Options),
    Stop,
}

pub enum BackgroundLoadResult {
    NodesUpdated(Vec<FileNode>),
    Progress(usize, usize),
}

pub struct BackgroundLoadTask {
    is_running: Arc<AtomicBool>,
}

impl BackgroundLoadTask {
    fn new() -> Self {
        Self {
            is_running: Arc::new(AtomicBool::new(false)),
        }
    }

    fn load_recursively(
        nodes: &mut [FileNode],
        options: &Options,
        result_sender: &Sender<BackgroundLoadResult>,
        loaded: &mut usize,
        total: usize,
    ) {
        for node in nodes {
            if !node.is_directory() {
                continue;
            }

            if node.loaded {
                Self::load_recursively(&mut node.children, options, result_sender, loaded, total);
                continue;
            }

            if node.load_children(options).is_ok() {
                *loaded += 1;

                if (*loaded).is_multiple_of(100) {
                    let _ = result_sender.send(BackgroundLoadResult::Progress(*loaded, total));
                }
            }

            Self::load_recursively(&mut node.children, options, result_sender, loaded, total);
        }
    }

    fn count_unloaded(nodes: &[FileNode]) -> usize {
        let mut count: usize = 0;

        for node in nodes {
            if node.is_directory() {
                if !node.loaded {
                    count += 1;
                }

                if node.loaded {
                    count += Self::count_unloaded(&node.children);
                }
            }
        }

        count
    }
}

impl WorkerTask for BackgroundLoadTask {
    type Command = BackgroundLoadCommand;
    type Result = BackgroundLoadResult;

    fn process(&mut self, command: Self::Command, result_sender: &Sender<Self::Result>) {
        match command {
            BackgroundLoadCommand::Start(mut nodes, options) => {
                self.is_running.store(true, Ordering::Relaxed);

                let total = Self::count_unloaded(&nodes);
                let mut loaded: usize = 0;

                Self::load_recursively(&mut nodes, &options, result_sender, &mut loaded, total);

                self.is_running.store(false, Ordering::Relaxed);
                let _ = result_sender.send(BackgroundLoadResult::NodesUpdated(nodes));
            }
            BackgroundLoadCommand::Stop => {
                self.is_running.store(false, Ordering::Relaxed);
            }
        }
    }
}

pub struct BackgroundLoader {
    is_running: Arc<AtomicBool>,
    worker: Worker<BackgroundLoadTask>,
}

impl Default for BackgroundLoader {
    fn default() -> Self {
        Self::new()
    }
}

impl BackgroundLoader {
    pub fn new() -> Self {
        let task = BackgroundLoadTask::new();
        let is_running = Arc::clone(&task.is_running);
        let worker = Worker::spawn(task);

        Self {
            is_running,
            worker,
        }
    }

    pub fn check_results(&self) -> Option<BackgroundLoadResult> {
        self.worker.try_recv()
    }

    pub fn is_running(&self) -> bool {
        self.is_running.load(Ordering::Relaxed)
    }

    pub fn start_loading(&self, nodes: Vec<FileNode>, options: Options) -> bool {
        if self.is_running() {
            return false;
        }

        self.worker.send(BackgroundLoadCommand::Start(nodes, options))
    }

    pub fn stop(&self) {
        let _ = self.worker.send(BackgroundLoadCommand::Stop);
    }
}

impl Clone for BackgroundLoader {
    fn clone(&self) -> Self {
        Self {
            is_running: Arc::clone(&self.is_running),
            worker: self.worker.clone(),
        }
    }
}

impl Drop for BackgroundLoader {
    fn drop(&mut self) {
        self.stop();
    }
}
