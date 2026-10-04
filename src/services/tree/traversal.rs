use core::slice;
use std::path::PathBuf;

use rustc_hash::FxHashSet;

use crate::model::node::{self, FileNode, TREE_DEPTH_MAX, Visit};
use crate::model::query::ParsedQuery;
use crate::services::filesystem::filter::GlobPathFilter;
use crate::services::filesystem::git::GitService;
use crate::services::tree::{loader, query};

pub struct PropagateRequest<'request> {
    pub checked: bool,
    pub depth: u32,
    pub filter: &'request GlobPathFilter,
    pub git: Option<&'request GitService>,
    pub matches: Option<&'request FxHashSet<PathBuf>>,
    pub query: Option<&'request ParsedQuery>,
}

pub fn expand_checked(nodes: &mut [FileNode], filter: &GlobPathFilter) -> u32 {
    let mut failed_count: u32 = 0;

    node::walk_mut(nodes, |current, depth| {
        if !current.is_directory() {
            return Visit::Skip;
        }

        if current.loaded {
            return Visit::Descend;
        }

        if !current.checked {
            return Visit::Skip;
        }

        if loader::load_children(current, depth, filter).is_err() {
            failed_count += 1;
        }

        Visit::Descend
    });

    failed_count
}

fn load_subtree(node: &mut FileNode, depth_base: u32, filter: &GlobPathFilter) -> u32 {
    assert!(depth_base < TREE_DEPTH_MAX);

    let mut failed_count: u32 = 0;

    node::walk_mut(slice::from_mut(node), |current, depth| {
        if !current.is_directory() {
            return Visit::Skip;
        }

        if !current.loaded {
            if loader::load_children(current, depth_base + depth, filter).is_err() {
                failed_count += 1;
            }
        }

        Visit::Descend
    });

    failed_count
}

pub fn propagate_checked(node: &mut FileNode, request: &PropagateRequest<'_>) -> u32 {
    let failed_count = load_subtree(node, request.depth, request.filter);

    let Some(query) = request.query.filter(|query| !query.is_empty()) else {
        node::visit_mut(slice::from_mut(node), |current, _| {
            current.checked = request.checked;
        });

        return failed_count;
    };

    let computed;

    let matches = if let Some(precomputed) = request.matches {
        precomputed
    } else {
        computed = query::matching_paths(slice::from_ref(node), query, request.git, request.depth);

        &computed
    };

    node.checked = request.checked;

    node::visit_mut(slice::from_mut(node), |current, depth| {
        if depth == 0 {
            return;
        }

        if matches.contains(&current.path) {
            current.checked = request.checked;
        }
    });

    failed_count
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;

    use super::*;
    use crate::model::node::NodeKind;
    use crate::model::options::Options;

    struct TemporaryTree {
        root: PathBuf,
    }

    impl TemporaryTree {
        fn new() -> Self {
            let root =
                std::env::temp_dir().join(format!("swarm-traversal-{}", uuid::Uuid::new_v4()));

            fs::create_dir_all(root.join("inner")).expect("the test directory is creatable");
            fs::write(root.join("top.rs"), "").expect("the test file is writable");
            fs::write(root.join("inner").join("deep.py"), "").expect("the test file is writable");

            Self { root }
        }
    }

    impl Drop for TemporaryTree {
        fn drop(&mut self) {
            fs::remove_dir_all(&self.root).expect("the test directory is removable");
        }
    }

    fn checked_files(node: &FileNode) -> Vec<String> {
        let mut names = Vec::new();

        node::visit(slice::from_ref(node), |current, _| {
            if current.is_file() {
                if current.checked {
                    names.push(current.name.to_string());
                }
            }
        });

        names.sort();

        names
    }

    #[test]
    fn propagation_loads_and_checks_the_whole_subtree() {
        let tree = TemporaryTree::new();

        let filter =
            GlobPathFilter::from_options(&Options::default()).expect("the defaults compile");

        let mut root = FileNode::with_kind(tree.root.clone(), NodeKind::Directory);

        let request = PropagateRequest {
            checked: true,
            depth: 0,
            filter: &filter,
            git: None,
            matches: None,
            query: None,
        };

        let failed_count = propagate_checked(&mut root, &request);

        assert_eq!(failed_count, 0);
        assert_eq!(checked_files(&root), vec!["deep.py".to_owned(), "top.rs".to_owned()]);
    }

    #[test]
    fn filtered_propagation_checks_only_matches() {
        let tree = TemporaryTree::new();

        let filter =
            GlobPathFilter::from_options(&Options::default()).expect("the defaults compile");

        let query = ParsedQuery::parse("ext:py");
        let mut root = FileNode::with_kind(tree.root.clone(), NodeKind::Directory);

        let request = PropagateRequest {
            checked: true,
            depth: 0,
            filter: &filter,
            git: None,
            matches: None,
            query: Some(&query),
        };

        let failed_count = propagate_checked(&mut root, &request);

        assert_eq!(failed_count, 0);
        assert_eq!(checked_files(&root), vec!["deep.py".to_owned()]);
    }

    #[test]
    fn expansion_loads_only_checked_directories() {
        let tree = TemporaryTree::new();

        let filter =
            GlobPathFilter::from_options(&Options::default()).expect("the defaults compile");

        let mut nodes = vec![FileNode::with_kind(tree.root.clone(), NodeKind::Directory)];

        assert_eq!(expand_checked(&mut nodes, &filter), 0);
        assert!(!nodes[0].loaded);

        nodes[0].checked = true;

        assert_eq!(expand_checked(&mut nodes, &filter), 0);
        assert!(nodes[0].loaded);
        assert!(!nodes[0].children[0].loaded);
    }
}
