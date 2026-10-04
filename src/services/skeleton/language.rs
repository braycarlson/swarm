use std::path::Path;

use tree_sitter::Language as Grammar;

const EXTENSIONS: [(&str, Language); 14] = [
    ("cjs", Language::JavaScript),
    ("css", Language::Css),
    ("js", Language::JavaScript),
    ("jsx", Language::JavaScript),
    ("less", Language::Css),
    ("mjs", Language::JavaScript),
    ("py", Language::Python),
    ("pyi", Language::Python),
    ("pyw", Language::Python),
    ("rs", Language::Rust),
    ("scss", Language::Css),
    ("ts", Language::JavaScript),
    ("tsx", Language::JavaScript),
    ("zig", Language::Zig),
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Language {
    Css,
    JavaScript,
    Python,
    Rust,
    Zig,
}

impl Language {
    pub const COUNT: usize = 5;

    pub fn body_field(self) -> &'static str {
        match self {
            Self::Css | Self::JavaScript | Self::Python | Self::Rust | Self::Zig => "body",
        }
    }

    pub fn body_kinds(self) -> &'static [&'static str] {
        match self {
            Self::Css => &["block", "keyframe_block_list"],
            Self::JavaScript | Self::Python | Self::Rust | Self::Zig => &[],
        }
    }

    pub fn class_types(self) -> &'static [&'static str] {
        match self {
            Self::Css => &["media_statement", "supports_statement"],
            Self::JavaScript => &["class_declaration"],
            Self::Python => &["class_definition"],
            Self::Rust => &["impl_item", "trait_item"],
            Self::Zig => &[],
        }
    }

    pub fn constant_types(self) -> &'static [&'static str] {
        match self {
            Self::Css => &[],
            Self::JavaScript => &["lexical_declaration", "variable_declaration"],
            Self::Python => &["expression_statement"],
            Self::Rust => &[
                "const_item",
                "enum_item",
                "macro_definition",
                "mod_item",
                "static_item",
                "struct_item",
                "type_item",
            ],
            Self::Zig => &["variable_declaration"],
        }
    }

    pub fn definition_types(self) -> &'static [&'static str] {
        match self {
            Self::Css => &["keyframes_statement", "rule_set"],
            Self::JavaScript => &[
                "arrow_function",
                "function_declaration",
                "method_definition",
            ],
            Self::Python => &["function_definition"],
            Self::Rust => &["function_item"],
            Self::Zig => &["function_declaration", "test_declaration"],
        }
    }

    pub fn ellipsis(self) -> &'static str {
        match self {
            Self::Css | Self::JavaScript | Self::Rust | Self::Zig => " { ... }",
            Self::Python => " ...",
        }
    }

    pub fn from_path(path: &Path) -> Option<Self> {
        let extension = path.extension()?.to_str()?;

        EXTENSIONS
            .iter()
            .find(|(candidate, _)| extension.eq_ignore_ascii_case(candidate))
            .map(|(_, language)| *language)
    }

    pub fn grammar(self) -> Grammar {
        match self {
            Self::Css => tree_sitter_css::LANGUAGE.into(),
            Self::JavaScript => tree_sitter_javascript::LANGUAGE.into(),
            Self::Python => tree_sitter_python::LANGUAGE.into(),
            Self::Rust => tree_sitter_rust::LANGUAGE.into(),
            Self::Zig => tree_sitter_zig::LANGUAGE.into(),
        }
    }

    pub fn import_types(self) -> &'static [&'static str] {
        match self {
            Self::Css => &[
                "charset_statement",
                "import_statement",
                "namespace_statement",
            ],
            Self::JavaScript => &["import_statement"],
            Self::Python => &["import_from_statement", "import_statement"],
            Self::Rust => &["extern_crate_declaration", "use_declaration"],
            Self::Zig => &[],
        }
    }

    pub fn index(self) -> usize {
        let index = match self {
            Self::Css => 0,
            Self::JavaScript => 1,
            Self::Python => 2,
            Self::Rust => 3,
            Self::Zig => 4,
        };

        assert!(index < Self::COUNT);

        index
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::Css => "CSS",
            Self::JavaScript => "JavaScript",
            Self::Python => "Python",
            Self::Rust => "Rust",
            Self::Zig => "Zig",
        }
    }

    pub fn wrapper_types(self) -> &'static [&'static str] {
        match self {
            Self::Css | Self::Rust | Self::Zig => &[],
            Self::JavaScript => &["export_statement"],
            Self::Python => &["decorated_definition"],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extensions_resolve_case_insensitively() {
        assert_eq!(Language::from_path(Path::new("/a/main.RS")), Some(Language::Rust));
        assert_eq!(Language::from_path(Path::new("/a/app.tsx")), Some(Language::JavaScript));
        assert_eq!(Language::from_path(Path::new("/a/notes.txt")), None);
        assert_eq!(Language::from_path(Path::new("/a/Makefile")), None);
    }

    #[test]
    fn every_language_has_a_distinct_index() {
        let languages = [
            Language::Css,
            Language::JavaScript,
            Language::Python,
            Language::Rust,
            Language::Zig,
        ];

        for (position, language) in languages.iter().enumerate() {
            assert_eq!(language.index(), position);
        }
    }
}
