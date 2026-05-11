use std::cell::RefCell;
use std::path::Path;

use grep_regex::{RegexMatcher, RegexMatcherBuilder};
use grep_searcher::SearcherBuilder;
use grep_searcher::sinks::UTF8;
use rustc_hash::FxHashMap;
use tree_sitter::Parser;

use crate::services::skeleton::language::Language;

thread_local! {
    static SEARCHER: RefCell<grep_searcher::Searcher> = RefCell::new(
        SearcherBuilder::new()
            .binary_detection(grep_searcher::BinaryDetection::quit(b'\x00'))
            .build()
    );

    static PARSERS: RefCell<FxHashMap<Language, Parser>> = RefCell::new(FxHashMap::default());
}

pub fn build_content_matchers(patterns: &[String]) -> Vec<RegexMatcher> {
    patterns
        .iter()
        .filter_map(|pattern| {
            RegexMatcherBuilder::new()
                .case_insensitive(true)
                .build(&escape_regex(pattern))
                .ok()
        })
        .collect()
}

pub fn content_matches(path: &Path, matchers: &[RegexMatcher]) -> bool {
    if matchers.is_empty() {
        return true;
    }

    SEARCHER.with(|searcher| {
        let mut searcher = searcher.borrow_mut();

        for matcher in matchers {
            let mut found = false;

            let result = searcher.search_path(
                matcher,
                path,
                UTF8(|_, _| {
                    found = true;
                    Ok(false)
                }),
            );

            if result.is_err() || !found {
                return false;
            }
        }

        true
    })
}

pub fn symbol_matches(path: &Path, patterns: &[String]) -> bool {
    if patterns.is_empty() {
        return true;
    }

    let language = match Language::from_path(path) {
        Some(l) => l,
        None => return false,
    };

    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return false,
    };

    let symbols = extract_symbol_names(&content, language);

    patterns.iter().all(|pattern| {
        symbols.iter().any(|sym| sym.contains(pattern.as_str()))
    })
}

fn extract_symbol_names(content: &str, language: Language) -> Vec<String> {
    with_parser(language, |parser| {
        let tree = parser.parse(content, None)?;
        let root = tree.root_node();
        let mut names = Vec::new();

        collect_symbol_names(root, content, language, &mut names);

        Some(names)
    })
    .unwrap_or_default()
}

fn collect_symbol_names(
    node: tree_sitter::Node,
    source: &str,
    language: Language,
    names: &mut Vec<String>,
) {
    let kind = node.kind();

    let is_definition = language.definition_types().contains(&kind)
        || language.class_types().contains(&kind)
        || language.constant_types().contains(&kind);

    if is_definition {
        if let Some(name) = extract_node_name(node, source) {
            names.push(name.to_ascii_lowercase());
        }
    }

    let mut cursor = node.walk();

    for child in node.children(&mut cursor) {
        collect_symbol_names(child, source, language, names);
    }
}

fn extract_node_name(node: tree_sitter::Node, source: &str) -> Option<String> {
    if let Some(name_node) = node.child_by_field_name("name") {
        let text = &source[name_node.start_byte()..name_node.end_byte()];
        return Some(text.to_string());
    }

    let mut cursor = node.walk();

    for child in node.children(&mut cursor) {
        let child_kind = child.kind();

        if child_kind == "identifier"
            || child_kind == "type_identifier"
            || child_kind == "property_identifier"
        {
            let text = &source[child.start_byte()..child.end_byte()];
            return Some(text.to_string());
        }
    }

    None
}

fn with_parser<F, R>(language: Language, f: F) -> Option<R>
where
    F: FnOnce(&mut Parser) -> Option<R>,
{
    PARSERS.with(|parsers| {
        let mut map = parsers.borrow_mut();

        let parser = map.entry(language).or_insert_with(|| {
            let mut p = Parser::new();
            let _ = p.set_language(&language.grammar());
            p
        });

        f(parser)
    })
}

fn escape_regex(pattern: &str) -> String {
    let mut escaped = String::with_capacity(pattern.len() + 8);

    for c in pattern.chars() {
        if matches!(c, '\\' | '.' | '+' | '*' | '?' | '(' | ')' | '|' | '[' | ']' | '{' | '}' | '^' | '$') {
            escaped.push('\\');
        }
        escaped.push(c);
    }

    escaped
}
