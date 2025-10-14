use serde::{Deserialize, Serialize};

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
