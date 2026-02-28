use std::path::Path;

use tree_sitter::Language as TsLanguage;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Language {
    Css,
    JavaScript,
    Python,
    Rust,
    Zig,
}

impl Language {
    pub fn from_path(path: &Path) -> Option<Self> {
        let extension = path.extension()?.to_str()?.to_lowercase();

        match extension.as_str() {
            "css" | "scss" | "less" => Some(Self::Css),
            "cjs" | "js" | "jsx" | "mjs" | "ts" | "tsx" => Some(Self::JavaScript),
            "py" | "pyi" | "pyw" => Some(Self::Python),
            "rs" => Some(Self::Rust),
            "zig" => Some(Self::Zig),
            _ => None,
        }
    }

    pub fn grammar(&self) -> TsLanguage {
        match self {
            Self::Css => tree_sitter_css::LANGUAGE.into(),
            Self::JavaScript => tree_sitter_javascript::LANGUAGE.into(),
            Self::Python => tree_sitter_python::LANGUAGE.into(),
            Self::Rust => tree_sitter_rust::LANGUAGE.into(),
            Self::Zig => tree_sitter_zig::LANGUAGE.into(),
        }
    }

    pub fn has_skeleton_support(&self) -> bool {
        true
    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::Css => "CSS",
            Self::JavaScript => "JavaScript",
            Self::Python => "Python",
            Self::Rust => "Rust",
            Self::Zig => "Zig",
        }
    }

    pub fn body_field(&self) -> &'static str {
        match self {
            Self::Python => "body",
            Self::JavaScript => "body",
            Self::Rust => "body",
            Self::Zig => "body",
            _ => "body",
        }
    }

    pub fn body_kinds(&self) -> &'static [&'static str] {
        match self {
            Self::Css => &["block", "keyframe_block_list"],
            _ => &[],
        }
    }

    pub fn class_types(&self) -> &'static [&'static str] {
        match self {
            Self::Css => &["media_statement", "supports_statement"],
            Self::JavaScript => &["class_declaration"],
            Self::Python => &["class_definition"],
            Self::Rust => &["impl_item", "trait_item"],
            Self::Zig => &[],
        }
    }

    pub fn constant_types(&self) -> &'static [&'static str] {
        match self {
            Self::JavaScript => &["lexical_declaration", "variable_declaration"],
            Self::Python => &["expression_statement"],
            Self::Rust => &[
                "const_item", "static_item", "type_item",
                "struct_item", "enum_item", "mod_item",
                "macro_definition",
            ],
            Self::Zig => &["variable_declaration"],
            _ => &[],
        }
    }

    pub fn definition_types(&self) -> &'static [&'static str] {
        match self {
            Self::Css => &["rule_set", "keyframes_statement"],
            Self::JavaScript => &["function_declaration", "method_definition", "arrow_function"],
            Self::Python => &["function_definition"],
            Self::Rust => &["function_item"],
            Self::Zig => &["function_declaration", "test_declaration"],
        }
    }

    pub fn ellipsis(&self) -> &'static str {
        match self {
            Self::Python => " ...",
            _ => " { ... }",
        }
    }

    pub fn import_types(&self) -> &'static [&'static str] {
        match self {
            Self::Css => &["import_statement", "charset_statement", "namespace_statement"],
            Self::JavaScript => &["import_statement"],
            Self::Python => &["import_statement", "import_from_statement"],
            Self::Rust => &["use_declaration", "extern_crate_declaration"],
            Self::Zig => &[],
        }
    }

    pub fn wrapper_types(&self) -> &'static [&'static str] {
        match self {
            Self::JavaScript => &["export_statement"],
            Self::Python => &["decorated_definition"],
            _ => &[],
        }
    }
}
