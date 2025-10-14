use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::sync::mpsc::Sender;
use std::thread;
use std::time::Duration;

use super::result::IndexResult;
use super::statistics::IndexStatistics;

pub struct ProgressReporter;

impl ProgressReporter {
    pub fn start(
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
}
