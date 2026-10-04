pub const DAEMON_ARGUMENT: &str = "--clipboard-daemon";

#[cfg(all(unix, not(target_os = "macos")))]
pub use daemon::{copy, run_daemon_if_requested};
#[cfg(not(all(unix, not(target_os = "macos"))))]
pub use direct::{copy, run_daemon_if_requested};

#[cfg(all(unix, not(target_os = "macos")))]
mod daemon {
    use core::time::Duration;
    use std::ffi::OsString;
    use std::io::{self, BufRead as _, BufReader, Read as _, Write as _};
    use std::os::unix::process::CommandExt as _;
    use std::process::{Child, ChildStdout, Command, Stdio};
    use std::sync::mpsc;
    use std::thread;

    use arboard::{Clipboard, SetExtLinux as _};

    use super::DAEMON_ARGUMENT;
    use crate::constants::OUTPUT_BYTES_MAX;

    const DAEMON_READY_TIMEOUT_SECONDS: u64 = 5;
    const DAEMON_STATUS_BYTES_MAX: u64 = 4_096;
    const DAEMON_STATUS_READY: &str = "ready";

    pub fn copy(text: &str) -> Result<(), String> {
        if text.len() as u64 > OUTPUT_BYTES_MAX {
            return Err(format!("the payload exceeds {OUTPUT_BYTES_MAX} bytes"));
        }

        let executable = std::env::current_exe()
            .map_err(|error| format!("cannot locate the swarm executable: {error}"))?;

        let mut daemon = Command::new(executable)
            .arg(DAEMON_ARGUMENT)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .process_group(0)
            .spawn()
            .map_err(|error| format!("cannot spawn the clipboard daemon: {error}"))?;

        let status = deliver(&mut daemon, text);

        reap(daemon);

        status
    }

    fn deliver(daemon: &mut Child, text: &str) -> Result<(), String> {
        let mut input = daemon
            .stdin
            .take()
            .ok_or_else(|| "the clipboard daemon has no stdin".to_owned())?;

        input
            .write_all(text.as_bytes())
            .map_err(|error| format!("cannot send the payload to the clipboard daemon: {error}"))?;

        drop(input);

        let output = daemon
            .stdout
            .take()
            .ok_or_else(|| "the clipboard daemon has no stdout".to_owned())?;

        read_status(output)
    }

    fn read_status(output: ChildStdout) -> Result<(), String> {
        let (sender, receiver) = mpsc::sync_channel(1);

        thread::Builder::new()
            .name("clipboard-status".to_owned())
            .spawn(move || {
                let mut reader = BufReader::new(output.take(DAEMON_STATUS_BYTES_MAX));
                let mut line = String::new();
                let result = reader.read_line(&mut line).map(|_| line);

                let _ = sender.send(result);
            })
            .map_err(|error| format!("cannot watch the clipboard daemon: {error}"))?;

        let timeout = Duration::from_secs(DAEMON_READY_TIMEOUT_SECONDS);

        match receiver.recv_timeout(timeout) {
            Ok(Ok(line)) => status_from_line(line.trim()),
            Ok(Err(error)) => Err(format!("cannot read the clipboard daemon status: {error}")),
            Err(_) => Err("the clipboard daemon did not respond".to_owned()),
        }
    }

    fn reap(mut daemon: Child) {
        let spawned = thread::Builder::new()
            .name("clipboard-reaper".to_owned())
            .spawn(move || {
                let _ = daemon.wait();
            });

        if let Err(error) = spawned {
            eprintln!("The clipboard daemon will not be reaped: {error}");
        }
    }

    fn report(status: &str) {
        let mut handle = io::stdout().lock();

        let _ = writeln!(handle, "{status}");
        let _ = handle.flush();
    }

    fn run_daemon() {
        let mut payload = String::new();
        let mut input = io::stdin().take(OUTPUT_BYTES_MAX + 1);

        if let Err(error) = input.read_to_string(&mut payload) {
            report(&format!("cannot read the clipboard payload: {error}"));

            return;
        }

        if payload.len() as u64 > OUTPUT_BYTES_MAX {
            report(&format!("the clipboard payload exceeds {OUTPUT_BYTES_MAX} bytes"));

            return;
        }

        let mut clipboard = match Clipboard::new() {
            Ok(clipboard) => clipboard,
            Err(error) => {
                report(&format!("cannot access the clipboard: {error}"));

                return;
            }
        };

        if let Err(error) = clipboard.set().text(payload.as_str()) {
            report(&format!("cannot write to the clipboard: {error}"));

            return;
        }

        report(DAEMON_STATUS_READY);

        let _ = clipboard.set().wait().text(payload.as_str());
    }

    pub fn run_daemon_if_requested(arguments: &[OsString]) -> bool {
        let requested = arguments
            .get(1)
            .is_some_and(|argument| argument.as_os_str() == DAEMON_ARGUMENT);

        if !requested {
            return false;
        }

        run_daemon();

        true
    }

    fn status_from_line(status: &str) -> Result<(), String> {
        if status == DAEMON_STATUS_READY {
            return Ok(());
        }

        if status.is_empty() {
            return Err("the clipboard daemon exited before taking ownership".to_owned());
        }

        Err(status.to_owned())
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn only_the_ready_line_succeeds() {
            assert_eq!(status_from_line(DAEMON_STATUS_READY), Ok(()));
            assert!(status_from_line("").is_err());
            assert_eq!(status_from_line("broken"), Err("broken".to_owned()));
        }

        #[test]
        fn the_daemon_runs_only_when_requested() {
            assert!(!run_daemon_if_requested(&[OsString::from("swarm"), OsString::from("/tmp")]));
            assert!(!run_daemon_if_requested(&[]));
        }
    }
}

#[cfg(not(all(unix, not(target_os = "macos"))))]
mod direct {
    use std::ffi::OsString;

    use arboard::Clipboard;

    pub fn copy(text: &str) -> Result<(), String> {
        let mut clipboard =
            Clipboard::new().map_err(|error| format!("cannot access the clipboard: {error}"))?;

        clipboard
            .set_text(text)
            .map_err(|error| format!("cannot write to the clipboard: {error}"))
    }

    pub fn run_daemon_if_requested(_arguments: &[OsString]) -> bool {
        false
    }
}
