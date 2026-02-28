use std::fmt::Write;
use std::fs;
use std::path::Path;
use std::sync::Arc;

use ignore::WalkBuilder;
use tree_sitter::{Node, Parser};

use crate::model::error::SwarmResult;
use crate::model::options::Options;
use crate::services::filesystem::filter::{GlobPathFilter, PathFilter};

use super::language::Language;

#[derive(Clone, Debug)]
pub struct SkeletonStats {
    pub file_count: usize,
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
        let mut files = Vec::new();

        for path_str in paths {
            let path = Path::new(path_str);

            if path.is_dir() {
                self.collect_directory(path, &mut files, &filter)?;
            } else if path.is_file() {
                if let Some(entry) = self.process_file(path) {
                    files.push(entry);
                }
            }
        }

        files.sort_by(|a, b| a.0.cmp(&b.0));

        let mut output = String::new();

        for (path, skeleton) in &files {
            let _ = writeln!(output, "[{}]", path);
            output.push_str(skeleton);
            output.push('\n');
        }

        let stats = SkeletonStats {
            file_count: files.len(),
            line_count: output.lines().count(),
            token_count: estimate_skeleton_tokens(&output),
        };

        Ok((output, stats))
    }

    fn process_file(&self, path: &Path) -> Option<(String, String)> {
        let language = Language::from_path(path)?;

        if !language.has_skeleton_support() {
            return None;
        }

        let content = fs::read_to_string(path).ok()?;
        let skeleton = extract_skeleton(&content, language)?;

        if skeleton.trim().is_empty() {
            return None;
        }

        Some((path.display().to_string(), skeleton))
    }

    fn collect_directory(
        &self,
        path: &Path,
        files: &mut Vec<(String, String)>,
        filter: &Arc<dyn PathFilter>,
    ) -> SwarmResult<()> {
        let walker = WalkBuilder::new(path)
            .hidden(true)
            .git_ignore(true)
            .git_global(true)
            .git_exclude(true)
            .build();

        for entry in walker.flatten() {
            let entry_path = entry.path();

            if !entry_path.is_file() {
                continue;
            }

            if !filter.should_include(entry_path) {
                continue;
            }

            if let Some(entry) = self.process_file(entry_path) {
                files.push(entry);
            }
        }

        Ok(())
    }
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
    let mut parser = Parser::new();
    parser.set_language(&language.grammar()).ok()?;

    let tree = parser.parse(content, None)?;
    let root = tree.root_node();
    let source = content.as_bytes();

    let mut output = String::new();
    extract_top_level(&mut output, root, source, language, 0);

    let trimmed = output.trim_end().to_string();

    if trimmed.is_empty() {
        return None;
    }

    Some(format!("{}\n", trimmed))
}

fn extract_top_level(
    output: &mut String,
    node: Node,
    source: &[u8],
    language: Language,
    depth: usize,
) {
    let mut cursor = node.walk();
    let children: Vec<Node> = node.children(&mut cursor).collect();
    let mut prev_category: Option<NodeCategory> = None;

    for child in children {
        let kind = child.kind();

        let category = match classify_node(kind, language) {
            Some(c) => c,
            None => continue,
        };

        let before_len = output.len();

        match category {
            NodeCategory::Wrapper => extract_wrapper(output, child, source, language, depth),
            NodeCategory::Class => extract_class(output, child, source, language, depth),
            NodeCategory::Definition => extract_definition(output, child, source, language, depth),
            NodeCategory::Import => append_node_text(output, child, source, depth),
            NodeCategory::Constant => extract_constant(output, child, source, language, depth),
        }

        let wrote_something = output.len() > before_len;

        if wrote_something {
            if let Some(prev) = prev_category {
                let is_multiline = child.start_position().row != child.end_position().row;

                let output_lines = output[before_len..].lines().count();
                let was_single_line = output_lines <= 1;

                let needs_blank = if depth == 0 {
                    if was_single_line {
                        false
                    } else {
                        prev != category
                            || matches!(category, NodeCategory::Class | NodeCategory::Wrapper)
                            || (category == NodeCategory::Constant && is_multiline)
                    }
                } else {
                    prev != category
                };

                if needs_blank {
                    output.insert(before_len, '\n');
                }
            }

            prev_category = Some(category);
        }
    }
}

