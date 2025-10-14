use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::sync::mpsc::Sender;
use std::thread;
use std::time::{Duration, Instant};

use crate::model::error::SwarmResult;
use crate::model::options::Options;

use super::file::IndexFile;
use super::result::IndexResult;
use super::statistics::IndexStatistics;
use super::stats::{FileCounter, ProgressUpdater, StatsFinalizer, StatsInitializer};
use super::walker::WalkerBuilder;

pub struct IndexBuilder;

impl IndexBuilder {
    pub fn build(
        paths: Vec<PathBuf>,
        options: Options,
        index: Arc<Mutex<HashMap<PathBuf, IndexFile>>>,
        stats: Arc<Mutex<IndexStatistics>>,
        is_running: Arc<AtomicBool>,
        is_paused: Arc<AtomicBool>,
        active_session_id: Arc<Mutex<Option<String>>>,
        result_sender: Sender<IndexResult>,
    ) -> SwarmResult<usize> {
        let start_time = Instant::now();

        StatsInitializer::initialize(&stats, &result_sender);

        let all_files = Self::collect_all_files(
            &paths,
            &options,
            &stats,
            &result_sender,
            &is_running,
            &is_paused,
            start_time
        );

        FileCounter::update(&stats, &result_sender, all_files.len());

        Self::index_files(
            all_files.clone(),
            index,
            stats.clone(),
            is_running,
            is_paused,
            result_sender.clone(),
            start_time
        )?;

        StatsFinalizer::finalize(&stats, &active_session_id, &result_sender, all_files.len(), start_time);

        Ok(all_files.len())
    }

    fn collect_all_files(
        paths: &[PathBuf],
        options: &Options,
        stats: &Arc<Mutex<IndexStatistics>>,
        result_sender: &Sender<IndexResult>,
        is_running: &Arc<AtomicBool>,
        is_paused: &Arc<AtomicBool>,
        start_time: Instant,
    ) -> Vec<PathBuf> {
        let mut all_files = Vec::new();

        for path in paths {
            if !is_running.load(Ordering::Relaxed) {
                break;
            }

            ProgressUpdater::update(
                stats,
                result_sender,
                Some(path.to_string_lossy().to_string()),
                start_time.elapsed().as_millis() as u64,
            );

            if path.is_dir() {
                let walker = WalkerBuilder::create(path, options.clone());
                Self::collect_files_from_walker(
                    walker,
                    &mut all_files,
                    stats,
                    result_sender,
                    is_running,
                    is_paused,
                    start_time
                );
            } else if path.is_file() {
                all_files.push(path.clone());
            }
        }

        all_files
    }

    fn collect_files_from_walker(
        walker: ignore::Walk,
        all_files: &mut Vec<PathBuf>,
        stats: &Arc<Mutex<IndexStatistics>>,
        result_sender: &Sender<IndexResult>,
        is_running: &Arc<AtomicBool>,
        is_paused: &Arc<AtomicBool>,
        start_time: Instant,
    ) {
        for entry in walker {
            if !is_running.load(Ordering::Relaxed) {
                break;
            }

            while is_paused.load(Ordering::Relaxed) {
                thread::sleep(Duration::from_millis(100));
                if !is_running.load(Ordering::Relaxed) {
                    break;
                }
            }

            if let Ok(entry) = entry {
                let entry_path = entry.path();

                ProgressUpdater::update(
                    stats,
                    result_sender,
                    Some(entry_path.to_string_lossy().to_string()),
                    start_time.elapsed().as_millis() as u64,
                );

                if entry.file_type().map_or(false, |file_type| file_type.is_file()) {
                    all_files.push(entry_path.to_path_buf());
                }
            }
        }
    }

    fn index_files(
        all_files: Vec<PathBuf>,
        index: Arc<Mutex<HashMap<PathBuf, IndexFile>>>,
        stats: Arc<Mutex<IndexStatistics>>,
        is_running: Arc<AtomicBool>,
        is_paused: Arc<AtomicBool>,
        result_sender: Sender<IndexResult>,
        start_time: Instant,
    ) -> SwarmResult<()> {
        for (idx, path) in all_files.iter().enumerate() {
            if !is_running.load(Ordering::Relaxed) {
                break;
            }

            while is_paused.load(Ordering::Relaxed) {
                thread::sleep(Duration::from_millis(100));
                if !is_running.load(Ordering::Relaxed) {
                    break;
                }
            }

            if let Ok(indexed_file) = IndexFile::new(path.clone()) {
                if let Ok(mut index_lock) = index.lock() {
                    index_lock.insert(path.clone(), indexed_file.clone());
                }

                if let Ok(mut stats_lock) = stats.lock() {
                    stats_lock.indexed_files = idx + 1;
                    stats_lock.total_size += indexed_file.size;
                    stats_lock.current_path = Some(path.to_string_lossy().to_string());
                    stats_lock.duration_ms = start_time.elapsed().as_millis() as u64;

                    let _ = result_sender.send(IndexResult::Progress(stats_lock.clone()));
                }
            }
        }

        Ok(())
    }
}
