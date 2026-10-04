use core::sync::atomic::{AtomicU64, Ordering};
use std::fs::{self, File};
use std::io::Read as _;
use std::path::{Path, PathBuf};

use git2::Repository;
use rayon::iter::{IntoParallelRefIterator as _, ParallelIterator as _};

use crate::constants::OUTPUT_BYTES_MAX;
use crate::model::error::{SwarmError, SwarmResult};
use crate::model::options::Options;
use crate::model::output::OutputEntry;
use crate::model::path;
use crate::model::query::{ParsedQuery, SearchCommand};
use crate::services::search::SearchScratch;
use crate::services::tokens::estimate_tokens;
use crate::services::tree::query::MatchContext;

use super::filter::GlobPathFilter;
use super::git::GitService;
use super::walk::{WalkOptions, walk_files};

struct Candidate {
    path: PathBuf,
    walked: bool,
}

enum FileOutcome {
    Entries(Vec<GatherEntry>),
    Filtered,
    Overflow,
    Skipped,
}

struct GatherPlan<'plan> {
    git: Option<&'plan GitService>,
    include_diff: bool,
    matcher: Option<MatchContext<'plan>>,
}

struct GatherEntry {
    content: String,
    label: String,
    path: PathBuf,
}

pub struct GatherRequest<'request> {
    pub git: Option<&'request GitService>,
    pub options: &'request Options,
    pub paths: &'request [PathBuf],
    pub query: Option<&'request ParsedQuery>,
}

#[derive(Clone, Copy, Debug)]
pub struct GatherStats {
    pub lines: u64,
    pub skipped: u32,
    pub tokens: u64,
}

struct WorkerState {
    repository: Option<Repository>,
    scratch: SearchScratch,
}

impl WorkerState {
    fn new(plan: &GatherPlan<'_>) -> Self {
        let repository = if plan.include_diff {
            plan.git.and_then(GitService::open_repository)
        } else {
            None
        };

        Self {
            repository,
            scratch: SearchScratch::default(),
        }
    }
}

fn candidates_for(paths: &[PathBuf], options: &Options) -> SwarmResult<(Vec<Candidate>, u32)> {
    let filter = GlobPathFilter::from_options(options)?;
    let mut candidates = Vec::with_capacity(paths.len());
    let mut skipped_count: u32 = 0;

    for requested in paths {
        let canonical = path::canonical(requested);

        let Ok(metadata) = fs::metadata(&canonical) else {
            skipped_count += 1;

            continue;
        };

        if metadata.is_file() {
            candidates.push(Candidate {
                path: canonical,
                walked: false,
            });

            continue;
        }

        if !metadata.is_dir() {
            skipped_count += 1;

            continue;
        }

        let walked = walk_files(&canonical, &filter, WalkOptions::GATHER)?;

        candidates.extend(
            walked
                .into_iter()
                .map(|path| Candidate { path, walked: true }),
        );
    }

    debug_assert!(skipped_count as usize <= paths.len());

    Ok((candidates, skipped_count))
}

fn entries_for(
    candidate: &Candidate,
    content: String,
    plan: &GatherPlan<'_>,
    repository: Option<&Repository>,
) -> Vec<GatherEntry> {
    let display = candidate.path.display();

    if plan.include_diff {
        if let Some(original) = original_for(&candidate.path, plan.git, repository) {
            return vec![
                GatherEntry {
                    content: original,
                    label: format!("{display} (original)"),
                    path: candidate.path.clone(),
                },
                GatherEntry {
                    content,
                    label: format!("{display} (modified)"),
                    path: candidate.path.clone(),
                },
            ];
        }
    }

    vec![GatherEntry {
        content,
        label: display.to_string(),
        path: candidate.path.clone(),
    }]
}

pub fn gather(request: &GatherRequest<'_>) -> SwarmResult<(String, GatherStats)> {
    let plan = GatherPlan {
        git: request.git,
        include_diff: request
            .query
            .is_some_and(|query| query.has_command(SearchCommand::Diff)),
        matcher: request
            .query
            .filter(|query| !query.is_empty())
            .map(|query| MatchContext::new(query, request.git)),
    };

    let (candidates, skipped_walk) = candidates_for(request.paths, request.options)?;
    let budget = AtomicU64::new(0);

    let outcomes: Vec<FileOutcome> = candidates
        .par_iter()
        .map_init(
            || WorkerState::new(&plan),
            |state, candidate| process(candidate, &plan, state, &budget),
        )
        .collect();

    assert_eq!(outcomes.len(), candidates.len());

    let (mut entries, skipped_read) = merge(outcomes)?;

    entries.sort_by(|left, right| left.path.cmp(&right.path));

    let formatted: Vec<OutputEntry> = entries
        .into_iter()
        .map(|entry| OutputEntry {
            content: entry.content,
            label: entry.label,
        })
        .collect();

    let format = request
        .query
        .and_then(|query| query.format_override)
        .unwrap_or(request.options.output_format);

    let output = format.format(&formatted)?;

    if output.len() as u64 > OUTPUT_BYTES_MAX {
        return Err(output_overflow());
    }

    let stats = GatherStats {
        lines: memchr::memchr_iter(b'\n', output.as_bytes()).count() as u64,
        skipped: skipped_walk + skipped_read,
        tokens: estimate_tokens(&output),
    };

    Ok((output, stats))
}

