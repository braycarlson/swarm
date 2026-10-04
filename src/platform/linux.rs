use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;

use super::process::spawn_reaped;

pub const CONTEXT_MENU_SUPPORTED: bool = true;
const DESKTOP: &str = r#"[Desktop Entry]
Type=Application
Name=swarm
Comment=Open a directory in swarm
Exec="{executable}" %f
Icon=swarm
Terminal=false
Categories=Development;Utility;
MimeType=inode/directory;
"#;
const EXTENSION: &str = "from gi.repository import Nautilus, GObject, Gio

EXECUTABLE = '{executable}'

PROVIDER = []


def open_in_swarm(_menu, paths):
    for path in paths:
        Gio.Subprocess.new([EXECUTABLE, path], Gio.SubprocessFlags.NONE)


def paths_from_files(files):
    paths = []

    for file in files:
        location = file.get_location() if file.is_directory() else file.get_parent_location()

        if location is None:
            continue

        path = location.get_path()

        if path and path not in paths:
            paths.append(path)

    return paths


def items_for_files(name, files):
    paths = paths_from_files(files)

    if not paths:
        return []

    item = Nautilus.MenuItem(name=name, label='Open in swarm', icon='swarm')
    item.connect('activate', open_in_swarm, paths)

    return [item]


class SwarmMenuProvider(GObject.GObject, Nautilus.MenuProvider):
    def is_owner(self):
        if not PROVIDER:
            PROVIDER.append(self)

        return PROVIDER[0] is self

    def get_file_items(self, files):
        if not self.is_owner():
            return []

        return items_for_files('SwarmNautilus::open_in_swarm', files)

    def get_background_items(self, file):
        if not self.is_owner():
            return []

        return items_for_files('SwarmNautilus::open_folder_in_swarm', [file])
";
const ICON: &[u8] = include_bytes!("../../assets/logo.png");
const ICON_SIZE_PIXELS: u32 = 256;
const PLACEHOLDER_EXECUTABLE: &str = "{executable}";

pub fn console_attach() {}

pub fn console_prompt_flush() {}

pub fn context_menu_is_registered() -> bool {
    path_extension().is_ok_and(|path| path.is_file())
}

pub fn context_menu_register() -> io::Result<()> {
    let executable = std::env::current_exe()?;
    let executable_text = executable.to_string_lossy();
    let extension = path_extension()?;
    let desktop = path_desktop()?;
    let icon = path_icon()?;

    for path in [&extension, &desktop, &icon] {
        let directory = path
            .parent()
            .ok_or_else(|| io::Error::other("the integration path has no directory"))?;

        fs::create_dir_all(directory)?;
    }

    fs::write(
        &extension,
        EXTENSION.replace(PLACEHOLDER_EXECUTABLE, &escape_python(&executable_text)),
    )?;

    fs::write(
        &desktop,
        DESKTOP.replace(PLACEHOLDER_EXECUTABLE, &escape_desktop(&executable_text)),
    )?;

    icon_write(&icon)?;
    nautilus_restart()?;

    debug_assert!(context_menu_is_registered());

    Ok(())
}

pub fn context_menu_unregister() -> io::Result<()> {
    let mut removed_count: u32 = 0;

    for path in [path_extension()?, path_desktop()?, path_icon()?] {
        match fs::remove_file(&path) {
            Ok(()) => removed_count += 1,
            Err(error) if error.kind() == io::ErrorKind::NotFound => {}
            Err(error) => return Err(error),
        }
    }

    debug_assert!(!context_menu_is_registered());

    if removed_count == 0 {
        return Ok(());
    }

    nautilus_restart()
}

fn data_directory() -> io::Result<PathBuf> {
    dirs::data_dir().ok_or_else(|| {
        io::Error::other("no data directory is available for the desktop integration")
    })
}

fn escape_desktop(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

fn escape_python(value: &str) -> String {
    value.replace('\\', "\\\\").replace('\'', "\\'")
}

fn icon_write(path: &Path) -> io::Result<()> {
    let image = image::load_from_memory(ICON)
        .map_err(io::Error::other)?
        .resize_exact(
            ICON_SIZE_PIXELS,
            ICON_SIZE_PIXELS,
            image::imageops::FilterType::Lanczos3,
        );

    assert_eq!(image.width(), ICON_SIZE_PIXELS);

    image
        .save_with_format(path, image::ImageFormat::Png)
        .map_err(io::Error::other)
}

fn nautilus_restart() -> io::Result<()> {
    spawn_reaped(Command::new("nautilus").arg("--quit")).map_err(|error| {
        io::Error::other(format!(
            "the files were written, but Nautilus could not be restarted ({error}); \
             the menu entry appears after the next login",
        ))
    })
}

pub fn path_open(path: &Path) -> io::Result<()> {
    spawn_reaped(Command::new("xdg-open").arg(path))
}

pub fn path_reveal(path: &Path) -> io::Result<()> {
    let directory = path.parent().unwrap_or(path);

    spawn_reaped(Command::new("xdg-open").arg(directory))
}

fn path_desktop() -> io::Result<PathBuf> {
    Ok(data_directory()?.join("applications").join("swarm.desktop"))
}

fn path_extension() -> io::Result<PathBuf> {
    Ok(data_directory()?
        .join("nautilus-python")
        .join("extensions")
        .join("swarm.py"))
}

fn path_icon() -> io::Result<PathBuf> {
    Ok(data_directory()?
        .join("icons")
        .join("hicolor")
        .join(format!("{ICON_SIZE_PIXELS}x{ICON_SIZE_PIXELS}"))
        .join("apps")
        .join("swarm.png"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_python_template_escapes_quotes_and_backslashes() {
        assert_eq!(escape_python("/a/it's\\b"), "/a/it\\'s\\\\b");
    }

    #[test]
    fn the_desktop_template_escapes_quotes_and_backslashes() {
        assert_eq!(escape_desktop("/a/\"b\"\\c"), "/a/\\\"b\\\"\\\\c");
    }

    #[test]
    fn both_templates_carry_the_placeholder() {
        assert!(DESKTOP.contains(PLACEHOLDER_EXECUTABLE));
        assert!(EXTENSION.contains(PLACEHOLDER_EXECUTABLE));
    }
}
