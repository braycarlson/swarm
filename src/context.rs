#[cfg(windows)]
use winreg::enums::*;
#[cfg(windows)]
use winreg::RegKey;
#[cfg(windows)]
use std::io;

#[cfg(windows)]
pub fn is_registered() -> bool {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);

    hkcu.open_subkey("Software\\Classes\\Directory\\shell\\swarm").is_ok()
        && hkcu.open_subkey("Software\\Classes\\*\\shell\\swarm").is_ok()
}

#[cfg(windows)]
pub fn register() -> io::Result<()> {
    let exe_path = std::env::current_exe()?;
    let exe_path_str = exe_path.to_string_lossy();

    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let classes = hkcu.create_subkey("Software\\Classes")?;

    let (key, _) = classes.0.create_subkey("Directory\\shell\\swarm")?;
    key.set_value("", &"Open swarm here")?;
    key.set_value("Icon", &exe_path_str.as_ref())?;

    let (command_key, _) = classes.0.create_subkey("Directory\\shell\\swarm\\command")?;
    command_key.set_value("", &format!("\"{}\" \"%V\"", exe_path_str))?;

    let (bg_key, _) = classes.0.create_subkey("Directory\\Background\\shell\\swarm")?;
    bg_key.set_value("", &"Open swarm here")?;
    bg_key.set_value("Icon", &exe_path_str.as_ref())?;

    let (bg_command_key, _) = classes.0.create_subkey("Directory\\Background\\shell\\swarm\\command")?;
    bg_command_key.set_value("", &format!("\"{}\" \"%V\"", exe_path_str))?;

    let (file_key, _) = classes.0.create_subkey("*\\shell\\swarm")?;
    file_key.set_value("", &"Open swarm here")?;
    file_key.set_value("Icon", &exe_path_str.as_ref())?;

    let (file_command_key, _) = classes.0.create_subkey("*\\shell\\swarm\\command")?;
    file_command_key.set_value("", &format!("\"{}\" \"%V\"", exe_path_str))?;

    Ok(())
}

#[cfg(windows)]
pub fn unregister() -> io::Result<()> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);

    if let Ok(classes) = hkcu.open_subkey("Software\\Classes") {
        let _ = classes.delete_subkey_all("Directory\\shell\\swarm");
        let _ = classes.delete_subkey_all("Directory\\Background\\shell\\swarm");
        let _ = classes.delete_subkey_all("*\\shell\\swarm");
    }

    Ok(())
}

#[cfg(not(windows))]
pub fn is_registered() -> bool {
    false
}

#[cfg(not(windows))]
pub fn register() -> std::io::Result<()> {
    Ok(())
}

#[cfg(not(windows))]
pub fn unregister() -> std::io::Result<()> {
    Ok(())
}
