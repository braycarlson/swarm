use std::fs;
use std::path::PathBuf;

use crate::model::error::SwarmResult;
use crate::model::path::PathExtensions;

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
