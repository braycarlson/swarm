use std::fs;
use std::io::ErrorKind;
use std::path::PathBuf;

use globset::Glob;
use serde::{Deserialize, Serialize};

use crate::constants::{UI_SCALE_MAX, UI_SCALE_MIN};
use crate::model::error::{SwarmError, SwarmResult};
use crate::model::output::OutputFormat;
use crate::model::storage;
use crate::model::theme::Theme;

const OPTIONS_FILE_NAME: &str = "options.toml";

pub const EXCLUDE_PATTERNS_DEFAULT: &[&str] = &[
    ".git",
    ".svn",
    ".hg",
    ".bzr",
    ".fossil",
    "_darcs",
    "target",
    "build",
    "dist",
    "out",
    "bin",
    "obj",
    "_build",
    ".build",
    "release",
    "debug",
    "Release",
    "Debug",
    "node_modules",
    "bower_components",
    "jspm_packages",
    "vendor",
    "packages",
    ".bundle",
    "deps",
    "_deps",
    "__pycache__",
    ".pytest_cache",
    ".mypy_cache",
    ".ruff_cache",
    ".pytype",
    ".tox",
    "venv",
    ".venv",
    "env",
    ".env",
    "virtualenv",
    ".virtualenv",
    "ENV",
    ".eggs",
    "*.egg-info",
    ".Python",
    ".npm",
    ".yarn",
    ".pnp",
    ".next",
    ".nuxt",
    ".cache",
    ".parcel-cache",
    ".turbo",
    ".vercel",
    ".docusaurus",
    ".gradle",
    ".mvn",
    ".m2",
    ".settings",
    ".vs",
    ".idea",
    ".vscode",
    ".vscode-test",
    ".fleet",
    ".eclipse",
    "coverage",
    ".coverage",
    "htmlcov",
    ".nyc_output",
    "test-results",
    "test-reports",
    ".jest",
    "_site",
    ".terraform",
    ".vagrant",
    ".docker",
    ".devcontainer",
    ".history",
    ".metals",
    ".bloop",
    "CMakeFiles",
    "cmake-build-debug",
    "cmake-build-release",
    ".DS_Store",
    "Thumbs.db",
    "Desktop.ini",
    "$RECYCLE.BIN",
    "*.tmp",
    "*.temp",
    "*.swp",
    "*.swo",
    "*.old",
    "*.orig",
    "*.cache",
    "*.log",
    "*.jpg",
    "*.jpeg",
    "*.png",
    "*.gif",
    "*.bmp",
    "*.svg",
    "*.ico",
    "*.webp",
    "*.tiff",
    "*.tif",
    "*.psd",
    "*.raw",
    "*.heif",
    "*.heic",
    "*.indd",
    "*.ai",
    "*.eps",
    "*.cr2",
    "*.nef",
    "*.orf",
    "*.sr2",
    "*.dng",
    "*.mp4",
    "*.avi",
    "*.mov",
    "*.wmv",
    "*.flv",
    "*.mkv",
    "*.webm",
    "*.m4v",
    "*.mpg",
    "*.mpeg",
    "*.3gp",
    "*.ogv",
    "*.m2ts",
    "*.mts",
    "*.vob",
    "*.mp3",
    "*.wav",
    "*.flac",
    "*.aac",
    "*.ogg",
    "*.wma",
    "*.m4a",
    "*.opus",
    "*.ape",
    "*.alac",
    "*.aiff",
    "*.au",
    "*.mid",
    "*.midi",
    "*.ra",
    "*.rm",
    "*.zip",
    "*.tar",
    "*.gz",
    "*.rar",
    "*.7z",
    "*.bz2",
    "*.xz",
    "*.tgz",
    "*.tbz2",
    "*.lz",
    "*.lzma",
    "*.z",
    "*.cab",
    "*.iso",
    "*.dmg",
    "*.pkg",
    "*.deb",
    "*.rpm",
    "*.apk",
    "*.msi",
    "*.exe",
    "*.dll",
    "*.so",
    "*.dylib",
    "*.lib",
    "*.a",
    "*.o",
    "*.obj",
    "*.pdb",
    "*.class",
    "*.jar",
    "*.war",
    "*.ear",
    "*.bin",
    "*.dat",
    "*.app",
    "*.com",
    "*.sys",
    "*.drv",
    "*.res",
    "*.db",
    "*.sqlite",
    "*.sqlite3",
    "*.mdb",
    "*.accdb",
    "*.dbf",
    "*.sdf",
    "*.bak",
    "*.db3",
    "*.fdb",
    "*.gdb",
    "*.kdb",
    "*.ttf",
    "*.otf",
    "*.woff",
    "*.woff2",
    "*.eot",
    "*.fnt",
    "*.fon",
    "*.pfb",
    "*.pfm",
    "*.pdf",
    "*.doc",
    "*.docx",
    "*.xls",
    "*.xlsx",
    "*.ppt",
    "*.pptx",
    "*.odt",
    "*.ods",
    "*.odp",
    "*.pages",
    "*.numbers",
    "*.key",
    "*.rtf",
    "*.pyc",
    "*.pyo",
    "*.pyd",
    "*.elc",
    "*.rbc",
    "*.beam",
    "*.fasl",
    "*.fbx",
    "*.dae",
    "*.3ds",
    "*.blend",
    "*.c4d",
    "*.max",
    "*.ma",
    "*.mb",
    "*.stl",
    "*.ply",
    "*.unity3d",
    "*.unitypackage",
    "*.asset",
    "*.prefab",
    "*.pak",
    "*.vpk",
    "*.wad",
    "*.bsp",
    "*.vdi",
    "*.vmdk",
    "*.vhd",
    "*.vhdx",
    "*.qcow2",
    "*.img",
    "*.toast",
    "*.enc",
    "*.gpg",
    "*.aes",
    "*.pgp",
    "*.p12",
    "*.pfx",
    "*.keystore",
    "*.crx",
    "*.xpi",
    "*.safariextz",
    "*.ipa",
    "*.aab",
    "*.nupkg",
    "*.snupkg",
    "*.vsix",
    "*.gem",
    "*.whl",
    "*.egg",
];

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Options {
    #[serde(default)]
    pub delete_sessions_on_exit: bool,
    #[serde(default = "exclude_patterns_default")]
    pub exclude: Vec<String>,
    #[serde(default)]
    pub include: Vec<String>,
    #[serde(default)]
    pub output_format: OutputFormat,
    #[serde(default)]
    pub show_hidden: bool,
    #[serde(default = "single_instance_default")]
    pub single_instance: bool,
    #[serde(default)]
    pub theme: Theme,
    #[serde(default)]
    pub ui_scale: Option<f32>,
    #[serde(default)]
    pub use_icon: bool,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            delete_sessions_on_exit: false,
            exclude: exclude_patterns_default(),
            include: Vec::new(),
            output_format: OutputFormat::default(),
            show_hidden: false,
            single_instance: single_instance_default(),
            theme: Theme::default(),
            ui_scale: None,
            use_icon: false,
        }
    }
}

