use std::fs;
use std::path::Path;
use std::sync::Arc;

use ignore::WalkBuilder;

use crate::app::state::search::{Command, ParsedQuery};
use crate::model::error::{SwarmError, SwarmResult};
use crate::model::options::Options;
use crate::model::path::PathExtensions;

use super::filter::{GlobPathFilter, PathFilter};
use super::git::GitService;

#[derive(Clone, Debug)]
pub struct GatherStats {
    pub line_count: usize,
    pub token_count: usize,
}

#[derive(Clone)]
pub struct GatherService;

impl GatherService {
    pub fn new() -> Self {
        Self
    }

    pub fn gather(&self, paths: &[String], options: &Options) -> SwarmResult<(String, GatherStats)> {
        self.gather_with_context(paths, options, None, None)
    }

    pub fn gather_with_context(
        &self,
        paths: &[String],
        options: &Options,
        git_service: Option<&GitService>,
        query: Option<&ParsedQuery>,
    ) -> SwarmResult<(String, GatherStats)> {
        let filter: Arc<dyn PathFilter> = Arc::new(GlobPathFilter::from_options(options)?);
        let mut files = Vec::new();

        let include_diff = query.is_some_and(|q| q.has_command(Command::Diff));

        for path_str in paths {
            let path = Path::new(path_str.trim());
            let clean_path = path.clean_path();

            if !clean_path.exists() {
                continue;
            }

            if clean_path.is_file() {
                Self::collect_file(&clean_path, &mut files, git_service, include_diff);
            } else if clean_path.is_dir() {
                Self::collect_directory(&clean_path, &mut files, &filter, git_service, include_diff)?;
            }
        }

        let output_format = query
            .and_then(|q| q.format_override)
            .unwrap_or(options.output_format);

        let output = output_format.format(&files)?;

        let stats = GatherStats {
            line_count: output.lines().count(),
            token_count: estimate_tokens(&output),
        };

        Ok((output, stats))
    }

    fn collect_file(
        path: &Path,
        files: &mut Vec<(String, String)>,
        git_service: Option<&GitService>,
        include_diff: bool,
    ) {
        let current_content = match fs::read_to_string(path) {
            Ok(c) => c,
            Err(_) => return,
        };

        if include_diff {
            if let Some(git) = git_service {
                let status = git.get_status(path);

                if status.has_diff() {
                    if let Some(original) = git.get_original_content(path) {
                        files.push((
                            format!("{} (original)", path.display()),
                            original,
                        ));
                        files.push((
                            format!("{} (modified)", path.display()),
                            current_content,
                        ));
                        return;
                    }
                }
            }
        }

        files.push((path.display().to_string(), current_content));
    }

    fn collect_directory(
        directory: &Path,
        files: &mut Vec<(String, String)>,
        filter: &Arc<dyn PathFilter>,
        git_service: Option<&GitService>,
        include_diff: bool,
    ) -> SwarmResult<()> {
        let walker = Self::create_walker(directory, filter);

        for result in walker {
            let entry = result.map_err(|error| {
                SwarmError::Other(format!("Error reading directory {}: {}", directory.display(), error))
            })?;

            if entry.file_type().is_some_and(|file_type| file_type.is_file())
                && filter.should_include(entry.path()) {
                    Self::collect_file(entry.path(), files, git_service, include_diff);
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

pub fn estimate_tokens(text: &str) -> usize {
    if text.is_empty() {
        return 0;
    }

    let char_count = text.chars().count();
    let word_count = text.split_whitespace().count();

    let char_estimate = char_count / 4;
    let word_estimate = (word_count as f64 * 1.3) as usize;

    (char_estimate + word_estimate) / 2
}
