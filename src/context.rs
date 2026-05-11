#[cfg(windows)]
use winreg::enums::*;
#[cfg(windows)]
use winreg::RegKey;
#[cfg(windows)]
use std::io;

#[cfg(windows)]
pub fn is_registered() -> bool {
    let current_user_key = RegKey::predef(HKEY_CURRENT_USER);

    current_user_key.open_subkey("Software\\Classes\\Directory\\shell\\swarm").is_ok()
        && current_user_key.open_subkey("Software\\Classes\\*\\shell\\swarm").is_ok()
}

#[cfg(windows)]
pub fn register() -> io::Result<()> {
    let executable_path = std::env::current_exe()?;
    let string_path_executable = executable_path.to_string_lossy();

    let current_user_key = RegKey::predef(HKEY_CURRENT_USER);
    let classes = current_user_key.create_subkey("Software\\Classes")?;

    let (key, _) = classes.0.create_subkey("Directory\\shell\\swarm")?;
    key.set_value("", &"Open swarm here")?;
    key.set_value("Icon", &string_path_executable.as_ref())?;

    let (command_key, _) = classes.0.create_subkey("Directory\\shell\\swarm\\command")?;
    command_key.set_value("", &format!("\"{}\" \"%V\"", string_path_executable))?;

    let (background_key, _) = classes.0.create_subkey("Directory\\Background\\shell\\swarm")?;
    background_key.set_value("", &"Open swarm here")?;
    background_key.set_value("Icon", &string_path_executable.as_ref())?;

    let (background_command_key, _) = classes.0.create_subkey("Directory\\Background\\shell\\swarm\\command")?;
    background_command_key.set_value("", &format!("\"{}\" \"%V\"", executable_path.display()))?;

    let (file_key, _) = classes.0.create_subkey("*\\shell\\swarm")?;
    file_key.set_value("", &"Open swarm here")?;
    file_key.set_value("Icon", &string_path_executable.as_ref())?;

    let (file_command_key, _) = classes.0.create_subkey("*\\shell\\swarm\\command")?;
    file_command_key.set_value("", &format!("\"{}\" \"%V\"", executable_path.display()))?;

    Ok(())
}

#[cfg(windows)]
pub fn unregister() -> io::Result<()> {
    let current_user_key = RegKey::predef(HKEY_CURRENT_USER);

    if let Ok(classes) = current_user_key.open_subkey("Software\\Classes") {
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