fn extract_definition(
    output: &mut String,
    node: Node,
    source: &[u8],
    language: Language,
    depth: usize,
) {
    let indent = "    ".repeat(depth);

    if let Some(body) = find_body(node, language) {
        let sig_start = node.start_byte();
        let sig_end = body.start_byte();
        let sig_text = String::from_utf8_lossy(&source[sig_start..sig_end]);
        let sig = sig_text.trim_end();

        let _ = writeln!(output, "{}{}{}", indent, sig, language.ellipsis());
    } else {
        append_node_text(output, node, source, depth);
    }
}

fn extract_class(
    output: &mut String,
    node: Node,
    source: &[u8],
    language: Language,
    depth: usize,
) {
    let indent = "    ".repeat(depth);

    if let Some(body) = find_body(node, language) {
        let sig_start = node.start_byte();
        let sig_end = body.start_byte();
        let sig_text = String::from_utf8_lossy(&source[sig_start..sig_end]);
        let sig = sig_text.trim_end();

        match language {
            Language::Python => {
                let _ = writeln!(output, "{}{}", indent, sig);
                extract_class_body(output, body, source, language, depth + 1);
            }
            Language::Rust => {
                let _ = writeln!(output, "{}{} {{", indent, sig);
                extract_class_body(output, body, source, language, depth + 1);
                let _ = writeln!(output, "{}}}", indent);
            }
            Language::Css => {
                if let Some(collapsed) = try_collapse_css_body(body, source, language) {
                    let _ = writeln!(output, "{}{} {{ {} }}", indent, sig, collapsed);
                } else {
                    let _ = writeln!(output, "{}{} {{", indent, sig);
                    extract_class_body(output, body, source, language, depth + 1);
                    let _ = writeln!(output, "{}}}", indent);
                }
            }
            _ => {
                let _ = writeln!(output, "{}{} {{", indent, sig);
                extract_class_body(output, body, source, language, depth + 1);
                let _ = writeln!(output, "{}}}", indent);
            }
        }
    } else {
        append_node_text(output, node, source, depth);
    }
}

fn try_collapse_css_body(body: Node, source: &[u8], language: Language) -> Option<String> {
    let mut cursor = body.walk();
    let children: Vec<Node> = body.children(&mut cursor).collect();

    let skeleton_children: Vec<Node> = children
        .into_iter()
        .filter(|child| {
            let kind = child.kind();
            language.definition_types().contains(&kind)
                || language.class_types().contains(&kind)
        })
        .collect();

    if skeleton_children.len() != 1 {
        return None;
    }

    let child = skeleton_children[0];
    let kind = child.kind();

    if language.definition_types().contains(&kind) {
        if let Some(child_body) = find_body(child, language) {
            let sig_start = child.start_byte();
            let sig_end = child_body.start_byte();
            let sig_text = String::from_utf8_lossy(&source[sig_start..sig_end]);
            let sig = sig_text.trim_end();

            return Some(format!("{}{}", sig, language.ellipsis()));
        }
    }

    if language.class_types().contains(&kind) {
        if let Some(child_body) = find_body(child, language) {
            let sig_start = child.start_byte();
            let sig_end = child_body.start_byte();
            let sig_text = String::from_utf8_lossy(&source[sig_start..sig_end]);
            let sig = sig_text.trim_end();

            if let Some(collapsed) = try_collapse_css_body(child_body, source, language) {
                return Some(format!("{} {{ {} }}", sig, collapsed));
            }
        }
    }

    None
}

