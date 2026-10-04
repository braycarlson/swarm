use std::fs;
use std::path::Path;

use grep_regex::{RegexMatcher, RegexMatcherBuilder};
use grep_searcher::sinks::UTF8;
use grep_searcher::{BinaryDetection, Searcher, SearcherBuilder};
use tree_sitter::Node;

use crate::model::node::Visit;
use crate::model::query::PATTERN_COUNT_MAX;
use crate::services::parser::{Parsers, walk_subtree};
use crate::services::skeleton::language::Language;

const IDENTIFIER_KINDS: [&str; 3] = ["identifier", "property_identifier", "type_identifier"];

pub struct ContentMatchers {
    matcher: Option<RegexMatcher>,
    patterns: Vec<String>,
}

impl ContentMatchers {
    pub fn is_empty(&self) -> bool {
        self.patterns.is_empty()
    }

    pub fn new(patterns: &[String]) -> Self {
        assert!(patterns.len() <= PATTERN_COUNT_MAX);

        if patterns.is_empty() {
            return Self {
                matcher: None,
                patterns: Vec::new(),
            };
        }

        let alternation = patterns
            .iter()
            .map(|pattern| escape_regex(pattern))
            .collect::<Vec<String>>()
            .join("|");

        let matcher = RegexMatcherBuilder::new()
            .case_insensitive(true)
            .build(&alternation)
            .expect("an alternation of escaped literals compiles");

        let lowercase: Vec<String> = patterns
            .iter()
            .map(|pattern| pattern.to_ascii_lowercase())
            .collect();

        debug_assert_eq!(lowercase.len(), patterns.len());

        Self {
            matcher: Some(matcher),
            patterns: lowercase,
        }
    }
}

#[derive(Default)]
pub struct SearchScratch {
    line: String,
    parsers: Parsers,
    searcher: Option<Searcher>,
}

pub fn content_matches(
    path: &Path,
    matchers: &ContentMatchers,
    scratch: &mut SearchScratch,
) -> bool {
    let Some(matcher) = matchers.matcher.as_ref() else {
        debug_assert_eq!(matchers.patterns.len(), 0);

        return true;
    };

    let full = mask_full(matchers.patterns.len());
    let lowercase = &mut scratch.line;
    let mut satisfied: u64 = 0;

    let searcher = scratch.searcher.get_or_insert_with(|| {
        SearcherBuilder::new()
            .binary_detection(BinaryDetection::quit(b'\x00'))
            .build()
    });

    let outcome = searcher.search_path(
        matcher,
        path,
        UTF8(|_, line| {
            lowercase.clear();
            lowercase.push_str(line);
            lowercase.make_ascii_lowercase();

            satisfied = mark_satisfied(&matchers.patterns, lowercase, satisfied);

            Ok(satisfied != full)
        }),
    );

    if outcome.is_err() {
        return false;
    }

    debug_assert_eq!(satisfied & !full, 0);

    satisfied == full
}

fn escape_regex(pattern: &str) -> String {
    let mut escaped = String::with_capacity(pattern.len() * 2);

    for character in pattern.chars() {
        if matches!(
            character,
            '$' | '(' | ')' | '*' | '+' | '.' | '?' | '[' | '\\' | ']' | '^' | '{' | '|' | '}',
        ) {
            escaped.push('\\');
        }

        escaped.push(character);
    }

    debug_assert!(escaped.len() >= pattern.len());

    escaped
}

fn is_definition(kind: &str, language: Language) -> bool {
    language.definition_types().contains(&kind)
        || language.class_types().contains(&kind)
        || language.constant_types().contains(&kind)
}

fn mark_satisfied(patterns: &[String], text_lowercase: &str, satisfied: u64) -> u64 {
    debug_assert!(patterns.len() <= PATTERN_COUNT_MAX);

    let mut found: u64 = 0;

    for (index, pattern) in patterns.iter().enumerate() {
        let bit = 1_u64 << index;

        if satisfied & bit != 0 {
            continue;
        }

        if text_lowercase.contains(pattern.as_str()) {
            found |= bit;
        }
    }

    debug_assert_eq!(found & satisfied, 0);

    satisfied | found
}

fn mask_full(count: usize) -> u64 {
    assert!(count > 0);
    assert!(count <= PATTERN_COUNT_MAX);

    u64::MAX >> (PATTERN_COUNT_MAX - count)
}

