use std::io;
use std::os::windows::process::CommandExt as _;
use std::path::Path;
use std::process::Command;

use winapi::um::processenv::GetStdHandle;
use winapi::um::winbase::STD_INPUT_HANDLE;
use winapi::um::wincon::{ATTACH_PARENT_PROCESS, AttachConsole, WriteConsoleInputW};
use winapi::um::wincontypes::{INPUT_RECORD, KEY_EVENT};
use winreg::RegKey;
use winreg::enums::HKEY_CURRENT_USER;

use super::process::spawn_reaped;

pub const CONTEXT_MENU_SUPPORTED: bool = true;
const CLASSES_KEY: &str = "Software\\Classes";
const CREATE_NO_WINDOW: u32 = 0x0800_0000;
const MENU_LABEL: &str = "Open swarm here";
const MENU_KEYS: [&str; 3] = [
    "*\\shell\\swarm",
    "Directory\\Background\\shell\\swarm",
    "Directory\\shell\\swarm",
];
const VIRTUAL_KEY_RETURN: u16 = 0x0D;

pub fn console_attach() {
    let _ = unsafe { AttachConsole(ATTACH_PARENT_PROCESS) };
}

pub fn console_prompt_flush() {
    let handle = unsafe { GetStdHandle(STD_INPUT_HANDLE) };
    let mut record: INPUT_RECORD = unsafe { core::mem::zeroed() };

    record.EventType = KEY_EVENT;

    let key_event = unsafe { record.Event.KeyEvent_mut() };

    key_event.bKeyDown = 1;
    key_event.wRepeatCount = 1;
    key_event.wVirtualKeyCode = VIRTUAL_KEY_RETURN;

    let character = unsafe { key_event.uChar.UnicodeChar_mut() };

    *character = u16::from(b'\r');

    let mut written: u32 = 0;
    let outcome = unsafe { WriteConsoleInputW(handle, &raw const record, 1, &raw mut written) };

    if outcome == 0 {
        eprintln!("The console prompt was not redrawn");

        return;
    }

    if written != 1 {
        eprintln!("The console prompt was not redrawn");
    }
}

pub fn context_menu_is_registered() -> bool {
    let Ok(classes) = RegKey::predef(HKEY_CURRENT_USER).open_subkey(CLASSES_KEY) else {
        return false;
    };

    MENU_KEYS.iter().all(|key| classes.open_subkey(key).is_ok())
}

pub fn context_menu_register() -> io::Result<()> {
    let executable = std::env::current_exe()?;
    let executable_text = executable.to_string_lossy();
    let command = format!("\"{executable_text}\" \"%V\"");
    let (classes, _) = RegKey::predef(HKEY_CURRENT_USER).create_subkey(CLASSES_KEY)?;

    for key_path in MENU_KEYS {
        let (key, _) = classes.create_subkey(key_path)?;

        key.set_value("", &MENU_LABEL)?;
        key.set_value("Icon", &executable_text.as_ref())?;

        let (command_key, _) = classes.create_subkey(format!("{key_path}\\command"))?;

        command_key.set_value("", &command)?;
    }

    debug_assert!(context_menu_is_registered());

    Ok(())
}

pub fn context_menu_unregister() -> io::Result<()> {
    let classes = RegKey::predef(HKEY_CURRENT_USER).open_subkey(CLASSES_KEY)?;

    for key_path in MENU_KEYS {
        match classes.delete_subkey_all(key_path) {
            Ok(()) => {}
            Err(error) if error.kind() == io::ErrorKind::NotFound => {}
            Err(error) => return Err(error),
        }
    }

    debug_assert!(!context_menu_is_registered());

    Ok(())
}

pub fn path_open(path: &Path) -> io::Result<()> {
    spawn_reaped(
        Command::new("explorer")
            .arg(path)
            .creation_flags(CREATE_NO_WINDOW),
    )
}

pub fn path_reveal(path: &Path) -> io::Result<()> {
    spawn_reaped(
        Command::new("explorer")
            .raw_arg(format!("/select,\"{}\"", path.display()))
            .creation_flags(CREATE_NO_WINDOW),
    )
}
