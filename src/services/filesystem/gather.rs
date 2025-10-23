use std::fs;
use std::path::Path;
use std::sync::Arc;

use ignore::WalkBuilder;

use crate::model::error::{SwarmError, SwarmResult};
use crate::model::options::Options;
use crate::model::path::PathExtensions;

use super::filter::{GlobPathFilter, PathFilter};

#[derive(Clone)]
pub struct GatherService;

impl GatherService {
    pub fn new() -> Self {
        Self
    }

    pub fn gather(&self, paths: &[String], options: &Options) -> SwarmResult<(String, usize)> {
        let filter: Arc<dyn PathFilter> = Arc::new(GlobPathFilter::from_options(options)?);
        let mut files = Vec::new();

        for path_str in paths {
            let path = Path::new(path_str.trim());
            let clean_path = path.clean_path();

            if !clean_path.exists() {
                continue;
            }

            if clean_path.is_file() {
                Self::collect_file(&clean_path, &mut files);
            } else if clean_path.is_dir() {
                Self::collect_directory(&clean_path, &mut files, &filter)?;
            }
        }

        let output = options.output_format.format(&files)?;
        let total_lines: usize = output.lines().count();

        Ok((output, total_lines))
    }

    fn collect_file(path: &Path, files: &mut Vec<(String, String)>) {
        if let Ok(content) = fs::read_to_string(path) {
            files.push((path.display().to_string(), content));
        }
    }

    fn collect_directory(
        directory: &Path,
        files: &mut Vec<(String, String)>,
        filter: &Arc<dyn PathFilter>,
    ) -> SwarmResult<()> {
        let walker = Self::create_walker(directory, filter);

        for result in walker {
            let entry = result.map_err(|error| {
                SwarmError::Other(format!("Error reading directory {}: {}", directory.display(), error))
            })?;

            if entry.file_type().is_some_and(|file_type| file_type.is_file())
                && filter.should_include(entry.path()) {
                    Self::collect_file(entry.path(), files);
                }
        }

        Ok(())
    }

    fn create_walker(directory: &Path, filter: &Arc<dyn PathFilter>) -> ignore::Walk {
        let filter = Arc::clone(filter);

        WalkBuilder::new(directory)
            .follow_links(true)
            .hidden(false)
            .ignore(false)
            .git_global(false)
            .git_exclude(false)
            .require_git(false)
            .filter_entry(move |entry| {
                filter.should_include(entry.path())
            })
            .build()
    }
}

impl Default for GatherService {
    fn default() -> Self {
        Self::new()
    }
}
