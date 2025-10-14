use std::sync::{Arc, Mutex};
use std::sync::mpsc::Sender;
use std::time::Instant;

use super::result::IndexResult;
use super::statistics::IndexStatistics;

pub struct StatsInitializer;

impl StatsInitializer {
    pub fn initialize(stats: &Arc<Mutex<IndexStatistics>>, result_sender: &Sender<IndexResult>) {
        if let Ok(mut stats_lock) = stats.lock() {
            stats_lock.total_files = 0;
            stats_lock.indexed_files = 0;
            stats_lock.total_size = 0;
            stats_lock.current_path = Some("Starting...".to_string());
            stats_lock.duration_ms = 0;

            let _ = result_sender.send(IndexResult::Progress(stats_lock.clone()));
        }
    }
}

pub struct ProgressUpdater;

impl ProgressUpdater {
    pub fn update(
        stats: &Arc<Mutex<IndexStatistics>>,
        result_sender: &Sender<IndexResult>,
        current_path: Option<String>,
        duration_ms: u64,
    ) {
        if let Ok(mut stats_lock) = stats.lock() {
            if let Some(path) = current_path {
                stats_lock.current_path = Some(path);
            }
            stats_lock.duration_ms = duration_ms;
            let _ = result_sender.send(IndexResult::Progress(stats_lock.clone()));
        }
    }
}

pub struct FileCounter;

impl FileCounter {
    pub fn update(
        stats: &Arc<Mutex<IndexStatistics>>,
        result_sender: &Sender<IndexResult>,
        count: usize,
    ) {
        if let Ok(mut stats_lock) = stats.lock() {
            stats_lock.total_files = count;
            stats_lock.current_path = Some(format!("Found {} files", count));
            stats_lock.current_operation = Some("Processing files...".to_string());
            let _ = result_sender.send(IndexResult::Progress(stats_lock.clone()));
        }
    }
}

pub struct StatsFinalizer;

impl StatsFinalizer {
    pub fn finalize(
        stats: &Arc<Mutex<IndexStatistics>>,
        active_session_id: &Arc<Mutex<Option<String>>>,
        result_sender: &Sender<IndexResult>,
        file_count: usize,
        start_time: Instant,
    ) {
        if let Ok(mut stats_lock) = stats.lock() {
            stats_lock.indexed_files = file_count;
            stats_lock.duration_ms = start_time.elapsed().as_millis() as u64;
            stats_lock.is_complete = true;
            stats_lock.current_path = Some("Indexing complete".to_string());
            stats_lock.current_operation = Some("Complete".to_string());

            let _ = result_sender.send(IndexResult::Progress(stats_lock.clone()));

            if let Ok(session_id_lock) = active_session_id.lock() {
                if let Some(session_id) = session_id_lock.as_ref() {
                    stats_lock.session_id = Some(session_id.clone());
                }
            }
        }
    }
}