impl Options {
    fn configuration_path() -> SwarmResult<PathBuf> {
        Ok(storage::application_directory()?.join(OPTIONS_FILE_NAME))
    }

    fn patterns_mut(&mut self, list: PatternList) -> &mut Vec<String> {
        match list {
            PatternList::Exclude => &mut self.exclude,
            PatternList::Include => &mut self.include,
        }
    }

    fn sanitize(&mut self) {
        retain_valid_patterns(&mut self.exclude);
        retain_valid_patterns(&mut self.include);

        if self.exclude.is_empty() {
            self.exclude = exclude_patterns_default();
        }

        if let Some(scale) = self.ui_scale {
            if !(UI_SCALE_MIN..=UI_SCALE_MAX).contains(&scale) {
                eprintln!("Ignoring the out-of-range UI scale {scale}");

                self.ui_scale = None;
            }
        }

        debug_assert_ne!(self.exclude.len(), 0);
        debug_assert!(self.ui_scale.is_none_or(|scale| scale.is_finite()));
    }

    pub fn add_pattern(&mut self, list: PatternList, pattern: &str) -> SwarmResult<()> {
        let trimmed = pattern.trim();

        if trimmed.is_empty() {
            return Err(SwarmError::Validation("the pattern is empty".to_owned()));
        }

        validate_pattern(trimmed)?;

        let patterns = self.patterns_mut(list);
        let patterns_length_before = patterns.len();

        patterns.push(trimmed.to_owned());

        debug_assert_eq!(patterns.len(), patterns_length_before + 1);

        Ok(())
    }

    pub fn load() -> SwarmResult<Self> {
        let path = Self::configuration_path()?;

        let content = match fs::read_to_string(&path) {
            Ok(content) => content,
            Err(error) if error.kind() == ErrorKind::NotFound => return Ok(Self::default()),
            Err(error) => return Err(error.into()),
        };

        let mut options: Self = toml::from_str(&content)?;

        options.sanitize();

        Ok(options)
    }

    pub fn load_or_default() -> Self {
        match Self::load() {
            Ok(options) => options,
            Err(error) => {
                eprintln!("Falling back to the default options: {error}");

                Self::default()
            }
        }
    }

