use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::constants::APP_NAME;
use crate::model::error::{SwarmError, SwarmResult};
use crate::model::output::OutputFormat;
use crate::ui::themes::Theme;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Options {
    #[serde(default)]
    pub auto_index_on_startup: bool,

    #[serde(default)]
    pub delete_sessions_on_exit: bool,

    #[serde(default = "default_exclude_patterns")]
    pub exclude: Vec<String>,

    #[serde(default)]
    pub include: Vec<String>,

    #[serde(default)]
    pub output_format: OutputFormat,

    #[serde(default = "default_single_instance")]
    pub single_instance: bool,

    #[serde(default)]
    pub theme: Theme,

    #[serde(default)]
    pub use_icon: bool,
}

fn default_exclude_patterns() -> Vec<String> {
    vec![
        ".git".to_string(),
        ".hg".to_string(),
        ".idea".to_string(),
        ".npm".to_string(),
        ".pytest_cache".to_string(),
        ".svn".to_string(),
        ".venv".to_string(),
        ".vs".to_string(),
        ".vscode".to_string(),
        "__pycache__".to_string(),
        "build".to_string(),
        "dist".to_string(),
        "node_modules".to_string(),
        "out".to_string(),
        "target".to_string(),
        "venv".to_string(),

        ".DS_Store".to_string(),
        "Thumbs.db".to_string(),
        "*.7z".to_string(),
        "*.a".to_string(),
        "*.aac".to_string(),
        "*.avi".to_string(),
        "*.bmp".to_string(),
        "*.bz2".to_string(),
        "*.cache".to_string(),
        "*.class".to_string(),
        "*.db".to_string(),
        "*.dll".to_string(),
        "*.doc".to_string(),
        "*.docx".to_string(),
        "*.dylib".to_string(),
        "*.egg-info".to_string(),
        "*.env".to_string(),
        "*.eot".to_string(),
        "*.exe".to_string(),
        "*.flac".to_string(),
        "*.flv".to_string(),
        "*.gif".to_string(),
        "*.gz".to_string(),
        "*.ico".to_string(),
        "*.ipynb".to_string(),
        "*.jar".to_string(),
        "*.jpeg".to_string(),
        "*.jpg".to_string(),
        "*.lib".to_string(),
        "*.m4a".to_string(),
        "*.m4v".to_string(),
        "*.mkv".to_string(),
        "*.mov".to_string(),
        "*.mp3".to_string(),
        "*.mp4".to_string(),
        "*.mpeg".to_string(),
        "*.mpg".to_string(),
        "*.nb".to_string(),
        "*.o".to_string(),
        "*.obj".to_string(),
        "*.ogg".to_string(),
        "*.otf".to_string(),
        "*.pdf".to_string(),
        "*.pdb".to_string(),
        "*.pickle".to_string(),
        "*.pkl".to_string(),
        "*.png".to_string(),
        "*.ppt".to_string(),
        "*.pptx".to_string(),
        "*.psd".to_string(),
        "*.rar".to_string(),
        "*.res".to_string(),
        "*.so".to_string(),
        "*.sqlite".to_string(),
        "*.sqlite3".to_string(),
        "*.svg".to_string(),
        "*.tar".to_string(),
        "*.tiff".to_string(),
        "*.ttf".to_string(),
        "*.war".to_string(),
        "*.wav".to_string(),
        "*.webm".to_string(),
        "*.webp".to_string(),
        "*.wma".to_string(),
        "*.wmv".to_string(),
        "*.woff".to_string(),
        "*.woff2".to_string(),
        "*.xls".to_string(),
        "*.xlsx".to_string(),
        "*.xz".to_string(),
        "*.zip".to_string(),
    ]
}

fn default_single_instance() -> bool {
    true
}

impl Default for Options {
    fn default() -> Self {
        Self {
            auto_index_on_startup: false,
            delete_sessions_on_exit: false,
            exclude: default_exclude_patterns(),
            include: Vec::new(),
            output_format: OutputFormat::default(),
            single_instance: true,
            theme: Theme::default(),
            use_icon: false,
        }
    }
}

impl Options {
    pub fn load() -> SwarmResult<Self> {
        let path = Self::config_path()?;

        if !path.exists() {
            return Ok(Self::default());
        }

        let content = fs::read_to_string(&path)?;
        let mut options: Self = toml::from_str(&content)?;

        if options.exclude.is_empty() {
            options.exclude = default_exclude_patterns();
        }

        Ok(options)
    }

    pub fn save(&self) -> SwarmResult<()> {
        let path = Self::config_path()?;

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let content = toml::to_string_pretty(self)?;
        fs::write(&path, content)?;

        Ok(())
    }

    pub fn add_exclude_filter(&mut self, filter: String) -> bool {
        if filter.is_empty() {
            return false;
        }

        self.exclude.push(filter);
        let _ = self.save();
        true
    }

    pub fn add_include_filter(&mut self, filter: String) -> bool {
        if filter.is_empty() {
            return false;
        }

        self.include.push(filter);
        let _ = self.save();
        true
    }

    pub fn is_equal(&self, other: &Self) -> bool {
        self.auto_index_on_startup == other.auto_index_on_startup
            && self.delete_sessions_on_exit == other.delete_sessions_on_exit
            && self.exclude == other.exclude
            && self.include == other.include
            && self.output_format == other.output_format
            && self.single_instance == other.single_instance
            && self.theme == other.theme
            && self.use_icon == other.use_icon
    }

    pub fn remove_exclude_filter(&mut self, index: usize) -> bool {
        if index >= self.exclude.len() {
            return false;
        }

        self.exclude.remove(index);
        let _ = self.save();
        true
    }

    pub fn remove_include_filter(&mut self, index: usize) -> bool {
        if index >= self.include.len() {
            return false;
        }

        self.include.remove(index);
        let _ = self.save();
        true
    }

    pub fn reset_excludes_to_defaults(&mut self) {
        self.exclude = default_exclude_patterns();
        let _ = self.save();
    }

    fn config_path() -> SwarmResult<PathBuf> {
        dirs::data_local_dir()
            .map(|dir| dir.join(APP_NAME.to_lowercase()).join("options.toml"))
            .ok_or_else(|| SwarmError::Config("Unable to determine configuration path".into()))
    }
}
