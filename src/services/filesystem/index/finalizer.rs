use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::Sender;
use std::thread;
use std::time::Duration;

use super::file::IndexFile;
use super::result::IndexResult;
use super::session::SessionIndexData;
use super::statistics::IndexStatistics;

pub struct IndexFinalizer;

impl IndexFinalizer {
    pub fn finalize(
        stats: &Arc<Mutex<IndexStatistics>>,
        index: &Arc<Mutex<HashMap<PathBuf, IndexFile>>>,
        session_data: &Arc<Mutex<HashMap<String, SessionIndexData>>>,
        active_session_id: &Arc<Mutex<Option<String>>>,
        result_sender: &Sender<IndexResult>,
        count: usize,
    ) {
        if let Ok(mut stats_lock) = stats.lock() {
            stats_lock.is_complete = true;
            stats_lock.current_operation = Some("Indexing complete".to_string());
            stats_lock.current_path = Some("Done".to_string());

            let _ = result_sender.send(IndexResult::Progress(stats_lock.clone()));
        }

        if let (Ok(index_lock), Ok(stats_lock), Ok(active_id_lock)) =
            (index.lock(), stats.lock(), active_session_id.lock())
        {
            if let Some(session_id) = active_id_lock.as_ref() {
                if let Ok(mut session_data_lock) = session_data.lock() {
                    let mut data = SessionIndexData::default();
                    data.index = index_lock.clone();
                    data.stats = stats_lock.clone();

                    let extensions: HashSet<String> = index_lock
                        .values()
                        .filter_map(|file| file.extension.clone())
                        .collect();
                    data.extensions = extensions;

                    session_data_lock.insert(session_id.clone(), data);
                }
            }
        }

        for _ in 0..3 {
            if result_sender.send(IndexResult::Completed(count)).is_ok() {
                break;
            }
            thread::sleep(Duration::from_millis(10));
        }
    }
}
