use alloc::sync::Arc;
use std::path::{MAIN_SEPARATOR, MAIN_SEPARATOR_STR, Path, PathBuf};

use git2::{Repository, Status, StatusOptions};
use rustc_hash::FxHashMap;

use crate::model::git::GitStatus;

#[derive(Clone, Default)]
pub struct GitService {
    repository_root: Option<Arc<Path>>,
    statuses: Arc<FxHashMap<PathBuf, GitStatus>>,
}

impl GitService {
    fn convert_status(status: Status) -> GitStatus {
        if status.contains(Status::CONFLICTED) {
            return GitStatus::Conflicted;
        }

        if status.contains(Status::INDEX_NEW) {
            return GitStatus::Added;
        }

        if status
            .intersects(Status::INDEX_MODIFIED | Status::INDEX_RENAMED | Status::INDEX_TYPECHANGE)
        {
            return GitStatus::Staged;
        }

        if status.contains(Status::WT_NEW) {
            return GitStatus::Untracked;
        }

        if status.intersects(Status::WT_MODIFIED | Status::WT_TYPECHANGE) {
            return GitStatus::Modified;
        }

        if status.intersects(Status::INDEX_DELETED | Status::WT_DELETED) {
            return GitStatus::Deleted;
        }

        if status.contains(Status::WT_RENAMED) {
            return GitStatus::Renamed;
        }

        GitStatus::Unmodified
    }

    fn find_repository(path: &Path) -> Option<Repository> {
        let canonical = dunce::canonicalize(path).ok()?;

        let start = if canonical.is_file() {
            canonical.parent()?
        } else {
            &canonical
        };

        Repository::discover(start).ok()
    }

    fn full_path(root: &Path, relative: &str) -> PathBuf {
        if MAIN_SEPARATOR == '/' {
            return root.join(relative);
        }

        root.join(relative.replace('/', MAIN_SEPARATOR_STR))
    }

    pub fn open_repository(&self) -> Option<Repository> {
        Repository::open(self.repository_root.as_deref()?).ok()
    }

    pub fn original_content(&self, repository: &Repository, path: &Path) -> Option<String> {
        let root = self.repository_root.as_deref()?;
        let canonical = dunce::canonicalize(path).ok()?;
        let relative = canonical.strip_prefix(root).ok()?;

        debug_assert!(relative.is_relative());

        let relative_text = relative.to_str()?.replace('\\', "/");
        let head = repository.head().ok()?;
        let tree = head.peel_to_tree().ok()?;
        let entry = tree.get_path(Path::new(&relative_text)).ok()?;
        let blob = repository.find_blob(entry.id()).ok()?;

        if blob.is_binary() {
            return None;
        }

        String::from_utf8(blob.content().to_vec()).ok()
    }

    pub fn refresh(&mut self, path: &Path) {
        self.repository_root = None;
        self.statuses = Arc::default();

        let Some(repository) = Self::find_repository(path) else {
            return;
        };

        let Some(workdir) = repository.workdir() else {
            return;
        };

        let root = dunce::canonicalize(workdir).unwrap_or_else(|_| workdir.to_path_buf());
        let mut options = StatusOptions::new();

        options
            .exclude_submodules(true)
            .include_ignored(false)
            .include_untracked(true)
            .recurse_untracked_dirs(true);

        let Ok(entries) = repository.statuses(Some(&mut options)) else {
            self.repository_root = Some(Arc::from(root));

            return;
        };

        let mut statuses = FxHashMap::default();

        for entry in entries.iter() {
            let status = Self::convert_status(entry.status());

            if status == GitStatus::Unmodified {
                continue;
            }

            let Some(relative) = entry.path() else {
                continue;
            };

            let replaced = statuses.insert(Self::full_path(&root, relative), status);

            debug_assert!(replaced.is_none());
        }

        debug_assert!(statuses.values().all(|status| *status != GitStatus::Unmodified));

        self.repository_root = Some(Arc::from(root));
        self.statuses = Arc::new(statuses);
    }

    pub fn status(&self, path: &Path) -> GitStatus {
        self.statuses.get(path).copied().unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn index_states_take_precedence_over_the_worktree() {
        assert_eq!(GitService::convert_status(Status::INDEX_NEW), GitStatus::Added);

        assert_eq!(
            GitService::convert_status(Status::INDEX_MODIFIED | Status::WT_MODIFIED),
            GitStatus::Staged,
        );

        assert_eq!(GitService::convert_status(Status::WT_NEW), GitStatus::Untracked);
        assert_eq!(GitService::convert_status(Status::WT_MODIFIED), GitStatus::Modified);
        assert_eq!(GitService::convert_status(Status::CURRENT), GitStatus::Unmodified);
    }

    #[test]
    fn a_path_outside_any_repository_has_no_status() {
        let mut service = GitService::default();

        service.refresh(Path::new("/"));

        assert_eq!(service.status(Path::new("/etc/hosts")), GitStatus::Unmodified);
    }
}
