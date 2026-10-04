use tree_sitter::{Node, Parser, Tree, TreeCursor};

use crate::model::node::Visit;
use crate::services::skeleton::language::Language;

const WALK_STEP_BYTES_SLACK: usize = 64;
const WALK_STEP_FACTOR: usize = 8;

#[derive(Default)]
pub struct Parsers {
    parsers: [Option<Parser>; Language::COUNT],
}

impl Parsers {
    pub fn parse(&mut self, language: Language, source: &str) -> Option<Tree> {
        let parser = self.parsers[language.index()].get_or_insert_with(|| {
            let mut parser = Parser::new();
            let outcome = parser.set_language(&language.grammar());

            assert!(
                outcome.is_ok(),
                "the {} grammar is incompatible with tree-sitter",
                language.name(),
            );

            parser
        });

        let tree = parser.parse(source, None)?;

        debug_assert!(tree.root_node().end_byte() <= source.len());

        Some(tree)
    }
}

fn climb(cursor: &mut TreeCursor<'_>, root_id: usize) -> bool {
    while cursor.node().id() != root_id {
        if cursor.goto_next_sibling() {
            return true;
        }

        if !cursor.goto_parent() {
            return false;
        }
    }

    false
}

pub fn walk_subtree<'tree>(root: Node<'tree>, mut visitor: impl FnMut(Node<'tree>) -> Visit) {
    let step_count_max = WALK_STEP_FACTOR * (root.end_byte() + WALK_STEP_BYTES_SLACK);
    let root_id = root.id();
    let mut cursor = root.walk();
    let mut step_count: usize = 0;

    while step_count < step_count_max {
        step_count += 1;

        match visitor(cursor.node()) {
            Visit::Descend => {
                if cursor.goto_first_child() {
                    continue;
                }
            }
            Visit::Skip => {}
            Visit::Stop => return,
        }

        if !climb(&mut cursor, root_id) {
            return;
        }
    }

    unreachable!("the subtree walk exceeded its bound of {step_count_max} steps");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_walk_visits_every_node_of_a_subtree_once() {
        let mut parsers = Parsers::default();
        let source = "fn one() {}\nfn two() {}\n";

        let tree = parsers
            .parse(Language::Rust, source)
            .expect("the source parses");

        let mut count: u32 = 0;
        let mut functions: u32 = 0;

        walk_subtree(tree.root_node(), |node| {
            count += 1;

            if node.kind() == "function_item" {
                functions += 1;
            }

            Visit::Descend
        });

        assert!(count > 2);
        assert_eq!(functions, 2);
    }

    #[test]
    fn a_walk_that_declines_to_descend_stays_shallow() {
        let mut parsers = Parsers::default();

        let tree = parsers
            .parse(Language::Rust, "fn one() {}\n")
            .expect("the source parses");

        let mut count: u32 = 0;

        walk_subtree(tree.root_node(), |_| {
            count += 1;

            Visit::Skip
        });

        assert_eq!(count, 1);
    }

    #[test]
    fn a_walk_does_not_leave_its_subtree() {
        let mut parsers = Parsers::default();

        let tree = parsers
            .parse(Language::Rust, "fn one() {}\nfn two() {}\n")
            .expect("the source parses");

        let first = tree.root_node().child(0).expect("the first item exists");
        let mut functions: u32 = 0;

        walk_subtree(first, |node| {
            if node.kind() == "function_item" {
                functions += 1;
            }

            Visit::Descend
        });

        assert_eq!(functions, 1);
    }
}
