use core::time::Duration;
use std::io::{self, Read as _};
use std::net::{TcpListener, TcpStream};
use std::path::PathBuf;
use std::thread;

use crate::app::message::{App, Message, MessageSender};
use crate::constants::{
    IPC_ADDRESS,
    IPC_MESSAGE_BYTES_MAX,
    IPC_PATH_COUNT_MAX,
    IPC_READ_TIMEOUT_MS,
};

fn handle_connection(stream: TcpStream, sender: &MessageSender) {
    if let Err(error) = stream.set_read_timeout(Some(Duration::from_millis(IPC_READ_TIMEOUT_MS))) {
        eprintln!("Failed to bound the IPC read: {error}");

        return;
    }

    let mut buffer = String::new();
    let mut reader = stream.take(IPC_MESSAGE_BYTES_MAX);

    if let Err(error) = reader.read_to_string(&mut buffer) {
        eprintln!("Failed to read from the IPC stream: {error}");

        return;
    }

    assert!(buffer.len() as u64 <= IPC_MESSAGE_BYTES_MAX);

    let paths = parse_paths(&buffer);

    if paths.is_empty() {
        return;
    }

    debug_assert!(paths.len() <= IPC_PATH_COUNT_MAX as usize);

    sender.send(Message::App(App::PathsReceivedFromIpc(paths)));
}

fn parse_paths(buffer: &str) -> Vec<PathBuf> {
    let paths: Vec<PathBuf> = buffer
        .lines()
        .filter(|line| !line.trim().is_empty())
        .take(IPC_PATH_COUNT_MAX as usize)
        .map(PathBuf::from)
        .collect();

    assert!(paths.len() <= IPC_PATH_COUNT_MAX as usize);

    paths
}

fn serve(listener: &TcpListener, sender: &MessageSender) {
    for connection in listener.incoming() {
        match connection {
            Ok(stream) => handle_connection(stream, sender),
            Err(error) => eprintln!("IPC connection error: {error}"),
        }
    }
}

pub fn spawn(sender: MessageSender) -> io::Result<()> {
    let listener = TcpListener::bind(IPC_ADDRESS)?;

    thread::Builder::new()
        .name("ipc".to_owned())
        .spawn(move || serve(&listener, &sender))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blank_lines_are_ignored() {
        let paths = parse_paths("/a\n\n  \n/b\r\n");

        assert_eq!(paths, vec![PathBuf::from("/a"), PathBuf::from("/b")]);
    }

    #[test]
    fn the_path_count_is_bounded() {
        let buffer = (0..IPC_PATH_COUNT_MAX + 10)
            .map(|index| format!("/path/{index}"))
            .collect::<Vec<String>>()
            .join("\n");

        assert_eq!(parse_paths(&buffer).len(), IPC_PATH_COUNT_MAX as usize);
    }
}