fn extract_class_body(
    output: &mut String,
    body: Node,
    source: &[u8],
    language: Language,
    depth: usize,
) {
    let indent = "    ".repeat(depth);
    let mut cursor = body.walk();
    let children: Vec<Node> = body.children(&mut cursor).collect();

    let has_skeleton_content = children.iter().any(|child| {
        let kind = child.kind();
        language.definition_types().contains(&kind)
            || language.class_types().contains(&kind)
            || language.wrapper_types().contains(&kind)
    });

    if !has_skeleton_content {
        let _ = writeln!(output, "{}...", indent);
        return;
    }

    for child in children {
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
    source: &[u8],
    language: Language,
    depth: usize,
) {
    let indent = "    ".repeat(depth);
    let mut cursor = node.walk();
    let children: Vec<Node> = node.children(&mut cursor).collect();

    for child in &children {
        let kind = child.kind();

        if kind == "decorator" || kind == "export" {
            let text = node_text(*child, source);
            let _ = writeln!(output, "{}{}", indent, text.trim());
        }
    }

    for child in &children {
        let kind = child.kind();

        if language.definition_types().contains(&kind) {
            extract_definition(output, *child, source, language, depth);
        } else if language.class_types().contains(&kind) {
            extract_class(output, *child, source, language, depth);
        } else if language.constant_types().contains(&kind) {
            extract_constant(output, *child, source, language, depth);
        }
    }
}

fn extract_constant(
    output: &mut String,
    node: Node,
    source: &[u8],
    language: Language,
    depth: usize,
) {
    match language {
        Language::Python => {
            extract_python_constant(output, node, source, depth);
        }
        Language::JavaScript => {
            let text = node_text(node, source);

            if text.starts_with("const ") || text.starts_with("let ") || text.starts_with("var ") {
                if node.start_position().row != node.end_position().row {
                    append_collapsed_assignment(output, node, source, depth);
                } else {
                    append_node_text(output, node, source, depth);
                }
            }
        }
        _ => {
            if node.start_position().row == node.end_position().row {
                append_node_text(output, node, source, depth);
            } else if has_nested_definitions(node, language) {
                extract_constant_with_definitions(output, node, source, language, depth);
            } else {
                append_node_text(output, node, source, depth);
            }
        }
    }
}

fn extract_python_constant(
    output: &mut String,
    node: Node,
    source: &[u8],
    depth: usize,
) {
    let mut cursor = node.walk();
    let has_assignment = node.children(&mut cursor).any(|c| c.kind() == "assignment");

    if !has_assignment {
        return;
    }

    if node.start_position().row != node.end_position().row {
        append_collapsed_assignment(output, node, source, depth);
    } else {
        append_node_text(output, node, source, depth);
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
    source: &[u8],
    language: Language,
    depth: usize,
) {
    let indent = "    ".repeat(depth);
    let mut replacements: Vec<(usize, usize, String)> = Vec::new();

    collect_definition_skeletons(node, source, language, &mut replacements);
    replacements.sort_by_key(|r| r.0);

    let node_start = node.start_byte();
    let node_end = node.end_byte();
    let mut result = String::new();
    let mut pos = node_start;

    for (start, end, skeleton) in &replacements {
        let before = String::from_utf8_lossy(&source[pos..*start]);
        result.push_str(&before);
        result.push_str(skeleton);
        pos = *end;
    }

    let remaining = String::from_utf8_lossy(&source[pos..node_end]);
    result.push_str(&remaining);

    for line in result.lines() {
        if line.trim().is_empty() {
            continue;
        }

        let _ = writeln!(output, "{}{}", indent, line);
    }
}

fn collect_definition_skeletons(
    node: Node,
    source: &[u8],
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

fn build_definition_skeleton(node: Node, source: &[u8], language: Language) -> String {
    if let Some(body) = find_body(node, language) {
        let sig_start = node.start_byte();
        let sig_end = body.start_byte();
        let sig_text = String::from_utf8_lossy(&source[sig_start..sig_end]);
        let sig = sig_text.trim_end();

        format!("{}{}", sig, language.ellipsis())
    } else {
        node_text(node, source)
    }
}

fn append_collapsed_assignment(
    output: &mut String,
    node: Node,
    source: &[u8],
    depth: usize,
) {
    let indent = "    ".repeat(depth);
    let text = node_text(node, source);
    let first_line = text.lines().next().unwrap_or("");

    if let Some(paren_pos) = first_line.find('(') {
        let _ = writeln!(output, "{}{}...)", indent, &first_line[..paren_pos + 1]);
    } else if let Some(bracket_pos) = first_line.find('[') {
        let _ = writeln!(output, "{}{}...]", indent, &first_line[..bracket_pos + 1]);
    } else if let Some(brace_pos) = first_line.find('{') {
        let _ = writeln!(output, "{}{}...}}", indent, &first_line[..brace_pos + 1]);
    } else {
        let _ = writeln!(output, "{}{} ...", indent, first_line.trim_end());
    }
}

fn append_node_text(output: &mut String, node: Node, source: &[u8], depth: usize) {
    let indent = "    ".repeat(depth);
    let text = node_text(node, source);

    for line in text.lines() {
        let _ = writeln!(output, "{}{}", indent, line);
    }
}

fn node_text(node: Node, source: &[u8]) -> String {
    let start = node.start_byte();
    let end = node.end_byte();

    String::from_utf8_lossy(&source[start..end]).to_string()
}

fn estimate_skeleton_tokens(text: &str) -> usize {
    let char_count = text.len();
    (char_count as f64 / 3.5).ceil() as usize
}
