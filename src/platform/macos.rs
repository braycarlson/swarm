use std::io;
use std::path::Path;
use std::process::Command;

use super::process::spawn_reaped;

pub const CONTEXT_MENU_SUPPORTED: bool = false;

pub fn console_attach() {}

pub fn console_prompt_flush() {}

pub fn context_menu_is_registered() -> bool {
    false
}

pub fn context_menu_register() -> io::Result<()> {
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        "the context menu integration is not available on macOS",
    ))
}

pub fn context_menu_unregister() -> io::Result<()> {
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        "the context menu integration is not available on macOS",
    ))
}

pub fn path_open(path: &Path) -> io::Result<()> {
    spawn_reaped(Command::new("open").arg(path))
}

pub fn path_reveal(path: &Path) -> io::Result<()> {
    spawn_reaped(Command::new("open").arg("-R").arg(path))
}
