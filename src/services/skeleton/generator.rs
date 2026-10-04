use std::fs::{self, File};
use std::io::Read as _;
use std::path::{Path, PathBuf};

use rayon::iter::{IntoParallelRefIterator as _, ParallelIterator as _};
use tree_sitter::Node;

use crate::constants::OUTPUT_BYTES_MAX;
use crate::model::error::{SwarmError, SwarmResult};
use crate::model::node::Visit;
use crate::model::options::Options;
use crate::model::output::{OutputEntry, OutputFormat};
use crate::services::filesystem::filter::GlobPathFilter;
use crate::services::filesystem::walk::{WalkOptions, walk_files};
use crate::services::parser::{Parsers, walk_subtree};
use crate::services::tokens::estimate_tokens;

use super::language::Language;

const BRACKETS: [(char, char); 3] = [('(', ')'), ('[', ']'), ('{', '}')];
const INDENT_DEPTH_MAX: u32 = 32;
const INDENT_WIDTH: usize = 4;
const INDENT_CACHE_BYTES: usize = INDENT_DEPTH_MAX as usize * INDENT_WIDTH;
const INDENT_SPACES: [u8; INDENT_CACHE_BYTES] = [b' '; INDENT_CACHE_BYTES];
const WRAPPER_LINE_KINDS: [&str; 2] = ["decorator", "export"];

const INDENT_CACHE: &str = match core::str::from_utf8(&INDENT_SPACES) {
    Ok(spaces) => spaces,
    Err(_) => panic!("the indent cache holds ASCII spaces only"),
};

#[derive(Clone, Copy)]
struct Document<'source> {
    language: Language,
    text: &'source str,
}

enum FileOutcome {
    Skeleton(OutputEntry),
    Unreadable,
    Unsupported,
}

#[derive(Clone, Copy, Eq, PartialEq)]
enum NodeCategory {
    Class,
    Constant,
    Definition,
    Import,
    Wrapper,
}

struct Replacement {
    end: usize,
    skeleton: String,
    start: usize,
}

pub struct SkeletonRequest<'request> {
    pub format: OutputFormat,
    pub options: &'request Options,
    pub paths: &'request [PathBuf],
}

#[derive(Clone, Copy, Debug)]
pub struct SkeletonStats {
    pub files: u32,
    pub lines: u64,
    pub skipped: u32,
    pub tokens: u64,
}

