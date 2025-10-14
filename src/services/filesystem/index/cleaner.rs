use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use super::file::IndexFile;
use super::statistics::IndexStatistics;

pub struct IndexCleaner;

impl IndexCleaner {
    pub fn clear(
        index: &Arc<Mutex<HashMap<PathBuf, IndexFile>>>,
        stats: &Arc<Mutex<IndexStatistics>>,
        active_session_id: &Arc<Mutex<Option<String>>>,
    ) {
        if let Ok(mut index_lock) = index.lock() {
            index_lock.clear();
        }

        if let Ok(mut stats_lock) = stats.lock() {
            *stats_lock = IndexStatistics::default();

            if let Ok(session_id_lock) = active_session_id.lock() {
                if let Some(id) = &*session_id_lock {
                    stats_lock.session_id = Some(id.clone());
                }
            }
        }
    }
}
