use alloc::sync::Arc;
use std::sync::mpsc::SyncSender;

use crate::model::node::FileNode;
use crate::services::filesystem::filter::GlobPathFilter;
use crate::services::tree::loader::{self, LoadSummary};

use super::core::{Worker, WorkerTask};
use super::generation::Generation;

pub struct BackgroundOutcome {
    pub generation: u64,
    pub nodes: Vec<FileNode>,
    pub session: String,
    pub summary: LoadSummary,
}

pub struct BackgroundRequest {
    pub filter: Arc<GlobPathFilter>,
    pub generation: u64,
    pub nodes: Vec<FileNode>,
    pub session: String,
}

struct BackgroundTask {
    generation: Generation,
}

impl WorkerTask for BackgroundTask {
    type Command = Box<BackgroundRequest>;
    type Result = Box<BackgroundOutcome>;

    fn process(&mut self, request: Self::Command, results: &SyncSender<Self::Result>) {
        let BackgroundRequest {
            filter,
            generation,
            mut nodes,
            session,
        } = *request;

        if !self.generation.is_current(generation) {
            return;
        }

        let summary = loader::load_tree(&mut nodes, &filter, |_| {
            self.generation.is_current(generation)
        });

        if !self.generation.is_current(generation) {
            return;
        }

        let outcome = BackgroundOutcome {
            generation,
            nodes,
            session,
            summary,
        };

        let _ = results.send(Box::new(outcome));
    }
}

pub struct BackgroundLoader {
    generation: Generation,
    worker: Worker<BackgroundTask>,
}

impl BackgroundLoader {
    pub fn check_results(&self) -> Option<Box<BackgroundOutcome>> {
        self.worker.try_recv()
    }

    pub fn is_current(&self, generation: u64) -> bool {
        self.generation.is_current(generation)
    }

    pub fn new() -> Self {
        let generation = Generation::default();

        let task = BackgroundTask {
            generation: generation.clone(),
        };

        Self {
            generation,
            worker: Worker::spawn("background-loader", task),
        }
    }

    pub fn start(
        &self,
        nodes: Vec<FileNode>,
        filter: Arc<GlobPathFilter>,
        session: String,
    ) -> bool {
        assert_ne!(session, "");

        let request = BackgroundRequest {
            filter,
            generation: self.generation.advance(),
            nodes,
            session,
        };

        self.worker.send(Box::new(request))
    }

    pub fn stop(&self) {
        let _ = self.generation.advance();
    }
}

impl Default for BackgroundLoader {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for BackgroundLoader {
    fn drop(&mut self) {
        self.stop();
    }
}
