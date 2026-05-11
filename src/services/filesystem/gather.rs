use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use ignore::WalkBuilder;
use rayon::prelude::*;

use crate::app::state::search::{Command, ParsedQuery};
use crate::model::error::SwarmResult;
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
                if let Some(entries) = Self::read_and_process(&clean_path, git_service, include_diff) {
                    files.extend(entries);
                }
            } else if clean_path.is_dir() {
                Self::collect_directory(&clean_path, &mut files, &filter, git_service, include_diff)?;
            }
        }

        let output_format = query
            .and_then(|q| q.format_override)
            .unwrap_or(options.output_format);

        let output = output_format.format(&files)?;

        let line_count = memchr::memchr_iter(b'\n', output.as_bytes()).count();
        let token_count = estimate_tokens(&output);

        let stats = GatherStats {
            line_count,
            token_count,
        };

        Ok((output, stats))
    }

    fn read_and_process(
        path: &Path,
        git_service: Option<&GitService>,
        include_diff: bool,
    ) -> Option<Vec<(String, String)>> {
        let current_content = fs::read_to_string(path).ok()?;
        let mut entries = Vec::new();

        if include_diff {
            if let Some(git) = git_service {
                let status = git.get_status(path);

                if status.has_diff() {
                    if let Some(original) = git.get_original_content(path) {
                        entries.push((
                            format!("{} (original)", path.display()),
                            original,
                        ));
                        entries.push((
                            format!("{} (modified)", path.display()),
                            current_content,
                        ));
                        return Some(entries);
                    }
                }
            }
        }

        entries.push((path.display().to_string(), current_content));
        Some(entries)
    }

    fn collect_directory(
        directory: &Path,
        files: &mut Vec<(String, String)>,
        filter: &Arc<dyn PathFilter>,
        git_service: Option<&GitService>,
        include_diff: bool,
    ) -> SwarmResult<()> {
        let paths = Self::walk_parallel(directory, filter);

        let results: Vec<Vec<(String, String)>> = paths
            .par_iter()
            .filter_map(|path| Self::read_and_process(path, git_service, include_diff))
            .collect();

        for batch in results {
            files.extend(batch);
        }

        Ok(())
    }

    fn walk_parallel(directory: &Path, filter: &Arc<dyn PathFilter>) -> Vec<PathBuf> {
        let (sender, receiver) = std::sync::mpsc::channel::<PathBuf>();

        WalkBuilder::new(directory)
            .follow_links(false)
            .hidden(false)
            .ignore(false)
            .git_global(false)
            .git_exclude(false)
            .require_git(false)
            .build_parallel()
            .run(|| {
                let sender = sender.clone();
                let filter = Arc::clone(filter);
                let mut local: Vec<PathBuf> = Vec::new();

                Box::new(move |result| {
                    match result {
                        Ok(entry) => {
                            if !filter.should_include(entry.path()) {
                                if entry.file_type().is_some_and(|ft| ft.is_dir()) {
                                    if !local.is_empty() {
                                        for path in local.drain(..) {
                                            let _ = sender.send(path);
                                        }
                                    }
                                    return ignore::WalkState::Skip;
                                }
                                return ignore::WalkState::Continue;
                            }

                            if entry.file_type().is_some_and(|ft| ft.is_file()) {
                                local.push(entry.into_path());

                                if local.len() >= 64 {
                                    for path in local.drain(..) {
                                        let _ = sender.send(path);
                                    }
                                }
                            }

                            ignore::WalkState::Continue
                        }
                        Err(_) => ignore::WalkState::Continue,
                    }
                })
            });

        drop(sender);
        receiver.iter().collect()
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

    let char_count = text.len();
    let word_count = text.split_ascii_whitespace().count();

    let char_estimate = char_count / 4;
    let word_estimate = word_count * 13 / 10;

    (char_estimate + word_estimate) / 2
}
