use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::Sender;

use super::file::IndexFile;
use super::result::IndexResult;
use super::session::SessionIndexData;
use super::statistics::IndexStatistics;

pub struct SessionSwitcher;

impl SessionSwitcher {
    pub fn switch(
        session_id: String,
        index: &Arc<Mutex<HashMap<PathBuf, IndexFile>>>,
        stats: &Arc<Mutex<IndexStatistics>>,
        session_data: &Arc<Mutex<HashMap<String, SessionIndexData>>>,
        active_session_id: &Arc<Mutex<Option<String>>>,
        result_sender: &Sender<IndexResult>,
        is_running: bool,
    ) {
        if let Some(previous_session_id) = active_session_id
            .lock()
            .ok()
            .and_then(|lock| lock.clone())
        {
            if is_running {
                Self::save_current_session(previous_session_id, index, stats, session_data);
            }
        }

        if let Ok(mut active_id_lock) = active_session_id.lock() {
            *active_id_lock = Some(session_id.clone());
        }

        Self::load_session(session_id, index, stats, session_data, result_sender);
    }

    fn save_current_session(
        session_id: String,
        index: &Arc<Mutex<HashMap<PathBuf, IndexFile>>>,
        stats: &Arc<Mutex<IndexStatistics>>,
        session_data: &Arc<Mutex<HashMap<String, SessionIndexData>>>,
    ) {
        if let (Ok(index_lock), Ok(stats_lock)) = (index.lock(), stats.lock()) {
            if let Ok(mut session_data_lock) = session_data.lock() {
                let mut data = SessionIndexData::default();
                data.index = index_lock.clone();
                data.stats = stats_lock.clone();
                let extensions: HashSet<String> = index_lock
                    .values()
                    .filter_map(|file| file.extension.clone())
                    .collect();
                data.extensions = extensions;
                session_data_lock.insert(session_id, data);
            }
        }
    }

    fn load_session(
        session_id: String,
        index: &Arc<Mutex<HashMap<PathBuf, IndexFile>>>,
        stats: &Arc<Mutex<IndexStatistics>>,
        session_data: &Arc<Mutex<HashMap<String, SessionIndexData>>>,
        result_sender: &Sender<IndexResult>,
    ) {
        if let Ok(session_data_lock) = session_data.lock() {
            if let Some(data) = session_data_lock.get(&session_id) {
                if let Ok(mut index_lock) = index.lock() {
                    *index_lock = data.index.clone();
                }

                if let Ok(mut stats_lock) = stats.lock() {
                    *stats_lock = data.stats.clone();
                    stats_lock.session_id = Some(session_id);

                    let _ = result_sender.send(IndexResult::Progress(stats_lock.clone()));
                    let _ = result_sender.send(IndexResult::Completed(data.index.len()));
                }
            }
        }
    }
}