    pub fn patterns(&self, list: PatternList) -> &[String] {
        match list {
            PatternList::Exclude => &self.exclude,
            PatternList::Include => &self.include,
        }
    }

    pub fn remove_pattern(&mut self, list: PatternList, index: usize) -> bool {
        let patterns = self.patterns_mut(list);

        if index >= patterns.len() {
            return false;
        }

        let patterns_length_before = patterns.len();
        let removed = patterns.remove(index);

        debug_assert_ne!(removed, "");
        debug_assert_eq!(patterns.len() + 1, patterns_length_before);

        true
    }

    pub fn reset_patterns(&mut self, list: PatternList) {
        match list {
            PatternList::Exclude => self.exclude = exclude_patterns_default(),
            PatternList::Include => self.include.clear(),
        }

        debug_assert_ne!(self.exclude.len(), 0);
    }

    pub fn save(&self) -> SwarmResult<()> {
        let path = Self::configuration_path()?;
        let content = toml::to_string_pretty(self)?;

        assert_ne!(content, "");

        storage::write_atomic(&path, content.as_bytes())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PatternList {
    Exclude,
    Include,
}

fn exclude_patterns_default() -> Vec<String> {
    let patterns: Vec<String> = EXCLUDE_PATTERNS_DEFAULT
        .iter()
        .map(|pattern| (*pattern).to_owned())
        .collect();

    debug_assert_eq!(patterns.len(), EXCLUDE_PATTERNS_DEFAULT.len());

    patterns
}

fn retain_valid_patterns(patterns: &mut Vec<String>) {
    patterns.retain(|pattern| match validate_pattern(pattern) {
        Ok(()) => true,
        Err(error) => {
            eprintln!("Ignoring the pattern '{pattern}': {error}");

            false
        }
    });
}

fn single_instance_default() -> bool {
    true
}

pub fn validate_pattern(pattern: &str) -> SwarmResult<()> {
    match Glob::new(pattern) {
        Ok(_) => Ok(()),
        Err(error) => Err(SwarmError::Parse(
            format!("Invalid glob pattern '{pattern}': {error}"),
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_excludes_match_the_pattern_table() {
        let options = Options::default();

        assert_eq!(options.exclude.len(), EXCLUDE_PATTERNS_DEFAULT.len());

        assert_eq!(
            options.exclude.first().map(String::as_str),
            EXCLUDE_PATTERNS_DEFAULT.first().copied(),
        );
    }

    #[test]
    fn every_default_exclude_pattern_compiles() {
        for pattern in EXCLUDE_PATTERNS_DEFAULT {
            assert!(validate_pattern(pattern).is_ok(), "the default pattern {pattern} is invalid");
        }
    }

    #[test]
    fn patterns_reject_out_of_range_indices() {
        let mut options = Options::default();
        let length = options.exclude.len();

        assert!(!options.remove_pattern(PatternList::Exclude, length));
        assert!(!options.remove_pattern(PatternList::Include, 0));
        assert_eq!(options.exclude.len(), length);
    }

    #[test]
    fn empty_and_invalid_patterns_are_rejected() {
        let mut options = Options::default();

        assert!(options.add_pattern(PatternList::Include, "   ").is_err());
        assert!(options.add_pattern(PatternList::Include, "a[").is_err());
        assert_eq!(options.include, Vec::<String>::new());
        assert!(options.add_pattern(PatternList::Include, " *.rs ").is_ok());
        assert_eq!(options.include, vec!["*.rs".to_owned()]);
    }

    #[test]
    fn sanitizing_drops_invalid_state() {
        let mut options = Options {
            delete_sessions_on_exit: false,
            exclude: vec!["a[".to_owned()],
            include: vec!["*.rs".to_owned(), "b[".to_owned()],
            output_format: OutputFormat::default(),
            show_hidden: false,
            single_instance: true,
            theme: Theme::default(),
            ui_scale: Some(f32::NAN),
            use_icon: false,
        };

        options.sanitize();

        assert_eq!(options.exclude.len(), EXCLUDE_PATTERNS_DEFAULT.len());
        assert_eq!(options.include, vec!["*.rs".to_owned()]);
        assert_eq!(options.ui_scale, None);
    }

    #[test]
    fn resetting_restores_each_list() {
        let mut options = Options::default();

        options.exclude.clear();
        options.include.push("*.rs".to_owned());
        options.reset_patterns(PatternList::Exclude);
        options.reset_patterns(PatternList::Include);

        assert_eq!(options.exclude.len(), EXCLUDE_PATTERNS_DEFAULT.len());
        assert_eq!(options.include, Vec::<String>::new());
    }
}
