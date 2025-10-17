use std::io::Read;
use std::net::TcpListener;
use std::sync::mpsc::Sender;

use crate::app::message::{App, Msg};

const MAX_IPC_CONNECTIONS: u32 = 1000;
const IPC_BIND_ADDR: &str = "127.0.0.1:44287";

pub fn spawn_ipc_listener(ipc_sender: Sender<Msg>) -> Option<std::thread::JoinHandle<()>> {
    let handle = std::thread::spawn(move || {
        run_ipc_listener(ipc_sender);
    });

    Some(handle)
}

fn run_ipc_listener(ipc_sender: Sender<Msg>) {
    let listener_result = TcpListener::bind(IPC_BIND_ADDR);

    if listener_result.is_err() {
        return;
    }

    let listener = listener_result.unwrap();
    let mut connection_count: u32 = 0;

    for stream_result in listener.incoming() {
        if connection_count >= MAX_IPC_CONNECTIONS {
            break;
        }

        connection_count = connection_count + 1;

        if stream_result.is_err() {
            continue;
        }

        let mut stream = stream_result.unwrap();
        let mut buffer = String::new();

        let read_result = stream.read_to_string(&mut buffer);

        if read_result.is_err() {
            continue;
        }

        let paths: Vec<std::path::PathBuf> = buffer
            .lines()
            .map(std::path::PathBuf::from)
            .collect();

        if paths.is_empty() {
            continue;
        }

        let send_result = ipc_sender.send(Msg::App(App::PathsReceivedFromIpc(paths)));

        if send_result.is_err() {
            break;
        }
    }
}