enum Work<'tree> {
    Class { depth: u32, node: Node<'tree> },
    ClassBody { body: Node<'tree>, depth: u32 },
    Constant { depth: u32, node: Node<'tree> },
    Definition { depth: u32, node: Node<'tree> },
    Import { node: Node<'tree> },
    Line(String),
    Wrapper { depth: u32, node: Node<'tree> },
}

fn append_collapsed_assignment(output: &mut String, text: &str, depth: u32) {
    let line_first = text.lines().next().unwrap_or("");

    let bracketed = BRACKETS.iter().find_map(|(open, close)| {
        line_first
            .split_once(*open)
            .map(|(before, _)| (before, *open, *close))
    });

    output.push_str(indent(depth));

    if let Some((before, open, close)) = bracketed {
        output.push_str(before);
        output.push(open);
        output.push_str("...");
        output.push(close);
    } else {
        output.push_str(line_first.trim_end());
        output.push_str(" ...");
    }

    output.push('\n');
}

fn child_depth(depth: u32) -> u32 {
    let deeper = (depth + 1).min(INDENT_DEPTH_MAX);

    debug_assert!(deeper >= depth);

    deeper
}

fn children_reversed(node: Node<'_>) -> impl Iterator<Item = Node<'_>> {
    let count = u32::try_from(node.child_count()).expect("tree-sitter counts children in u32");

    (0..count).rev().filter_map(move |index| node.child(index))
}

fn classify_node(kind: &str, language: Language) -> Option<NodeCategory> {
    let tables = [
        (language.import_types(), NodeCategory::Import),
        (language.wrapper_types(), NodeCategory::Wrapper),
        (language.class_types(), NodeCategory::Class),
        (language.definition_types(), NodeCategory::Definition),
        (language.constant_types(), NodeCategory::Constant),
    ];

    tables
        .iter()
        .find(|(kinds, _)| kinds.contains(&kind))
        .map(|(_, category)| *category)
}

fn collect_definition_skeletons(node: Node<'_>, document: Document<'_>) -> Vec<Replacement> {
    let root_id = node.id();
    let mut replacements = Vec::new();

    walk_subtree(node, |current| {
        if current.id() == root_id {
            return Visit::Descend;
        }

        if !document
            .language
            .definition_types()
            .contains(&current.kind())
        {
            if current.child_count() > 0 {
                return Visit::Descend;
            }

            return Visit::Skip;
        }

        replacements.push(Replacement {
            end: current.end_byte(),
            skeleton: definition_skeleton(current, document),
            start: current.start_byte(),
        });

        Visit::Skip
    });

    debug_assert!(replacements.is_sorted_by_key(|replacement| replacement.start));

    replacements
}

fn definition_skeleton(node: Node<'_>, document: Document<'_>) -> String {
    let Some(body) = find_body(node, document.language) else {
        return node_text(node, document.text).to_owned();
    };

    format!(
        "{}{}",
        signature_text(node, body.start_byte(), document.text),
        document.language.ellipsis(),
    )
}

fn extract_class<'tree>(
    output: &mut String,
    work: &mut Vec<Work<'tree>>,
    node: Node<'tree>,
    document: Document<'tree>,
    depth: u32,
) {
    let Some(body) = find_body(node, document.language) else {
        write_indented_lines(output, node_text(node, document.text), depth);

        return;
    };

    let indentation = indent(depth);
    let signature = signature_text(node, body.start_byte(), document.text);
    let depth_body = child_depth(depth);

    if document.language == Language::Python {
        write_line(output, &[indentation, signature]);

        work.push(Work::ClassBody {
            body,
            depth: depth_body,
        });

        return;
    }

    if document.language == Language::Css {
        if let Some(collapsed) = try_collapse_css_body(body, document) {
            write_line(output, &[indentation, signature, " { ", &collapsed, " }"]);

            return;
        }
    }

    write_line(output, &[indentation, signature, " {"]);

    work.push(Work::Line(format!("{indentation}}}")));
    work.push(Work::ClassBody {
        body,
        depth: depth_body,
    });
}

fn extract_constant(output: &mut String, node: Node<'_>, document: Document<'_>, depth: u32) {
    let replacements = collect_definition_skeletons(node, document);

    if !replacements.is_empty() {
        write_with_replacements(output, node, &replacements, document, depth);

        return;
    }

    let text = node_text(node, document.text);

    if text.lines().nth(1).is_some() {
        append_collapsed_assignment(output, text, depth);

        return;
    }

    write_line(output, &[indent(depth), text.lines().next().unwrap_or("")]);
}

fn extract_definition(output: &mut String, node: Node<'_>, document: Document<'_>, depth: u32) {
    let Some(body) = find_body(node, document.language) else {
        write_indented_lines(output, node_text(node, document.text), depth);

        return;
    };

    let signature = signature_text(node, body.start_byte(), document.text);

    write_lines(output, signature, depth, document.language.ellipsis());
}

fn extract_skeleton(text: &str, language: Language, parsers: &mut Parsers) -> Option<String> {
    let tree = parsers.parse(language, text)?;
    let root = tree.root_node();
    let document = Document { language, text };
    let work_count_max = 2 * root.descendant_count() + 1;
    let mut output = String::new();
    let mut work: Vec<Work<'_>> = Vec::new();

    for child in children_reversed(root) {
        if let Some(category) = classify_node(child.kind(), language) {
            work.push(work_for(category, child, 0));
        }
    }

    while let Some(item) = work.pop() {
        assert!(work.len() < work_count_max);

        process_work(item, &mut output, &mut work, document);
    }

    let output_length_trimmed = output.trim_end().len();

    output.truncate(output_length_trimmed);

    if output.is_empty() {
        return None;
    }

    output.push('\n');

    Some(output)
}

fn find_body(node: Node<'_>, language: Language) -> Option<Node<'_>> {
    if let Some(body) = node.child_by_field_name(language.body_field()) {
        return Some(body);
    }

    let kinds = language.body_kinds();

    if kinds.is_empty() {
        return None;
    }

    let mut cursor = node.walk();

    node.children(&mut cursor)
        .find(|child| kinds.contains(&child.kind()))
}

pub fn generate(request: &SkeletonRequest<'_>) -> SwarmResult<(String, SkeletonStats)> {
    let filter = GlobPathFilter::from_options(request.options)?;
    let candidates = paths_for(request.paths, &filter)?;

    let outcomes: Vec<FileOutcome> = candidates
        .par_iter()
        .map_init(Parsers::default, |parsers, path| {
            process_file(path, parsers)
        })
        .collect();

    assert_eq!(outcomes.len(), candidates.len());

    let mut entries = Vec::with_capacity(outcomes.len());
    let mut skipped: u32 = 0;

    for outcome in outcomes {
        match outcome {
            FileOutcome::Skeleton(entry) => entries.push(entry),
            FileOutcome::Unreadable => skipped += 1,
            FileOutcome::Unsupported => {}
        }
    }

    entries.sort_by(|left, right| left.label.cmp(&right.label));

    let output = request.format.format(&entries)?;

    if output.len() as u64 > OUTPUT_BYTES_MAX {
        return Err(SwarmError::Validation(format!(
            "the skeleton holds more than {OUTPUT_BYTES_MAX} bytes; narrow the selection",
        )));
    }

    let stats = SkeletonStats {
        files: u32::try_from(entries.len()).expect("the walk bounds the file count"),
        lines: memchr::memchr_iter(b'\n', output.as_bytes()).count() as u64,
        skipped,
        tokens: estimate_tokens(&output),
    };

    Ok((output, stats))
}

fn indent(depth: u32) -> &'static str {
    debug_assert!(depth <= INDENT_DEPTH_MAX);

    let end = (depth as usize * INDENT_WIDTH).min(INDENT_CACHE.len());

    INDENT_CACHE.get(..end).unwrap_or("")
}

fn member_work(node: Node<'_>, depth: u32, language: Language) -> Option<Work<'_>> {
    let kind = node.kind();

    if language.definition_types().contains(&kind) {
        return Some(Work::Definition { depth, node });
    }

    if language.class_types().contains(&kind) {
        return Some(Work::Class { depth, node });
    }

    if language.wrapper_types().contains(&kind) {
        return Some(Work::Wrapper { depth, node });
    }

    None
}

fn node_text<'source>(node: Node<'_>, source: &'source str) -> &'source str {
    debug_assert!(node.start_byte() <= node.end_byte());
    debug_assert!(node.end_byte() <= source.len());

    node.utf8_text(source.as_bytes()).unwrap_or("")
}

fn paths_for(paths: &[PathBuf], filter: &GlobPathFilter) -> SwarmResult<Vec<PathBuf>> {
    let mut candidates = Vec::with_capacity(paths.len());

    for path in paths {
        let Ok(metadata) = fs::metadata(path) else {
            continue;
        };

        if metadata.is_dir() {
            candidates.extend(walk_files(path, filter, WalkOptions::SKELETON)?);

            continue;
        }

        if metadata.is_file() {
            candidates.push(path.clone());
        }
    }

    Ok(candidates)
}

fn process_file(path: &Path, parsers: &mut Parsers) -> FileOutcome {
    let Some(language) = Language::from_path(path) else {
        return FileOutcome::Unsupported;
    };

    let Some(text) = read_bounded(path) else {
        return FileOutcome::Unreadable;
    };

    let Some(skeleton) = extract_skeleton(&text, language, parsers) else {
        return FileOutcome::Unsupported;
    };

    if skeleton.trim().is_empty() {
        return FileOutcome::Unsupported;
    }

    FileOutcome::Skeleton(OutputEntry {
        content: skeleton,
        label: path.display().to_string(),
    })
}

fn process_work<'tree>(
    item: Work<'tree>,
    output: &mut String,
    work: &mut Vec<Work<'tree>>,
    document: Document<'tree>,
) {
    match item {
        Work::Class { depth, node } => extract_class(output, work, node, document, depth),
        Work::ClassBody { body, depth } => queue_class_body(output, work, body, document, depth),
        Work::Constant { depth, node } => extract_constant(output, node, document, depth),
        Work::Definition { depth, node } => extract_definition(output, node, document, depth),
        Work::Import { node } => write_line(output, &[node_text(node, document.text)]),
        Work::Line(line) => write_line(output, &[&line]),
        Work::Wrapper { depth, node } => queue_wrapper(work, node, document, depth),
    }
}

fn queue_class_body<'tree>(
    output: &mut String,
    work: &mut Vec<Work<'tree>>,
    body: Node<'tree>,
    document: Document<'tree>,
    depth: u32,
) {
    let mut cursor = body.walk();

    let has_content = body
        .children(&mut cursor)
        .any(|child| member_work(child, depth, document.language).is_some());

    if !has_content {
        write_line(output, &[indent(depth), "..."]);

        return;
    }

    let work_length_before = work.len();

    work.extend(
        children_reversed(body).filter_map(|child| member_work(child, depth, document.language)),
    );

    debug_assert!(work.len() > work_length_before);
}

fn queue_wrapper<'tree>(
    work: &mut Vec<Work<'tree>>,
    node: Node<'tree>,
    document: Document<'tree>,
    depth: u32,
) {
    for child in children_reversed(node) {
        if WRAPPER_LINE_KINDS.contains(&child.kind()) {
            work.push(Work::Line(format!(
                "{}{}",
                indent(depth),
                node_text(child, document.text),
            )));

            continue;
        }

        if let Some(item) = member_work(child, depth, document.language) {
            work.push(item);
        }
    }
}

fn read_bounded(path: &Path) -> Option<String> {
    let metadata = fs::symlink_metadata(path).ok()?;

    if !metadata.is_file() {
        return None;
    }

    let size_bytes = metadata.len();

    if size_bytes > OUTPUT_BYTES_MAX {
        return None;
    }

    let file = File::open(path).ok()?;
    let capacity = usize::try_from(size_bytes).ok()?;
    let mut text = String::with_capacity(capacity);

    file.take(size_bytes).read_to_string(&mut text).ok()?;

    Some(text)
}

fn signature_text<'source>(
    node: Node<'_>,
    body_start: usize,
    source: &'source str,
) -> &'source str {
    debug_assert!(node.start_byte() <= body_start);
    debug_assert!(body_start <= source.len());

    source
        .get(node.start_byte()..body_start)
        .unwrap_or("")
        .trim_end()
}

fn single_skeleton_child(body: Node<'_>, language: Language) -> Option<Node<'_>> {
    let mut cursor = body.walk();

    let mut matching = body.children(&mut cursor).filter(|child| {
        let kind = child.kind();

        language.definition_types().contains(&kind) || language.class_types().contains(&kind)
    });

    let first = matching.next()?;

    if matching.next().is_some() {
        return None;
    }

    Some(first)
}

fn try_collapse_css_body(body: Node<'_>, document: Document<'_>) -> Option<String> {
    debug_assert_eq!(document.language, Language::Css);

    let language = document.language;
    let mut signatures: Vec<&str> = Vec::new();
    let mut current = body;

    for _ in 0..INDENT_DEPTH_MAX {
        let child = single_skeleton_child(current, language)?;
        let kind = child.kind();
        let child_body = find_body(child, language)?;
        let signature = signature_text(child, child_body.start_byte(), document.text);

        if language.definition_types().contains(&kind) {
            let mut collapsed = format!("{signature}{}", language.ellipsis());

            for wrapper in signatures.iter().rev() {
                collapsed = format!("{wrapper} {{ {collapsed} }}");
            }

            return Some(collapsed);
        }

        if !language.class_types().contains(&kind) {
            return None;
        }

        signatures.push(signature);
        current = child_body;
    }

    None
}

fn work_for(category: NodeCategory, node: Node<'_>, depth: u32) -> Work<'_> {
    match category {
        NodeCategory::Class => Work::Class { depth, node },
        NodeCategory::Constant => Work::Constant { depth, node },
        NodeCategory::Definition => Work::Definition { depth, node },
        NodeCategory::Import => Work::Import { node },
        NodeCategory::Wrapper => Work::Wrapper { depth, node },
    }
}

fn write_indented_lines(output: &mut String, text: &str, depth: u32) {
    write_lines(output, text, depth, "");
}

fn write_line(output: &mut String, parts: &[&str]) {
    for part in parts {
        output.push_str(part);
    }

    output.push('\n');
}

fn write_lines(output: &mut String, text: &str, depth: u32, suffix: &'static str) {
    let indentation = indent(depth);
    let mut pending: Option<&str> = None;

    for line in text.lines().filter(|line| !line.trim().is_empty()) {
        if let Some(previous) = pending {
            write_line(output, &[indentation, previous]);
        }

        pending = Some(line);
    }

    if let Some(last) = pending {
        write_line(output, &[indentation, last, suffix]);

        return;
    }

    if !suffix.is_empty() {
        write_line(output, &[indentation, suffix]);
    }
}

fn write_with_replacements(
    output: &mut String,
    node: Node<'_>,
    replacements: &[Replacement],
    document: Document<'_>,
    depth: u32,
) {
    let node_end = node.end_byte();
    let mut position = node.start_byte();
    let mut result = String::new();

    for replacement in replacements {
        assert!(replacement.start >= position);
        assert!(replacement.end <= node_end);
        assert!(replacement.start <= replacement.end);

        result.push_str(document.text.get(position..replacement.start).unwrap_or(""));
        result.push_str(&replacement.skeleton);
        position = replacement.end;
    }

    result.push_str(document.text.get(position..node_end).unwrap_or(""));

    write_indented_lines(output, &result, depth);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn skeleton(source: &str, language: Language) -> Option<String> {
        extract_skeleton(source, language, &mut Parsers::default())
    }

    #[test]
    fn rust_skeleton_keeps_signatures_and_drops_bodies() {
        let output = skeleton(
            "use std::fmt;\n\nfn add(a: u32, b: u32) -> u32 {\n    a + b\n}\n",
            Language::Rust,
        )
        .expect("the source has a skeleton");

        assert!(output.contains("use std::fmt;"));
        assert!(output.contains("fn add(a: u32, b: u32) -> u32 { ... }"));
        assert!(!output.contains("a + b"));
    }

    #[test]
    fn a_definition_ellipsis_stays_on_the_signature_line() {
        let output = skeleton(
            "fn wide(\n    a: u32,\n    b: u32,\n) -> u32 {\n    a + b\n}\n",
            Language::Rust,
        )
        .expect("the source has a skeleton");

        assert!(output.contains(") -> u32 { ... }"));
        assert!(!output.lines().any(|line| line.trim() == "{ ... }"));
    }

    #[test]
    fn nested_impl_blocks_keep_their_order() {
        let output = skeleton(
            "impl Thing {\n    fn one(&self) {}\n    fn two(&self) {}\n}\n",
            Language::Rust,
        )
        .expect("the source has a skeleton");

        let one = output.find("fn one").expect("the first method is present");
        let two = output.find("fn two").expect("the second method is present");

        assert!(one < two);
        assert!(output.trim_end().ends_with('}'));
    }

    #[test]
    fn python_class_bodies_are_indented() {
        let output = skeleton(
            "class Thing:\n    def one(self):\n        pass\n",
            Language::Python,
        )
        .expect("the source has a skeleton");

        assert!(output.contains("class Thing:"));
        assert!(output.contains("    def one(self): ..."));
        assert!(!output.contains("pass"));
    }

    #[test]
    fn an_empty_source_yields_no_skeleton() {
        assert!(skeleton("", Language::Rust).is_none());
    }

    #[test]
    fn constants_with_closures_keep_only_the_signature() {
        let output = skeleton(
            "const handler = (value) => {\n    return value;\n};\n",
            Language::JavaScript,
        )
        .expect("the source has a skeleton");

        assert!(output.contains("(value) => { ... }"));
        assert!(!output.contains("return value"));
    }

    #[test]
    fn deeply_nested_css_does_not_panic() {
        let depth = INDENT_DEPTH_MAX as usize + 8;
        let opening = "@media screen { ".repeat(depth);
        let closing = " }".repeat(depth);
        let source = format!("{opening}a {{ color: red; }}{closing}");
        let output = skeleton(&source, Language::Css).expect("the source has a skeleton");

        assert!(output.starts_with("@media"));
    }

    #[test]
    fn deeply_nested_python_classes_do_not_panic() {
        let depth = INDENT_DEPTH_MAX as usize + 8;
        let mut source = String::new();

        for level in 0..depth {
            source.push_str(&"    ".repeat(level));
            source.push_str("class Level");
            source.push_str(&level.to_string());
            source.push_str(":\n");
        }

        source.push_str(&"    ".repeat(depth));
        source.push_str("pass\n");

        let output = skeleton(&source, Language::Python).expect("the source has a skeleton");

        assert!(output.contains("class Level0:"));
    }

    #[test]
    fn indentation_is_clamped() {
        assert_eq!(indent(0), "");
        assert_eq!(indent(1).len(), INDENT_WIDTH);
        assert_eq!(indent(INDENT_DEPTH_MAX).len(), INDENT_DEPTH_MAX as usize * INDENT_WIDTH);
    }

    #[test]
    fn multi_line_assignments_collapse_at_the_first_bracket() {
        let mut output = String::new();

        append_collapsed_assignment(&mut output, "VALUES = [\n    1,\n]", 0);

        assert_eq!(output, "VALUES = [...]\n");
    }
}
