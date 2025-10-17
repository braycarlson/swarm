use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;

use crate::model::error::SwarmResult;
use crate::model::options::Options;

use super::command::IndexCommand;
use super::file::IndexFile;
use super::result::IndexResult;
use super::session::SessionIndexData;
use super::statistics::IndexStatistics;
use super::worker::IndexWorker;

pub struct IndexService {
    active_session_id: Arc<Mutex<Option<String>>>,
    index: Arc<Mutex<HashMap<PathBuf, IndexFile>>>,
    is_paused: Arc<AtomicBool>,
    is_running: Arc<AtomicBool>,
    receiver: Arc<Mutex<Receiver<IndexResult>>>,
    sender: Sender<IndexCommand>,
    session_data: Arc<Mutex<HashMap<String, SessionIndexData>>>,
    stats: Arc<Mutex<IndexStatistics>>,
}

impl IndexService {
    pub fn new() -> Self {
        let (command_sender, command_receiver) = mpsc::channel();
        let (result_sender, result_receiver) = mpsc::channel();

        let active_session_id = Arc::new(Mutex::new(None));
        let index = Arc::new(Mutex::new(HashMap::new()));
        let is_paused = Arc::new(AtomicBool::new(false));
        let is_running = Arc::new(AtomicBool::new(false));
        let receiver = Arc::new(Mutex::new(result_receiver));
        let session_data = Arc::new(Mutex::new(HashMap::new()));
        let stats = Arc::new(Mutex::new(IndexStatistics::default()));

        let worker = IndexWorker::new(
            command_receiver,
            result_sender,
            Arc::clone(&index),
            Arc::clone(&stats),
            Arc::clone(&is_running),
            Arc::clone(&is_paused),
            Arc::clone(&session_data),
            Arc::clone(&active_session_id),
        );

        thread::spawn(move || {
            worker.run();
        });

        Self {
            active_session_id,
            index,
            is_paused,
            is_running,
            receiver,
            sender: command_sender,
            session_data,
            stats,
        }
    }

    pub fn get_session_statistics(&self, session_id: &str) -> Option<IndexStatistics> {
        self.session_data
            .lock()
            .ok()
            .and_then(|data| data.get(session_id).map(|session| session.stats.clone()))
    }

    pub fn get_session_extensions(&self, session_id: &str) -> HashSet<String> {
        self.session_data
            .lock()
            .ok()
            .and_then(|data| data.get(session_id).map(|session| session.extensions.clone()))
            .unwrap_or_default()
    }

    pub fn check_results(&self) -> Option<IndexResult> {
        self.receiver
            .lock()
            .ok()?
            .try_recv()
            .ok()
    }

    pub fn filter_by_extension(&self, extension: &str) -> Vec<IndexFile> {
        let Ok(index_lock) = self.index.lock() else {
            return vec![];
        };

        let extension_lower = extension.to_lowercase();

        index_lock
            .values()
            .filter(|file| {
                file.extension
                    .as_ref()
                    .map_or(false, |ext| ext.to_lowercase() == extension_lower)
            })
            .cloned()
            .collect()
    }

    pub fn get_all_extensions(&self) -> HashSet<String> {
        if let Ok(active_id) = self.active_session_id.lock() {
            if let Some(id) = active_id.as_ref() {
                if let Ok(session_data) = self.session_data.lock() {
                    if let Some(data) = session_data.get(id) {
                        return data.extensions.clone();
                    }
                }
            }
        }

        let Ok(index_lock) = self.index.lock() else {
            return HashSet::new();
        };

        index_lock
            .values()
            .filter_map(|file| file.extension.clone())
            .collect()
    }

    pub fn get_indexed_count(&self) -> usize {
        self.index
            .lock()
            .map(|lock| lock.len())
            .unwrap_or(0)
    }

