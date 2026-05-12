use std::cell::RefCell;
use std::fmt::Write;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use ignore::WalkBuilder;
use rayon::prelude::*;
use rustc_hash::FxHashMap;
use tree_sitter::{Node, Parser};

use crate::model::error::SwarmResult;
use crate::model::options::Options;
use crate::services::filesystem::filter::{GlobPathFilter, PathFilter};

use super::language::Language;

#[derive(Clone, Debug)]
pub struct SkeletonStats {
    pub files_count: usize,
    pub line_count: usize,
    pub token_count: usize,
}

#[derive(Clone)]
pub struct SkeletonGenerator;

impl Default for SkeletonGenerator {
    fn default() -> Self {
        Self::new()
    }
}

thread_local! {
    static PARSERS: RefCell<FxHashMap<Language, Parser>> = RefCell::new(FxHashMap::default());
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

impl SkeletonGenerator {
    pub fn new() -> Self {
        Self
    }

    pub fn generate(
        &self,
        paths: &[String],
        options: &Options,
    ) -> SwarmResult<(String, SkeletonStats)> {
        let filter: Arc<dyn PathFilter> = Arc::new(GlobPathFilter::from_options(options)?);

        let mut all_paths = Vec::new();

        for string_path in paths {
            let path = Path::new(string_path);

            if path.is_dir() {
                let walked = Self::walk_parallel(path, &filter);
                all_paths.extend(walked);
            } else if path.is_file() {
                all_paths.push(path.to_path_buf());
            }
        }

        let mut files: Vec<(String, String)> = all_paths
            .par_iter()
            .filter_map(|path| process_file(path))
            .collect();

        files.sort_by(|a, b| a.0.cmp(&b.0));

        let approximate_capacity: usize = files.iter()
            .map(|(p, s)| p.len() + s.len() + 4)
            .sum();
        let mut output = String::with_capacity(approximate_capacity);

        for (path, skeleton) in &files {
            let _ = writeln!(output, "[{}]", path);
            output.push_str(skeleton);
            output.push('\n');
        }

        let line_count = memchr::memchr_iter(b'\n', output.as_bytes()).count();

        let stats = SkeletonStats {
            files_count: files.len(),
            line_count,
            token_count: estimate_skeleton_tokens(&output),
        };

        Ok((output, stats))
    }

    fn walk_parallel(directory: &Path, filter: &Arc<dyn PathFilter>) -> Vec<PathBuf> {
        let (sender, receiver) = std::sync::mpsc::channel::<PathBuf>();

        WalkBuilder::new(directory)
            .hidden(true)
            .git_ignore(true)
            .git_global(true)
            .git_exclude(true)
            .build_parallel()
            .run(|| {
                let sender = sender.clone();
                let filter = Arc::clone(filter);
                let mut local: Vec<PathBuf> = Vec::new();

                Box::new(move |result| {
                    match result {
                        Ok(entry) => {
                            if !filter.should_include(entry.path()) {
                                if entry.file_type().is_some_and(|type_file| type_file.is_dir()) {
                                    if !local.is_empty() {
                                        for path in local.drain(..) {
                                            let _ = sender.send(path);
                                        }
                                    }
                                    return ignore::WalkState::Skip;
                                }
                                return ignore::WalkState::Continue;
                            }

                            if entry.file_type().is_some_and(|type_file| type_file.is_file()) {
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

fn process_file(path: &Path) -> Option<(String, String)> {
    let language = Language::from_path(path)?;
    let content = fs::read_to_string(path).ok()?;
    let skeleton = extract_skeleton(&content, language)?;

    if skeleton.trim().is_empty() {
        return None;
    }

    Some((path.display().to_string(), skeleton))
}

const INDENT_CACHE: &str = "                                                                                                                                ";

fn indent(depth: usize) -> &'static str {
    let end = (depth * 4).min(INDENT_CACHE.len());
    &INDENT_CACHE[..end]
}

fn node_text<'a>(node: Node, source: &'a str) -> &'a str {
    &source[node.start_byte()..node.end_byte()]
}

#[derive(Clone, Copy, Eq, PartialEq)]
enum NodeCategory {
    Class,
    Constant,
    Definition,
    Import,
    Wrapper,
}

fn classify_node(kind: &str, language: Language) -> Option<NodeCategory> {
    if language.import_types().contains(&kind) {
        Some(NodeCategory::Import)
    } else if language.wrapper_types().contains(&kind) {
        Some(NodeCategory::Wrapper)
    } else if language.class_types().contains(&kind) {
        Some(NodeCategory::Class)
    } else if language.definition_types().contains(&kind) {
        Some(NodeCategory::Definition)
    } else if language.constant_types().contains(&kind) {
        Some(NodeCategory::Constant)
    } else {
        None
    }
}

fn find_body<'a>(node: Node<'a>, language: Language) -> Option<Node<'a>> {
    if let Some(body) = node.child_by_field_name(language.body_field()) {
        return Some(body);
    }

    let kinds = language.body_kinds();

    if kinds.is_empty() {
        return None;
    }

    let mut cursor = node.walk();

    for child in node.children(&mut cursor) {
        if kinds.contains(&child.kind()) {
            return Some(child);
        }
    }

    None
}

fn extract_skeleton(content: &str, language: Language) -> Option<String> {
    with_parser(language, |parser| {
        let tree = parser.parse(content, None)?;
        let root = tree.root_node();
        let mut output = String::new();
        let mut cursor = root.walk();

        for child in root.children(&mut cursor) {
            let kind = child.kind();

            match classify_node(kind, language) {
                Some(NodeCategory::Import) => {
                    extract_import(&mut output, child, content);
                }
                Some(NodeCategory::Definition) => {
                    extract_definition(&mut output, child, content, language, 0);
                }
                Some(NodeCategory::Class) => {
                    extract_class(&mut output, child, content, language, 0);
                }
                Some(NodeCategory::Constant) => {
                    extract_constant(&mut output, child, content, language, 0);
                }
                Some(NodeCategory::Wrapper) => {
                    extract_wrapper(&mut output, child, content, language, 0);
                }
                None => {}
            }
        }

        let new_length = output.trim_end().len();
        output.truncate(new_length);

        if output.is_empty() {
            return None;
        }

        output.push('\n');
        Some(output)
    })
}

fn extract_import(output: &mut String, node: Node, source: &str) {
    let text = node_text(node, source);
    let _ = writeln!(output, "{}", text);
}

fn extract_definition(
    output: &mut String,
    node: Node,
    source: &str,
    language: Language,
    depth: usize,
) {
    let indentation = indent(depth);

    if let Some(body) = find_body(node, language) {
        let signature = source[node.start_byte()..body.start_byte()].trim_end();

        for line in signature.lines() {
            if line.trim().is_empty() {
                continue;
            }

            let _ = writeln!(output, "{}{}", indentation, line);
        }

        let _ = writeln!(output, "{}{}", indentation, language.ellipsis());
    } else {
        let text = node_text(node, source);

        for line in text.lines() {
            if line.trim().is_empty() {
                continue;
            }

            let _ = writeln!(output, "{}{}", indentation, line);
        }
    }
}

fn extract_class(
    output: &mut String,
    node: Node,
    source: &str,
    language: Language,
    depth: usize,
) {
    let indentation = indent(depth);

    if let Some(body) = find_body(node, language) {
        let signature = source[node.start_byte()..body.start_byte()].trim_end();

        match language {
            Language::Python => {
                let _ = writeln!(output, "{}{}", indentation, signature);
                extract_class_body(output, body, source, language, depth + 1);
            }
            Language::Css => {
                if let Some(collapsed) = try_collapse_css_body(body, source, language) {
                    let _ = writeln!(output, "{}{} {{ {} }}", indentation, signature, collapsed);
                } else {
                    let _ = writeln!(output, "{}{} {{", indentation, signature);
                    extract_class_body(output, body, source, language, depth + 1);
                    let _ = writeln!(output, "{}}}", indentation);
                }
            }
            _ => {
                let _ = writeln!(output, "{}{} {{", indentation, signature);
                extract_class_body(output, body, source, language, depth + 1);
                let _ = writeln!(output, "{}}}", indentation);
            }
        }
    } else {
        let text = node_text(node, source);

        for line in text.lines() {
            if line.trim().is_empty() {
                continue;
            }

            let _ = writeln!(output, "{}{}", indentation, line);
        }
    }
}

fn try_collapse_css_body(body: Node, source: &str, language: Language) -> Option<String> {
    if !matches!(language, Language::Css) {
        return None;
    }

    let mut cursor = body.walk();
    let mut matching_iter = body.children(&mut cursor)
        .filter(|child| {
            let kind = child.kind();
            language.definition_types().contains(&kind)
                || language.class_types().contains(&kind)
        });

    let first = matching_iter.next()?;

    if matching_iter.next().is_some() {
        return None;
    }

    let child = first;
    let kind = child.kind();

    if language.definition_types().contains(&kind) {
        if let Some(body) = find_body(child, language) {
            let signature = source[child.start_byte()..body.start_byte()].trim_end();
            return Some(format!("{}{}", signature, language.ellipsis()));
        }
    }

    if language.class_types().contains(&kind) {
        if let Some(child_body) = find_body(child, language) {
            let signature = source[child.start_byte()..child_body.start_byte()].trim_end();

            if let Some(collapsed) = try_collapse_css_body(child_body, source, language) {
                return Some(format!("{} {{ {} }}", signature, collapsed));
            }
        }
    }

    None
}

fn extract_class_body(
    output: &mut String,
    body: Node,
    source: &str,
    language: Language,
    depth: usize,
) {
    let indentation = indent(depth);
    let mut cursor = body.walk();

    let has_skeleton_content = body.children(&mut cursor).any(|child| {
        let kind = child.kind();
        language.definition_types().contains(&kind)
            || language.class_types().contains(&kind)
            || language.wrapper_types().contains(&kind)
    });

    if !has_skeleton_content {
        let _ = writeln!(output, "{}...", indentation);
        return;
    }

    let mut cursor2 = body.walk();

    for child in body.children(&mut cursor2) {
        let kind = child.kind();

        if language.definition_types().contains(&kind) {
            extract_definition(output, child, source, language, depth);
        } else if language.class_types().contains(&kind) {
            extract_class(output, child, source, language, depth);
        } else if language.wrapper_types().contains(&kind) {
            extract_wrapper(output, child, source, language, depth);
        }
    }
}

fn extract_wrapper(
    output: &mut String,
    node: Node,
    source: &str,
    language: Language,
    depth: usize,
) {
    let indentation = indent(depth);
    let mut cursor = node.walk();

    for child in node.children(&mut cursor) {
        let kind = child.kind();

        if kind == "decorator" || kind == "export" {
            let text = node_text(child, source);
            let _ = writeln!(output, "{}{}", indentation, text);
        } else if language.definition_types().contains(&kind) {
            extract_definition(output, child, source, language, depth);
        } else if language.class_types().contains(&kind) {
            extract_class(output, child, source, language, depth);
        } else if language.wrapper_types().contains(&kind) {
            extract_wrapper(output, child, source, language, depth);
        }
    }
}

fn extract_constant(
    output: &mut String,
    node: Node,
    source: &str,
    language: Language,
    depth: usize,
) {
    if has_nested_definitions(node, language) {
        extract_constant_with_definitions(output, node, source, language, depth);
        return;
    }

    let indentation = indent(depth);
    let text = node_text(node, source);
    let first_line = text.lines().next().unwrap_or("");

    if text.lines().count() > 1 {
        append_collapsed_assignment(output, node, source, depth);
    } else {
        let _ = writeln!(output, "{}{}", indentation, first_line);
    }
}

fn has_nested_definitions(node: Node, language: Language) -> bool {
    let mut cursor = node.walk();

    for child in node.children(&mut cursor) {
        let kind = child.kind();

        if language.definition_types().contains(&kind) {
            return true;
        }

        if child.child_count() > 0 && has_nested_definitions(child, language) {
            return true;
        }
    }

    false
}

fn extract_constant_with_definitions(
    output: &mut String,
    node: Node,
    source: &str,
    language: Language,
    depth: usize,
) {
    let indentation = indent(depth);
    let mut replacements: Vec<(usize, usize, String)> = Vec::new();

    collect_definition_skeletons(node, source, language, &mut replacements);
    replacements.sort_by_key(|r| r.0);

    let node_start = node.start_byte();
    let node_end = node.end_byte();
    let mut result = String::new();
    let mut position = node_start;

    for (start, end, skeleton) in &replacements {
        result.push_str(&source[position..*start]);
        result.push_str(skeleton);
        position = *end;
    }

    result.push_str(&source[position..node_end]);

    for line in result.lines() {
        if line.trim().is_empty() {
            continue;
        }

        let _ = writeln!(output, "{}{}", indentation, line);
    }
}

fn collect_definition_skeletons(
    node: Node,
    source: &str,
    language: Language,
    skeletons: &mut Vec<(usize, usize, String)>,
) {
    let mut cursor = node.walk();

    for child in node.children(&mut cursor) {
        let kind = child.kind();

        if language.definition_types().contains(&kind) {
            let skeleton = build_definition_skeleton(child, source, language);
            skeletons.push((child.start_byte(), child.end_byte(), skeleton));
        } else if child.child_count() > 0 {
            collect_definition_skeletons(child, source, language, skeletons);
        }
    }
}

fn build_definition_skeleton(node: Node, source: &str, language: Language) -> String {
    if let Some(body) = find_body(node, language) {
        let signature = source[node.start_byte()..body.start_byte()].trim_end();

        format!("{}{}", signature, language.ellipsis())
    } else {
        node_text(node, source).to_string()
    }
}

fn append_collapsed_assignment(
    output: &mut String,
    node: Node,
    source: &str,
    depth: usize,
) {
    let indentation = indent(depth);
    let text = node_text(node, source);
    let first_line = text.lines().next().unwrap_or("");

    if let Some(parenthesis_position) = first_line.find('(') {
        let _ = writeln!(output, "{}{}...)", indentation, &first_line[..parenthesis_position + 1]);
    } else if let Some(bracket_position) = first_line.find('[') {
        let _ = writeln!(output, "{}{}...]", indentation, &first_line[..bracket_position + 1]);
    } else if let Some(brace_position) = first_line.find('{') {
        let _ = writeln!(output, "{}{}...}}", indentation, &first_line[..brace_position + 1]);
    } else {
        let _ = writeln!(output, "{}{} ...", indentation, first_line.trim_end());
    }
}

fn estimate_skeleton_tokens(text: &str) -> usize {
    let byte_count = text.len();
    (byte_count * 2 + 6) / 7
}