fn node_name<'source>(node: Node<'_>, source: &'source str) -> Option<&'source str> {
    if let Some(name) = node.child_by_field_name("name") {
        return name.utf8_text(source.as_bytes()).ok();
    }

    let mut cursor = node.walk();

    let identifier = node
        .children(&mut cursor)
        .find(|child| IDENTIFIER_KINDS.contains(&child.kind()))?;

    identifier.utf8_text(source.as_bytes()).ok()
}

pub fn symbol_matches(path: &Path, patterns: &[String], scratch: &mut SearchScratch) -> bool {
    if patterns.is_empty() {
        return true;
    }

    let Some(language) = Language::from_path(path) else {
        return false;
    };

    let Ok(content) = fs::read_to_string(path) else {
        return false;
    };

    let Some(tree) = scratch.parsers.parse(language, &content) else {
        return false;
    };

    let full = mask_full(patterns.len());
    let lowercase = &mut scratch.line;
    let mut satisfied: u64 = 0;

    walk_subtree(tree.root_node(), |node| {
        if satisfied == full {
            return Visit::Stop;
        }

        if !is_definition(node.kind(), language) {
            return Visit::Descend;
        }

        if let Some(name) = node_name(node, &content) {
            lowercase.clear();
            lowercase.push_str(name);
            lowercase.make_ascii_lowercase();

            satisfied = mark_satisfied(patterns, lowercase, satisfied);
        }

        Visit::Descend
    });

    satisfied == full
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temporary_path(extension: &str) -> std::path::PathBuf {
        std::env::temp_dir().join(format!("swarm-search-{}.{extension}", uuid::Uuid::new_v4()))
    }

    #[test]
    fn an_empty_pattern_list_matches_everything() {
        let matchers = ContentMatchers::new(&[]);
        let mut scratch = SearchScratch::default();

        assert!(matchers.is_empty());
        assert!(content_matches(Path::new("/nonexistent"), &matchers, &mut scratch));
    }

    #[test]
    fn regex_metacharacters_are_escaped() {
        assert_eq!(escape_regex("a.b*c"), "a\\.b\\*c");
        assert_eq!(escape_regex("a|b"), "a\\|b");
        assert_eq!(escape_regex("plain"), "plain");
    }

    #[test]
    fn satisfaction_tracking_counts_each_pattern_once() {
        let patterns = vec!["alpha".to_owned(), "beta".to_owned()];
        let full = mask_full(patterns.len());
        let first = mark_satisfied(&patterns, "alpha alpha", 0);

        assert_eq!(first, 0b01);

        let second = mark_satisfied(&patterns, "beta", first);

        assert_eq!(second, full);
    }

    #[test]
    fn the_full_mask_covers_exactly_the_patterns() {
        assert_eq!(mask_full(1), 0b1);
        assert_eq!(mask_full(3), 0b111);
        assert_eq!(mask_full(PATTERN_COUNT_MAX), u64::MAX);
    }

    #[test]
    fn every_pattern_must_appear_somewhere_in_the_file() {
        let path = temporary_path("txt");

        fs::write(&path, "first line has Alpha\nsecond has beta\n")
            .expect("the test file is writable");

        let mut scratch = SearchScratch::default();
        let both = ContentMatchers::new(&["alpha".to_owned(), "BETA".to_owned()]);
        let missing = ContentMatchers::new(&["alpha".to_owned(), "gamma".to_owned()]);
        let matched_both = content_matches(&path, &both, &mut scratch);
        let matched_missing = content_matches(&path, &missing, &mut scratch);

        fs::remove_file(&path).expect("the test file is removable");

        assert!(matched_both);
        assert!(!matched_missing);
    }

    #[test]
    fn symbols_match_definition_names() {
        let path = temporary_path("rs");

        fs::write(&path, "struct Widget;\nfn render_widget() {}\n")
            .expect("the test file is writable");

        let mut scratch = SearchScratch::default();

        let found = symbol_matches(
            &path,
            &["widget".to_owned(), "render".to_owned()],
            &mut scratch,
        );

        let absent = symbol_matches(&path, &["missing".to_owned()], &mut scratch);

        fs::remove_file(&path).expect("the test file is removable");

        assert!(found);
        assert!(!absent);
    }
}
