pub mod clipboard;
mod process;

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "macos")]
mod macos;
#[cfg(not(any(target_os = "linux", target_os = "macos", windows)))]
mod other;
#[cfg(windows)]
mod windows;

#[cfg(target_os = "linux")]
use linux as system;
#[cfg(target_os = "macos")]
use macos as system;
#[cfg(not(any(target_os = "linux", target_os = "macos", windows)))]
use other as system;
#[cfg(windows)]
use windows as system;

pub use system::{
    CONTEXT_MENU_SUPPORTED,
    console_attach,
    console_prompt_flush,
    context_menu_is_registered,
    context_menu_register,
    context_menu_unregister,
    path_open,
    path_reveal,
};
