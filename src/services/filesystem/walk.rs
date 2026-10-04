use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, SyncSender};
use std::thread;

use ignore::{DirEntry, WalkBuilder, WalkState};

use crate::constants::{WALK_FILE_COUNT_MAX, WALK_QUEUE_COUNT_MAX};
use crate::model::error::{SwarmError, SwarmResult};

use super::filter::GlobPathFilter;

#[derive(Clone, Copy, Debug)]
pub struct WalkOptions {
    pub git_exclude: bool,
    pub git_global: bool,
    pub git_ignore: bool,
    pub ignore: bool,
    pub require_git: bool,
    pub skip_hidden: bool,
}

impl WalkOptions {
    pub const GATHER: Self = Self {
        git_exclude: false,
        git_global: false,
        git_ignore: true,
        ignore: false,
        require_git: false,
        skip_hidden: false,
    };

    pub const SKELETON: Self = Self {
        git_exclude: true,
        git_global: true,
        git_ignore: true,
        ignore: true,
        require_git: true,
        skip_hidden: true,
    };
}

fn visit_entry(
    result: Result<DirEntry, ignore::Error>,
    filter: &GlobPathFilter,
    sender: &SyncSender<PathBuf>,
) -> WalkState {
    let Ok(entry) = result else {
        return WalkState::Continue;
    };

    let is_directory = entry.file_type().is_some_and(|kind| kind.is_dir());

    if !filter.should_include(entry.path(), is_directory) {
        if is_directory {
            return WalkState::Skip;
        }

        return WalkState::Continue;
    }

    if is_directory {
        return WalkState::Continue;
    }

    if sender.send(entry.into_path()).is_err() {
        return WalkState::Quit;
    }

    WalkState::Continue
}

pub fn walk_files(
    directory: &Path,
    filter: &GlobPathFilter,
    options: WalkOptions,
) -> SwarmResult<Vec<PathBuf>> {
    let bound = WALK_FILE_COUNT_MAX as usize;
    let (sender, receiver) = mpsc::sync_channel::<PathBuf>(WALK_QUEUE_COUNT_MAX as usize);

    let collector = thread::Builder::new()
        .name("walk-collector".to_owned())
        .spawn(move || receiver.iter().take(bound + 1).collect::<Vec<PathBuf>>())?;

    WalkBuilder::new(directory)
        .follow_links(false)
        .git_exclude(options.git_exclude)
        .git_global(options.git_global)
        .git_ignore(options.git_ignore)
        .hidden(options.skip_hidden)
        .ignore(options.ignore)
        .require_git(options.require_git)
        .build_parallel()
        .run(|| {
            let entry_sender = sender.clone();

            Box::new(move |result| visit_entry(result, filter, &entry_sender))
        });

    drop(sender);

    let paths = collector.join().expect("the walk collector does not panic");

    if paths.len() > bound {
        return Err(SwarmError::Validation(format!(
            "'{}' holds more than {WALK_FILE_COUNT_MAX} files; narrow the selection",
            directory.display(),
        )));
    }

    Ok(paths)
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;
    use crate::model::options::Options;

    #[test]
    fn a_walk_returns_included_files_only() {
        let root = std::env::temp_dir().join(format!("swarm-walk-{}", uuid::Uuid::new_v4()));

        fs::create_dir_all(root.join("nested")).expect("the test directory is creatable");
        fs::write(root.join("keep.rs"), "").expect("the test file is writable");
        fs::write(root.join("nested").join("keep.rs"), "").expect("the test file is writable");
        fs::write(root.join("drop.log"), "").expect("the test file is writable");

        let filter =
            GlobPathFilter::from_options(&Options::default()).expect("the defaults compile");

        let outcome = walk_files(&root, &filter, WalkOptions::GATHER);

        fs::remove_dir_all(&root).expect("the test directory is removable");

        let mut paths = outcome.expect("the walk succeeds");

        paths.sort();

        assert_eq!(paths.len(), 2);
        assert!(paths.iter().all(|path| path.ends_with("keep.rs")));
    }
}
