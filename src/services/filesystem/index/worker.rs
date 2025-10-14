use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::sync::mpsc::{Receiver, Sender};

use crate::model::options::Options;

use super::builder::IndexBuilder;
use super::cleaner::IndexCleaner;
use super::command::IndexCommand;
use super::file::IndexFile;
use super::finalizer::IndexFinalizer;
use super::progress::ProgressReporter;
use super::result::IndexResult;
use super::session::SessionIndexData;
use super::switcher::SessionSwitcher;
use super::statistics::IndexStatistics;

pub struct IndexWorker {
    command_receiver: Receiver<IndexCommand>,
    result_sender: Sender<IndexResult>,
    index: Arc<Mutex<HashMap<PathBuf, IndexFile>>>,
    stats: Arc<Mutex<IndexStatistics>>,
    is_running: Arc<AtomicBool>,
    is_paused: Arc<AtomicBool>,
    session_data: Arc<Mutex<HashMap<String, SessionIndexData>>>,
    active_session_id: Arc<Mutex<Option<String>>>,
}

impl IndexWorker {
    pub fn new(
        command_receiver: Receiver<IndexCommand>,
        result_sender: Sender<IndexResult>,
        index: Arc<Mutex<HashMap<PathBuf, IndexFile>>>,
        stats: Arc<Mutex<IndexStatistics>>,
        is_running: Arc<AtomicBool>,
        is_paused: Arc<AtomicBool>,
        session_data: Arc<Mutex<HashMap<String, SessionIndexData>>>,
        active_session_id: Arc<Mutex<Option<String>>>,
    ) -> Self {
        Self {
            command_receiver,
            result_sender,
            index,
            stats,
            is_running,
            is_paused,
            session_data,
            active_session_id,
        }
    }

    pub fn run(self) {
        while let Ok(command) = self.command_receiver.recv() {
            self.handle_command(command);
        }
    }

    fn handle_command(&self, command: IndexCommand) {
        match command {
            IndexCommand::Start(paths, options) => {
                self.handle_start(paths, options);
            }
            IndexCommand::Stop => {
                self.handle_stop();
            }
            IndexCommand::Pause => {
                self.handle_pause();
            }
            IndexCommand::Resume => {
                self.handle_resume();
            }
            IndexCommand::SwitchSession(session_id) => {
                self.handle_session_switch(session_id);
            }
        }
    }

    fn handle_start(&self, paths: Vec<PathBuf>, options: Options) {
        if self.is_running.load(Ordering::Relaxed) {
            let _ = self.result_sender.send(IndexResult::Error("Indexing already in progress".into()));
            return;
        }

        self.is_running.store(true, Ordering::Relaxed);
        self.is_paused.store(false, Ordering::Relaxed);

        IndexCleaner::clear(&self.index, &self.stats, &self.active_session_id);
        ProgressReporter::start(&self.stats, &self.result_sender, &self.is_running, &self.is_paused);

        let result = IndexBuilder::build(
            paths,
            options,
            Arc::clone(&self.index),
            Arc::clone(&self.stats),
            Arc::clone(&self.is_running),
            Arc::clone(&self.is_paused),
            Arc::clone(&self.active_session_id),
            self.result_sender.clone(),
        );

        match result {
            Ok(count) => {
                IndexFinalizer::finalize(
                    &self.stats,
                    &self.index,
                    &self.session_data,
                    &self.active_session_id,
                    &self.result_sender,
                    count,
                );
            }
            Err(error) => {
                let _ = self.result_sender.send(IndexResult::Error(format!("Indexing error: {}", error)));
            }
        }

        self.is_running.store(false, Ordering::SeqCst);
    }

    fn handle_stop(&self) {
        self.is_running.store(false, Ordering::Relaxed);
        self.is_paused.store(false, Ordering::Relaxed);
    }

    fn handle_pause(&self) {
        self.is_paused.store(true, Ordering::Relaxed);
    }

    fn handle_resume(&self) {
        self.is_paused.store(false, Ordering::Relaxed);
    }

    fn handle_session_switch(&self, session_id: String) {
        SessionSwitcher::switch(
            session_id,
            &self.index,
            &self.stats,
            &self.session_data,
            &self.active_session_id,
            &self.result_sender,
            self.is_running.load(Ordering::Relaxed),
        );
    }
}
