use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

use super::file::IndexFile;
use super::statistics::IndexStatistics;

#[derive(Clone, Default)]
pub struct SessionIndexData {
    pub extensions: HashSet<String>,
    pub index: HashMap<PathBuf, IndexFile>,
    pub stats: IndexStatistics,
}
