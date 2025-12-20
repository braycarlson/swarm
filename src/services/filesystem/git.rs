use std::collections::HashMap;
use std::path::{Path, PathBuf};

use git2::{Repository, Status, StatusOptions};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum GitStatus {
    #[default]
    Unmodified,
    Added,
    Conflicted,
    Deleted,
    Modified,
    Renamed,
    Staged,
    Untracked,
}

impl GitStatus {
    pub fn has_diff(&self) -> bool {
        matches!(self, Self::Modified | Self::Added | Self::Staged | Self::Renamed)
    }
}

#[derive(Clone)]
pub struct GitService {
    statuses: HashMap<PathBuf, GitStatus>,
    repo_root: Option<PathBuf>,
}

impl Default for GitService {
    fn default() -> Self {
        Self::new()
    }
}

impl GitService {
    pub fn new() -> Self {
        Self {
            statuses: HashMap::new(),
            repo_root: None,
        }
    }

    pub fn refresh(&mut self, path: &Path) {
        self.statuses.clear();
        self.repo_root = None;

        let repo = match Self::find_repo(path) {
            Some(r) => r,
            None => return,
        };

        let workdir = match repo.workdir() {
            Some(w) => match dunce::canonicalize(w) {
                Ok(c) => c,
                Err(_) => w.to_path_buf(),
            },
            None => return,
        };

        self.repo_root = Some(workdir.clone());

        let mut opts = StatusOptions::new();

        opts.include_untracked(true)
            .recurse_untracked_dirs(true)
            .include_ignored(false);

        let statuses = match repo.statuses(Some(&mut opts)) {
            Ok(s) => s,
            Err(_) => return,
        };

        for entry in statuses.iter() {
            let status = entry.status();
            let git_status = Self::convert_status(status);

            if git_status == GitStatus::Unmodified {
                continue;
            }

            if let Some(entry_path) = entry.path() {
                let normalized = entry_path.replace('/', std::path::MAIN_SEPARATOR_STR);
                let full_path = workdir.join(&normalized);
                self.statuses.insert(full_path, git_status);
            }
        }
    }

    pub fn get_status(&self, path: &Path) -> GitStatus {
        if let Some(status) = self.statuses.get(path) {
            return *status;
        }

        GitStatus::Unmodified
    }

    pub fn get_original_content(&self, path: &Path) -> Option<String> {
        let repo_root = self.repo_root.as_ref()?;
        let repo = Repository::open(repo_root).ok()?;

        let canonical = dunce::canonicalize(path).ok()?;
        let relative_path = canonical.strip_prefix(repo_root).ok()?;
        let relative_str = relative_path.to_str()?;

        let relative_unix = relative_str.replace('\\', "/");

        let head = repo.head().ok()?;
        let tree = head.peel_to_tree().ok()?;
        let entry = tree.get_path(Path::new(&relative_unix)).ok()?;
        let blob = repo.find_blob(entry.id()).ok()?;

        if blob.is_binary() {
            return None;
        }

        String::from_utf8(blob.content().to_vec()).ok()
    }

    pub fn is_in_repo(&self) -> bool {
        self.repo_root.is_some()
    }

    pub fn has_changes(&self) -> bool {
        self.statuses.values().any(|s| s.has_diff())
    }

    fn find_repo(path: &Path) -> Option<Repository> {
        let canonical = dunce::canonicalize(path).ok()?;

        let start = if canonical.is_file() {
            canonical.parent()?
        } else {
            &canonical
        };

        Repository::discover(start).ok()
    }

    fn convert_status(status: Status) -> GitStatus {
        if status.contains(Status::CONFLICTED) {
            return GitStatus::Conflicted;
        }

        if status.contains(Status::INDEX_NEW) {
            return GitStatus::Staged;
        }

        if status.contains(Status::INDEX_MODIFIED)
            || status.contains(Status::INDEX_RENAMED)
            || status.contains(Status::INDEX_TYPECHANGE)
        {
            return GitStatus::Staged;
        }

        if status.contains(Status::WT_NEW) {
            return GitStatus::Untracked;
        }

        if status.contains(Status::WT_MODIFIED) || status.contains(Status::WT_TYPECHANGE) {
            return GitStatus::Modified;
        }

        if status.contains(Status::WT_DELETED) || status.contains(Status::INDEX_DELETED) {
            return GitStatus::Deleted;
        }

        if status.contains(Status::WT_RENAMED) {
            return GitStatus::Renamed;
        }

        GitStatus::Unmodified
    }
}
