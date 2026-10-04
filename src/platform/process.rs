use std::io;
use std::process::{Command, Stdio};
use std::thread;

pub(crate) fn spawn_reaped(command: &mut Command) -> io::Result<()> {
    command
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());

    let mut child = command.spawn()?;

    thread::Builder::new()
        .name("process-reaper".to_owned())
        .spawn(move || {
            let _ = child.wait();
        })?;

    Ok(())
}