    pub fn get_statistics(&self) -> SwarmResult<IndexStatistics> {
        self.stats
            .lock()
            .map(|stats| stats.clone())
            .map_err(|_| crate::model::error::SwarmError::Other("Failed to lock statistics".into()))
    }

    pub fn get_statistics_if_active(&self) -> Option<IndexStatistics> {
        if self.is_running.load(Ordering::Relaxed) {
            return self.stats.lock().ok().map(|stats| stats.clone());
        }

        let active_id = self.active_session_id.lock().ok()?;
        let id = active_id.as_ref()?;

        let session_data = self.session_data.lock().ok()?;
        let data = session_data.get(id)?;

        if data.stats.is_complete {
            Some(data.stats.clone())
        } else {
            None
        }
    }

    pub fn has_indexed_session(&self, session_id: &str) -> bool {
        self.session_data
            .lock()
            .ok()
            .and_then(|data| data.get(session_id).cloned())
            .map_or(false, |data| data.stats.is_complete && !data.index.is_empty())
    }

    pub fn pause_indexing(&self) -> bool {
        if !self.is_running.load(Ordering::Relaxed) || self.is_paused.load(Ordering::Relaxed) {
            return false;
        }

        self.sender.send(IndexCommand::Pause).is_ok()
    }

    pub fn resume_indexing(&self) -> bool {
        if !self.is_running.load(Ordering::Relaxed) || !self.is_paused.load(Ordering::Relaxed) {
            return false;
        }

        self.sender.send(IndexCommand::Resume).is_ok()
    }

    pub fn search_files(&self, query: &str, max_results: usize) -> Vec<IndexFile> {
        let Ok(index_lock) = self.index.lock() else {
            return vec![];
        };

        let query_lower = query.to_lowercase();

        let mut results: Vec<_> = index_lock
            .values()
            .filter(|file| {
                file.name.to_lowercase().contains(&query_lower)
                    || file.path.to_string_lossy().to_lowercase().contains(&query_lower)
            })
            .cloned()
            .collect();

        results.sort_by(|first, second| {
            let first_exact = first.name.to_lowercase() == query_lower;
            let second_exact = second.name.to_lowercase() == query_lower;

            match (first_exact, second_exact) {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                _ => first.name.cmp(&second.name),
            }
        });

        results.truncate(max_results);
        results
    }

    pub fn start_indexing(&self, paths: Vec<PathBuf>, options: Options) -> bool {
        if self.is_running.load(Ordering::Relaxed) {
            return false;
        }

        self.sender.send(IndexCommand::Start(paths, options)).is_ok()
    }

    pub fn stop_indexing(&self) -> bool {
        if !self.is_running.load(Ordering::Relaxed) {
            return false;
        }

        self.sender.send(IndexCommand::Stop).is_ok()
    }

    pub fn switch_session(&self, session_id: String) -> bool {
        if self.sender.send(IndexCommand::SwitchSession(session_id.clone())).is_ok() {
            if let Ok(mut active_id) = self.active_session_id.lock() {
                *active_id = Some(session_id);
            }

            true
        } else {
            false
        }
    }

    pub fn update_stats(&self, stats: IndexStatistics) {
        if let Ok(mut stats_lock) = self.stats.lock() {
            let session_id = stats_lock.session_id.clone();
            *stats_lock = stats;

            if session_id.is_some() {
                stats_lock.session_id = session_id;
            }
        }
    }
}

impl Clone for IndexService {
    fn clone(&self) -> Self {
        Self {
            active_session_id: Arc::clone(&self.active_session_id),
            index: Arc::clone(&self.index),
            is_paused: Arc::clone(&self.is_paused),
            is_running: Arc::clone(&self.is_running),
            receiver: Arc::clone(&self.receiver),
            sender: self.sender.clone(),
            session_data: Arc::clone(&self.session_data),
            stats: Arc::clone(&self.stats),
        }
    }
}

impl Default for IndexService {
    fn default() -> Self {
        Self::new()
    }
}
