use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::constants::APP_NAME;
use crate::model::error::{SwarmError, SwarmResult};
use crate::model::output::OutputFormat;
use crate::ui::themes::Theme;

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Options {
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
    pub ui_scale: Option<f32>,

    #[serde(default)]
    pub use_icon: bool,
}

fn default_exclude_patterns() -> Vec<String> {
    vec![
        ".git".to_string(),
        ".svn".to_string(),
        ".hg".to_string(),
        ".bzr".to_string(),
        ".vs".to_string(),
        ".vscode".to_string(),
        ".idea".to_string(),
        "node_modules".to_string(),
        "target".to_string(),
        "dist".to_string(),
        "build".to_string(),
        "out".to_string(),
        "bin".to_string(),
        "obj".to_string(),
        "__pycache__".to_string(),
        ".pytest_cache".to_string(),
        ".mypy_cache".to_string(),
        "venv".to_string(),
        ".venv".to_string(),
        "env".to_string(),
        ".env".to_string(),
        "coverage".to_string(),
        ".coverage".to_string(),
        "*.pyc".to_string(),
        "*.pyo".to_string(),
        "*.log".to_string(),
        ".DS_Store".to_string(),
        "Thumbs.db".to_string(),
        "*.7z".to_string(),
        "*.avi".to_string(),
        "*.bin".to_string(),
        "*.bmp".to_string(),
        "*.bz2".to_string(),
        "*.class".to_string(),
        "*.db".to_string(),
        "*.dll".to_string(),
        "*.doc".to_string(),
        "*.docx".to_string(),
        "*.dylib".to_string(),
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

pub fn calculate_default_ui_scale() -> f32 {
    if let Some(scale) = detect_screen_scale() {
        return scale;
    }

    1.3
}

pub fn calculate_ui_scale_for_position(x: f32, y: f32) -> f32 {
    if let Some(scale) = detect_screen_scale_at_position(x as i32, y as i32) {
        return scale;
    }

    calculate_default_ui_scale()
}

fn detect_screen_scale() -> Option<f32> {
    #[cfg(target_os = "windows")]
    {
        use winapi::um::winuser::{GetSystemMetrics, SM_CYSCREEN};

        let height = unsafe { GetSystemMetrics(SM_CYSCREEN) };

        if height > 0 {
            return Some(scale_for_height(height as u32));
        }
    }

    #[cfg(target_os = "macos")]
    {
        use core_graphics::display::CGDisplay;

        let display = CGDisplay::main();
        let height = display.pixels_high() as u32;

        if height > 0 {
            return Some(scale_for_height(height));
        }
    }

    #[cfg(target_os = "linux")]
    {
        if let Ok(output) = std::process::Command::new("xrandr")
            .arg("--current")
            .output()
        {
            let stdout = String::from_utf8_lossy(&output.stdout);

            for line in stdout.lines() {
                if line.contains('*') {
                    if let Some(resolution) = line.split_whitespace().next() {
                        if let Some(height_str) = resolution.split('x').nth(1) {
                            if let Ok(height) = height_str.parse::<u32>() {
                                return Some(scale_for_height(height));
                            }
                        }
                    }
                }
            }
        }
    }

    None
}

fn detect_screen_scale_at_position(x: i32, y: i32) -> Option<f32> {
    #[cfg(target_os = "windows")]
    {
        use winapi::shared::windef::{HMONITOR, POINT, RECT};
        use winapi::um::winuser::{GetMonitorInfoW, MonitorFromPoint, MONITORINFO, MONITOR_DEFAULTTONEAREST};

        let point = POINT { x, y };
        let monitor: HMONITOR = unsafe { MonitorFromPoint(point, MONITOR_DEFAULTTONEAREST) };

        if !monitor.is_null() {
            let mut info: MONITORINFO = unsafe { std::mem::zeroed() };
            info.cbSize = std::mem::size_of::<MONITORINFO>() as u32;

            if unsafe { GetMonitorInfoW(monitor, &mut info) } != 0 {
                let rect: RECT = info.rcMonitor;
                let height = (rect.bottom - rect.top) as u32;

                if height > 0 {
                    return Some(scale_for_height(height));
                }
            }
        }
    }

    #[cfg(target_os = "macos")]
    {
        use core_graphics::display::CGDisplay;

        let displays = CGDisplay::active_displays().ok()?;

        for display_id in displays {
            let display = CGDisplay::new(display_id);
            let bounds = display.bounds();

            let display_x = bounds.origin.x as i32;
            let display_y = bounds.origin.y as i32;
            let display_width = bounds.size.width as i32;
            let display_height = bounds.size.height as i32;

            if x >= display_x
                && x < display_x + display_width
                && y >= display_y
                && y < display_y + display_height
            {
                return Some(scale_for_height(display_height as u32));
            }
        }

        let main_display = CGDisplay::main();
        return Some(scale_for_height(main_display.pixels_high() as u32));
    }

    #[cfg(target_os = "linux")]
    {
        let _ = (x, y);
    }

    None
}

fn scale_for_height(height: u32) -> f32 {
    match height {
        0..=720 => 1.0,
        721..=900 => 1.1,
        901..=1080 => 1.3,
        1081..=1200 => 1.4,
        1201..=1440 => 1.6,
        1441..=1600 => 1.8,
        1601..=1800 => 2.0,
        1801..=2160 => 2.2,
        _ => 2.5,
    }
}

impl Default for Options {
    fn default() -> Self {
        Self {
            delete_sessions_on_exit: false,
            exclude: default_exclude_patterns(),
            include: Vec::new(),
            output_format: OutputFormat::default(),
            single_instance: true,
            theme: Theme::default(),
            ui_scale: None,
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

        if let Some(scale) = options.ui_scale {
            if scale < 0.5 || scale > 3.0 {
                options.ui_scale = None;
            }
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

    pub fn effective_ui_scale(&self) -> f32 {
        self.ui_scale.unwrap_or_else(calculate_default_ui_scale)
    }

    pub fn effective_ui_scale_at_position(&self, x: f32, y: f32) -> f32 {
        self.ui_scale.unwrap_or_else(|| calculate_ui_scale_for_position(x, y))
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

    pub fn clear_includes(&mut self) {
        self.include.clear();
        let _ = self.save();
    }

    pub fn is_equal(&self, other: &Self) -> bool {
        self.delete_sessions_on_exit == other.delete_sessions_on_exit
            && self.exclude == other.exclude
            && self.include == other.include
            && self.output_format == other.output_format
            && self.single_instance == other.single_instance
            && self.theme == other.theme
            && self.ui_scale == other.ui_scale
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