fn merge(outcomes: Vec<FileOutcome>) -> SwarmResult<(Vec<GatherEntry>, u32)> {
    let mut entries = Vec::with_capacity(outcomes.len());
    let mut skipped_count: u32 = 0;

    for outcome in outcomes {
        match outcome {
            FileOutcome::Entries(found) => entries.extend(found),
            FileOutcome::Filtered => {}
            FileOutcome::Overflow => return Err(output_overflow()),
            FileOutcome::Skipped => skipped_count += 1,
        }
    }

    Ok((entries, skipped_count))
}

fn original_for(
    path: &Path,
    git: Option<&GitService>,
    repository: Option<&Repository>,
) -> Option<String> {
    let service = git?;

    if !service.status(path).has_diff() {
        return None;
    }

    service.original_content(repository?, path)
}

fn output_overflow() -> SwarmError {
    SwarmError::Validation(format!(
        "the selection holds more than {OUTPUT_BYTES_MAX} bytes; narrow it",
    ))
}

fn process(
    candidate: &Candidate,
    plan: &GatherPlan<'_>,
    state: &mut WorkerState,
    budget: &AtomicU64,
) -> FileOutcome {
    if candidate.walked {
        if let Some(matcher) = plan.matcher.as_ref() {
            if !matcher.path_matches(&candidate.path, &mut state.scratch) {
                return FileOutcome::Filtered;
            }
        }
    }

    let Ok(metadata) = fs::symlink_metadata(&candidate.path) else {
        return FileOutcome::Skipped;
    };

    if !metadata.is_file() {
        return FileOutcome::Filtered;
    }

    let Ok(file) = File::open(&candidate.path) else {
        return FileOutcome::Skipped;
    };

    let size_bytes = metadata.len();
    let reserved = budget.fetch_add(size_bytes, Ordering::Relaxed) + size_bytes;

    if reserved > OUTPUT_BYTES_MAX {
        return FileOutcome::Overflow;
    }

    let capacity = usize::try_from(size_bytes).expect("a budgeted file fits in memory");
    let mut content = String::with_capacity(capacity);

    if file.take(size_bytes).read_to_string(&mut content).is_err() {
        return FileOutcome::Skipped;
    }

    let entries = entries_for(candidate, content, plan, state.repository.as_ref());

    debug_assert_ne!(entries.len(), 0);

    FileOutcome::Entries(entries)
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TemporaryTree {
        root: PathBuf,
    }

    impl TemporaryTree {
        fn new(files: &[(&str, &str)]) -> Self {
            let root = std::env::temp_dir().join(format!("swarm-gather-{}", uuid::Uuid::new_v4()));

            fs::create_dir_all(&root).expect("the test directory is creatable");

            for (name, content) in files {
                fs::write(root.join(name), content).expect("the test file is writable");
            }

            Self { root }
        }
    }

    impl Drop for TemporaryTree {
        fn drop(&mut self) {
            fs::remove_dir_all(&self.root).expect("the test directory is removable");
        }
    }

    fn sample() -> TemporaryTree {
        TemporaryTree::new(&[("keep.toml", "alpha = 1\n"), ("drop.rs", "fn beta() {}\n")])
    }

    fn gather_with(paths: &[PathBuf], text: &str) -> (String, GatherStats) {
        let query = ParsedQuery::parse(text);
        let options = Options::default();

        let request = GatherRequest {
            git: None,
            options: &options,
            paths,
            query: Some(&query),
        };

        gather(&request).expect("the gather succeeds")
    }

    #[test]
    fn a_query_filters_the_walked_files() {
        let tree = sample();
        let (output, _) = gather_with(core::slice::from_ref(&tree.root), "ext:toml");

        assert!(output.contains("alpha = 1"));
        assert!(!output.contains("fn beta"));
    }

    #[test]
    fn an_empty_query_gathers_every_file() {
        let tree = sample();
        let (output, _) = gather_with(core::slice::from_ref(&tree.root), "");

        assert!(output.contains("alpha = 1"));
        assert!(output.contains("fn beta"));
    }

    #[test]
    fn a_query_leaves_an_explicitly_named_file_alone() {
        let tree = sample();
        let paths = [tree.root.join("keep.toml")];
        let (output, stats) = gather_with(&paths, "ext:rs");

        assert!(output.contains("alpha = 1"));
        assert_eq!(stats.skipped, 0);
    }

    #[test]
    fn a_content_query_reaches_into_the_files() {
        let tree = sample();
        let (output, _) = gather_with(core::slice::from_ref(&tree.root), "content:beta");

        assert!(output.contains("fn beta"));
        assert!(!output.contains("alpha = 1"));
    }

    #[test]
    fn a_missing_path_is_counted_as_skipped() {
        let tree = sample();
        let paths = [tree.root.join("missing.rs"), tree.root.join("keep.toml")];
        let (output, stats) = gather_with(&paths, "");

        assert!(output.contains("alpha = 1"));
        assert_eq!(stats.skipped, 1);
    }

    #[test]
    fn entries_are_ordered_by_path() {
        let tree = sample();
        let (output, _) = gather_with(core::slice::from_ref(&tree.root), "");
        let first = output.find("drop.rs").expect("the first file is present");

        let second = output
            .find("keep.toml")
            .expect("the second file is present");

        assert!(first < second);
    }
}
