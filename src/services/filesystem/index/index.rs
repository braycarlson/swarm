use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use std::time::{Duration, Instant};

use ignore::WalkBuilder;
use serde::{Deserialize, Serialize};

use crate::model::error::SwarmResult;
use crate::model::options::Options;
use crate::model::path::PathExtensions;
use crate::services::tree::filter;

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct IndexStatistics {
    pub current_operation: Option<String>,
    pub current_path: Option<String>,
    pub duration_ms: u64,
    pub indexed_files: usize,
    pub is_complete: bool,
    pub session_id: Option<String>,
    pub total_files: usize,
    pub total_size: u64,
}

#[derive(Clone, Debug)]
pub struct IndexFile {
    pub extension: Option<String>,
    pub name: String,
    pub path: PathBuf,
    pub size: u64,
}

impl IndexFile {
    pub fn new(path: PathBuf) -> SwarmResult<Self> {
        let metadata = fs::metadata(&path)?;
        let name = path.file_name_string().unwrap_or_default();
        let extension = path.extension()
            .and_then(|ext| ext.to_str())
            .map(String::from);

        Ok(Self {
            extension,
            name,
            path,
            size: metadata.len(),
        })
    }
}

pub enum IndexCommand {
    Pause,
    Resume,
    Start(Vec<PathBuf>, Options),
    Stop,
    SwitchSession(String),
}

pub enum IndexResult {
    Completed(usize),
    Error(String),
    Progress(IndexStatistics),
}

