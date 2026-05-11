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

    #[serde(default)]
    pub show_hidden: bool,

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
        // Version control
        ".git".into(),
        ".svn".into(),
        ".hg".into(),
        ".bzr".into(),
        ".fossil".into(),
        "_darcs".into(),

        // Build outputs
        "target".into(),
        "build".into(),
        "dist".into(),
        "out".into(),
        "bin".into(),
        "obj".into(),
        "_build".into(),
        ".build".into(),
        "release".into(),
        "debug".into(),
        "Release".into(),
        "Debug".into(),

        // Dependencies
        "node_modules".into(),
        "bower_components".into(),
        "jspm_packages".into(),
        "vendor".into(),
        "packages".into(),
        ".bundle".into(),
        "deps".into(),
        "_deps".into(),

        // Python
        "__pycache__".into(),
        ".pytest_cache".into(),
        ".mypy_cache".into(),
        ".ruff_cache".into(),
        ".pytype".into(),
        ".tox".into(),
        "venv".into(),
        ".venv".into(),
        "env".into(),
        ".env".into(),
        "virtualenv".into(),
        ".virtualenv".into(),
        "ENV".into(),
        ".eggs".into(),
        "*.egg-info".into(),
        ".Python".into(),

        // JavaScript / TypeScript
        ".npm".into(),
        ".yarn".into(),
        ".pnp".into(),
        ".next".into(),
        ".nuxt".into(),
        ".cache".into(),
        ".parcel-cache".into(),
        ".turbo".into(),
        ".vercel".into(),
        ".docusaurus".into(),

        // Java / JVM
        ".gradle".into(),
        ".mvn".into(),
        ".m2".into(),
        ".settings".into(),

        // .NET
        ".vs".into(),

        // IDEs and editors
        ".idea".into(),
        ".vscode".into(),
        ".vscode-test".into(),
        ".fleet".into(),
        ".eclipse".into(),

        // Testing and coverage
        "coverage".into(),
        ".coverage".into(),
        "htmlcov".into(),
        ".nyc_output".into(),
        "test-results".into(),
        "test-reports".into(),
        ".jest".into(),

        // Documentation build output
        "_site".into(),

        // Infrastructure
        ".terraform".into(),
        ".vagrant".into(),
        ".docker".into(),
        ".devcontainer".into(),

        // Other dev artifacts
        ".history".into(),
        ".metals".into(),
        ".bloop".into(),
        "CMakeFiles".into(),
        "cmake-build-debug".into(),
        "cmake-build-release".into(),

        // OS files
        ".DS_Store".into(),
        "Thumbs.db".into(),
        "Desktop.ini".into(),
        "$RECYCLE.BIN".into(),

        // Temp and backup
        "*.tmp".into(),
        "*.temp".into(),
        "*.swp".into(),
        "*.swo".into(),
        "*.old".into(),
        "*.orig".into(),
        "*.cache".into(),
        "*.log".into(),

        // Images
        "*.jpg".into(),
        "*.jpeg".into(),
        "*.png".into(),
        "*.gif".into(),
        "*.bmp".into(),
        "*.svg".into(),
        "*.ico".into(),
        "*.webp".into(),
        "*.tiff".into(),
        "*.tif".into(),
        "*.psd".into(),
        "*.raw".into(),
        "*.heif".into(),
        "*.heic".into(),
        "*.indd".into(),
        "*.ai".into(),
        "*.eps".into(),
        "*.cr2".into(),
        "*.nef".into(),
        "*.orf".into(),
        "*.sr2".into(),
        "*.dng".into(),

        // Video
        "*.mp4".into(),
        "*.avi".into(),
        "*.mov".into(),
        "*.wmv".into(),
        "*.flv".into(),
        "*.mkv".into(),
        "*.webm".into(),
        "*.m4v".into(),
        "*.mpg".into(),
        "*.mpeg".into(),
        "*.3gp".into(),
        "*.ogv".into(),
        "*.m2ts".into(),
        "*.mts".into(),
        "*.vob".into(),

        // Audio
        "*.mp3".into(),
        "*.wav".into(),
        "*.flac".into(),
        "*.aac".into(),
        "*.ogg".into(),
        "*.wma".into(),
        "*.m4a".into(),
        "*.opus".into(),
        "*.ape".into(),
        "*.alac".into(),
        "*.aiff".into(),
        "*.au".into(),
        "*.mid".into(),
        "*.midi".into(),
        "*.ra".into(),
        "*.rm".into(),

        // Archives
        "*.zip".into(),
        "*.tar".into(),
        "*.gz".into(),
        "*.rar".into(),
        "*.7z".into(),
        "*.bz2".into(),
        "*.xz".into(),
        "*.tgz".into(),
        "*.tbz2".into(),
        "*.lz".into(),
        "*.lzma".into(),
        "*.z".into(),
        "*.cab".into(),
        "*.iso".into(),
        "*.dmg".into(),
        "*.pkg".into(),
        "*.deb".into(),
        "*.rpm".into(),
        "*.apk".into(),
        "*.msi".into(),

        // Executables and libraries
        "*.exe".into(),
        "*.dll".into(),
        "*.so".into(),
        "*.dylib".into(),
        "*.lib".into(),
        "*.a".into(),
        "*.o".into(),
        "*.obj".into(),
        "*.pdb".into(),
        "*.class".into(),
        "*.jar".into(),
        "*.war".into(),
        "*.ear".into(),
        "*.bin".into(),
        "*.dat".into(),
        "*.app".into(),
        "*.com".into(),
        "*.sys".into(),
        "*.drv".into(),
        "*.res".into(),

        // Database
        "*.db".into(),
        "*.sqlite".into(),
        "*.sqlite3".into(),
        "*.mdb".into(),
        "*.accdb".into(),
        "*.dbf".into(),
        "*.sdf".into(),
        "*.bak".into(),
        "*.db3".into(),
        "*.fdb".into(),
        "*.gdb".into(),
        "*.kdb".into(),

        // Fonts
        "*.ttf".into(),
        "*.otf".into(),
        "*.woff".into(),
        "*.woff2".into(),
        "*.eot".into(),
        "*.fnt".into(),
        "*.fon".into(),
        "*.pfb".into(),
        "*.pfm".into(),

        // Office documents
        "*.pdf".into(),
        "*.doc".into(),
        "*.docx".into(),
        "*.xls".into(),
        "*.xlsx".into(),
        "*.ppt".into(),
        "*.pptx".into(),
        "*.odt".into(),
        "*.ods".into(),
        "*.odp".into(),
        "*.pages".into(),
        "*.numbers".into(),
        "*.key".into(),
        "*.rtf".into(),

        // Compiled / intermediate
        "*.pyc".into(),
        "*.pyo".into(),
        "*.pyd".into(),
        "*.elc".into(),
        "*.rbc".into(),
        "*.beam".into(),
        "*.fasl".into(),

        // 3D models
        "*.fbx".into(),
        "*.dae".into(),
        "*.3ds".into(),
        "*.blend".into(),
        "*.c4d".into(),
        "*.max".into(),
        "*.ma".into(),
        "*.mb".into(),
        "*.stl".into(),
        "*.ply".into(),

        // Game assets
        "*.unity3d".into(),
        "*.unitypackage".into(),
        "*.asset".into(),
        "*.prefab".into(),
        "*.pak".into(),
        "*.vpk".into(),
        "*.wad".into(),
        "*.bsp".into(),

        // VM / disk images
        "*.vdi".into(),
        "*.vmdk".into(),
        "*.vhd".into(),
        "*.vhdx".into(),
        "*.qcow2".into(),
        "*.img".into(),
        "*.toast".into(),

        // Encrypted
        "*.enc".into(),
        "*.gpg".into(),
        "*.aes".into(),
        "*.pgp".into(),
        "*.p12".into(),
        "*.pfx".into(),
        "*.keystore".into(),

        // Package formats
        "*.crx".into(),
        "*.xpi".into(),
        "*.safariextz".into(),
        "*.ipa".into(),
        "*.aab".into(),
        "*.nupkg".into(),
        "*.snupkg".into(),
        "*.vsix".into(),
        "*.gem".into(),
        "*.whl".into(),
        "*.egg".into(),
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
            show_hidden: false,
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
            && self.show_hidden == other.show_hidden
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
