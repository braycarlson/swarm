use core::sync::atomic::{AtomicU32, Ordering};
use std::fs;

use rayon::iter::{IntoParallelIterator as _, ParallelIterator as _};

use crate::constants::TREE_NODE_COUNT_MAX;
use crate::model::error::{SwarmError, SwarmResult};
use crate::model::node::{self, FileNode, NodeKind, TREE_DEPTH_MAX};
use crate::services::filesystem::filter::GlobPathFilter;

#[derive(Default)]
struct LevelCounters {
    created: AtomicU32,
    failed: AtomicU32,
    loaded: AtomicU32,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct LoadProgress {
    pub directories: u32,
    pub failed: u32,
    pub nodes: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LoadStop {
    Bound,
    Cancelled,
    Finished,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LoadSummary {
    pub progress: LoadProgress,
    pub stop: LoadStop,
}

pub fn load_children(
    node: &mut FileNode,
    depth: u32,
    filter: &GlobPathFilter,
) -> SwarmResult<bool> {
    assert!(depth < TREE_DEPTH_MAX);

    if node.is_file() {
        return Ok(false);
    }

    node.children.clear();
    node.loaded = true;
    node.load_failed = false;

    if depth + 1 >= TREE_DEPTH_MAX {
        node.load_failed = true;

        return Err(SwarmError::Validation(format!(
            "'{}' is nested deeper than {TREE_DEPTH_MAX} levels",
            node.path.display(),
        )));
    }

    let loaded = fs::read_dir(&node.path)
        .map_err(SwarmError::from)
        .and_then(|entries| read_children(entries, filter));

    match loaded {
        Ok(children) => node.children = children,
        Err(error) => {
            node.load_failed = true;

            return Err(error);
        }
    }

    debug_assert!(node.loaded);
    debug_assert!(!node.load_failed);

    Ok(!node.children.is_empty())
}

pub fn load_tree(
    nodes: &mut [FileNode],
    filter: &GlobPathFilter,
    mut proceed: impl FnMut(LoadProgress) -> bool,
) -> LoadSummary {
    let mut progress = LoadProgress::default();

    node::visit(nodes, |_, _| progress.nodes += 1);

    let mut level: Vec<&mut FileNode> = nodes.iter_mut().collect();
    let mut depth: u32 = 0;

    while !level.is_empty() {
        assert!(depth < TREE_DEPTH_MAX);

        if !proceed(progress) {
            return LoadSummary {
                progress,
                stop: LoadStop::Cancelled,
            };
        }

        if depth + 1 >= TREE_DEPTH_MAX {
            return LoadSummary {
                progress,
                stop: LoadStop::Bound,
            };
        }

        if progress.nodes >= TREE_NODE_COUNT_MAX {
            return LoadSummary {
                progress,
                stop: LoadStop::Bound,
            };
        }

        let counters = LevelCounters::default();

        level = level
            .into_par_iter()
            .flat_map_iter(|current| {
                load_pending(current, depth, filter, &counters);

                current
                    .children
                    .iter_mut()
                    .filter(|child| child.is_directory())
            })
            .collect();

        progress.directories += counters.loaded.into_inner();
        progress.failed += counters.failed.into_inner();
        progress.nodes += counters.created.into_inner();
        depth += 1;
    }

    LoadSummary {
        progress,
        stop: LoadStop::Finished,
    }
}

fn load_pending(
    node: &mut FileNode,
    depth: u32,
    filter: &GlobPathFilter,
    counters: &LevelCounters,
) {
    if !node.is_directory() {
        return;
    }

    if node.loaded {
        return;
    }

    match load_children(node, depth, filter) {
        Ok(_) => {
            let child_count =
                u32::try_from(node.children.len()).expect("a directory listing fits in u32");

            counters.loaded.fetch_add(1, Ordering::Relaxed);
            counters.created.fetch_add(child_count, Ordering::Relaxed);
        }
        Err(_) => {
            counters.failed.fetch_add(1, Ordering::Relaxed);
        }
    }
}

fn read_children(entries: fs::ReadDir, filter: &GlobPathFilter) -> SwarmResult<Vec<FileNode>> {
    let mut directories = Vec::new();
    let mut files = Vec::new();

    for entry_result in entries {
        let entry = entry_result?;
        let path = entry.path();
        let is_directory = entry.file_type()?.is_dir();

        if !filter.admits_entry(&path, is_directory) {
            continue;
        }

        if is_directory {
            directories.push(FileNode::with_kind(path, NodeKind::Directory));
        } else {
            files.push(FileNode::with_kind(path, NodeKind::File));
        }
    }

    directories.sort_by(|left, right| left.name_lowercase.cmp(&right.name_lowercase));
    files.sort_by(|left, right| left.name_lowercase.cmp(&right.name_lowercase));

    let children_count = directories.len() + files.len();

    directories.append(&mut files);

    assert_eq!(directories.len(), children_count);
    debug_assert!(directories.is_sorted_by_key(|node| !node.is_directory()));

    Ok(directories)
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;
    use crate::model::options::Options;

    struct TemporaryTree {
        root: PathBuf,
    }

    impl TemporaryTree {
        fn new() -> Self {
            let root = std::env::temp_dir().join(format!("swarm-loader-{}", uuid::Uuid::new_v4()));

            fs::create_dir_all(root.join("beta").join("inner"))
                .expect("the test directory is creatable");

            fs::create_dir_all(root.join("alpha")).expect("the test directory is creatable");
            fs::write(root.join("zeta.rs"), "").expect("the test file is writable");
            fs::write(root.join("Aardvark.rs"), "").expect("the test file is writable");

            fs::write(root.join("beta").join("inner").join("deep.rs"), "")
                .expect("the test file is writable");

            Self { root }
        }
    }

    impl Drop for TemporaryTree {
        fn drop(&mut self) {
            fs::remove_dir_all(&self.root).expect("the test directory is removable");
        }
    }

    fn filter() -> GlobPathFilter {
        GlobPathFilter::from_options(&Options::default()).expect("the defaults compile")
    }

    #[test]
    fn directories_precede_files_and_each_group_is_sorted() {
        let tree = TemporaryTree::new();
        let mut root = FileNode::with_kind(tree.root.clone(), NodeKind::Directory);
        let has_children = load_children(&mut root, 0, &filter()).expect("the directory loads");
        let names: Vec<&str> = root.children.iter().map(|child| &*child.name).collect();

        assert!(has_children);
        assert_eq!(names, vec!["alpha", "beta", "Aardvark.rs", "zeta.rs"]);
    }

    #[test]
    fn a_missing_directory_is_marked_failed() {
        let mut node = FileNode::with_kind(
            PathBuf::from("/nonexistent/swarm/tree"),
            NodeKind::Directory,
        );

        assert!(load_children(&mut node, 0, &filter()).is_err());
        assert!(node.loaded);
        assert!(node.load_failed);
    }

    #[test]
    fn a_directory_at_the_depth_bound_refuses_to_load() {
        let tree = TemporaryTree::new();
        let mut root = FileNode::with_kind(tree.root.clone(), NodeKind::Directory);

        assert!(load_children(&mut root, TREE_DEPTH_MAX - 1, &filter()).is_err());
        assert!(root.load_failed);
        assert_eq!(root.children.len(), 0);
    }

    #[test]
    fn the_whole_tree_loads_level_by_level() {
        let tree = TemporaryTree::new();
        let mut nodes = vec![FileNode::with_kind(tree.root.clone(), NodeKind::Directory)];
        let mut levels: u32 = 0;

        let summary = load_tree(&mut nodes, &filter(), |_| {
            levels += 1;

            true
        });

        assert_eq!(summary.stop, LoadStop::Finished);
        assert_eq!(summary.progress.failed, 0);
        assert_eq!(summary.progress.directories, 4);
        assert!(levels >= 3);
        assert!(nodes[0].children[1].children[0].children[0].name.ends_with("deep.rs"));
    }

    #[test]
    fn a_declined_load_stops_before_loading() {
        let tree = TemporaryTree::new();
        let mut nodes = vec![FileNode::with_kind(tree.root.clone(), NodeKind::Directory)];
        let summary = load_tree(&mut nodes, &filter(), |_| false);

        assert_eq!(summary.stop, LoadStop::Cancelled);
        assert!(!nodes[0].loaded);
    }
}