#[derive(Clone, Default)]
struct SessionIndexData {
    extensions: HashSet<String>,
    index: HashMap<PathBuf, IndexFile>,
    stats: IndexStatistics,
}

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

        let worker_active_session = Arc::clone(&active_session_id);
        let worker_index = Arc::clone(&index);
        let worker_is_paused = Arc::clone(&is_paused);
        let worker_is_running = Arc::clone(&is_running);
        let worker_session_data = Arc::clone(&session_data);
        let worker_stats = Arc::clone(&stats);

        thread::spawn(move || {
            Self::worker_thread(
                command_receiver,
                result_sender,
                worker_index,
                worker_stats,
                worker_is_running,
                worker_is_paused,
                worker_session_data,
                worker_active_session,
            );
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

    fn worker_thread(
        command_receiver: Receiver<IndexCommand>,
        result_sender: Sender<IndexResult>,
        index: Arc<Mutex<HashMap<PathBuf, IndexFile>>>,
        stats: Arc<Mutex<IndexStatistics>>,
        is_running: Arc<AtomicBool>,
        is_paused: Arc<AtomicBool>,
        session_data: Arc<Mutex<HashMap<String, SessionIndexData>>>,
        active_session_id: Arc<Mutex<Option<String>>>,
    ) {
        while let Ok(command) = command_receiver.recv() {
            match command {
                IndexCommand::Start(paths, options) => {
                    if is_running.load(Ordering::Relaxed) {
                        let _ = result_sender.send(IndexResult::Error("Indexing already in progress".into()));
                        continue;
                    }

                    is_running.store(true, Ordering::Relaxed);
                    is_paused.store(false, Ordering::Relaxed);

                    Self::clear_index(&index, &stats, &active_session_id);
                    Self::start_progress_reporter(&stats, &result_sender, &is_running, &is_paused);

                    let result = Self::index_paths(
                        paths,
                        options,
                        Arc::clone(&index),
                        Arc::clone(&stats),
                        Arc::clone(&is_running),
                        Arc::clone(&is_paused),
                        Arc::clone(&session_data),
                        Arc::clone(&active_session_id),
                        result_sender.clone(),
                    );

                    match result {
                        Ok(count) => {
                            Self::finalize_indexing(
                                &stats,
                                &index,
                                &session_data,
                                &active_session_id,
                                &result_sender,
                                count,
                            );
                        }
                        Err(error) => {
                            let _ = result_sender.send(IndexResult::Error(format!("Indexing error: {}", error)));
                        }
                    }

                    is_running.store(false, Ordering::SeqCst);
                }
                IndexCommand::Stop => {
                    is_running.store(false, Ordering::Relaxed);
                    is_paused.store(false, Ordering::Relaxed);
                }
                IndexCommand::Pause => {
                    is_paused.store(true, Ordering::Relaxed);
                }
                IndexCommand::Resume => {
                    is_paused.store(false, Ordering::Relaxed);
                }
                IndexCommand::SwitchSession(session_id) => {
                    Self::handle_session_switch(
                        session_id,
                        &index,
                        &stats,
                        &session_data,
                        &active_session_id,
                        &result_sender,
                        is_running.load(Ordering::Relaxed),
                    );
                }
            }
        }
    }

    fn clear_index(
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

    fn start_progress_reporter(
        stats: &Arc<Mutex<IndexStatistics>>,
        result_sender: &Sender<IndexResult>,
        is_running: &Arc<AtomicBool>,
        is_paused: &Arc<AtomicBool>,
    ) {
        let stats_clone = Arc::clone(stats);
        let result_sender_clone = result_sender.clone();
        let is_running_clone = Arc::clone(is_running);
        let is_paused_clone = Arc::clone(is_paused);

        thread::spawn(move || {
            while is_running_clone.load(Ordering::Relaxed) {
                if !is_paused_clone.load(Ordering::Relaxed) {
                    if let Ok(stats) = stats_clone.lock() {
                        let _ = result_sender_clone.send(IndexResult::Progress(stats.clone()));
                    }
                }
                thread::sleep(Duration::from_millis(100));
            }
        });
    }

    fn index_paths(
            paths: Vec<PathBuf>,
            options: Options,
            index: Arc<Mutex<HashMap<PathBuf, IndexFile>>>,
            stats: Arc<Mutex<IndexStatistics>>,
            is_running: Arc<AtomicBool>,
            is_paused: Arc<AtomicBool>,
            _session_data: Arc<Mutex<HashMap<String, SessionIndexData>>>,
            active_session_id: Arc<Mutex<Option<String>>>,
            result_sender: Sender<IndexResult>,
        ) -> SwarmResult<usize> {
            let start_time = Instant::now();

            Self::initialize_stats(&stats, &result_sender);

            let mut all_files = Vec::new();

            for path in &paths {
                if !is_running.load(Ordering::Relaxed) {
                    break;
                }

                Self::update_progress(
                    &stats,
                    &result_sender,
                    Some(path.to_string_lossy().to_string()),
                    start_time.elapsed().as_millis() as u64,
                );

                if path.is_dir() {
                    let walker = Self::create_filtered_walker(path, options.clone());

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

                            Self::update_progress(
                                &stats,
                                &result_sender,
                                Some(entry_path.to_string_lossy().to_string()),
                                start_time.elapsed().as_millis() as u64,
                            );

                            if entry.file_type().map_or(false, |file_type| file_type.is_file()) {
                                all_files.push(entry_path.to_path_buf());
                            }
                        }
                    }
                } else if path.is_file() {
                    all_files.push(path.clone());
                }
            }

            Self::update_file_count(&stats, &result_sender, all_files.len());

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

            Self::finalize_stats(&stats, &active_session_id, &result_sender, all_files.len(), start_time);

            Ok(all_files.len())
        }

    fn initialize_stats(stats: &Arc<Mutex<IndexStatistics>>, result_sender: &Sender<IndexResult>) {
        if let Ok(mut stats_lock) = stats.lock() {
            stats_lock.total_files = 0;
            stats_lock.indexed_files = 0;
            stats_lock.total_size = 0;
            stats_lock.current_path = Some("Starting...".to_string());
            stats_lock.duration_ms = 0;

            let _ = result_sender.send(IndexResult::Progress(stats_lock.clone()));
        }
    }

    fn update_progress(
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

    fn update_file_count(
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

    fn finalize_stats(
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

    fn create_filtered_walker(directory: &Path, options: Options) -> ignore::Walk {
        let options = Arc::new(options);
        let options_clone = Arc::clone(&options);

        WalkBuilder::new(directory)
            .follow_links(true)
            .hidden(true)
            .ignore(false)
            .git_global(false)
            .git_exclude(false)
            .git_ignore(false)
            .require_git(false)
            .filter_entry(move |entry| {
                let path = entry.path();

                if path.is_hidden() {
                    return false;
                }

                if entry.file_type().map_or(false, |file_type| file_type.is_dir()) {
                    return !filter::is_path_in_excluded_patterns(path, &options_clone.exclude);
                }

                filter::should_include_path(path, &options_clone)
            })
            .build()
    }

    fn finalize_indexing(
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

    fn handle_session_switch(
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
                        session_data_lock.insert(previous_session_id, data);
                    }
                }
            }
        }

        if let Ok(mut active_id_lock) = active_session_id.lock() {
            *active_id_lock = Some(session_id.clone());
        }

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
