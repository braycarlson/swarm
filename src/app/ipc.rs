use std::io::Read;
use std::net::{TcpListener, TcpStream};
use std::path::PathBuf;
use std::sync::mpsc::Sender;
use std::thread;

use crate::app::message::{App, Msg};

pub struct IpcListener {
    bind_addr: String,
    max_connections: usize,
    sender: Sender<Msg>,
}

impl IpcListener {
    pub fn new(sender: Sender<Msg>) -> Self {
        Self {
            bind_addr: "127.0.0.1:44287".to_string(),
            max_connections: 1000,
            sender,
        }
    }

    pub fn spawn(self) -> Option<thread::JoinHandle<()>> {
        let handle = thread::spawn(move || {
            self.run();
        });

        Some(handle)
    }

    fn run(self) {
        let listener = match TcpListener::bind(&self.bind_addr) {
            Ok(listener) => listener,
            Err(error) => {
                eprintln!("Failed to bind IPC listener to {}: {}", self.bind_addr, error);
                return;
            }
        };

        for (index, stream) in listener.incoming().enumerate() {
            if index >= self.max_connections {
                break;
            }

            match stream {
                Ok(stream) => self.handle_connection(stream),
                Err(error) => {
                    eprintln!("IPC connection error: {}", error);
                    continue;
                }
            }
        }
    }

    fn handle_connection(&self, mut stream: TcpStream) {
        let mut buffer = String::new();

        if let Err(error) = stream.read_to_string(&mut buffer) {
            eprintln!("Failed to read from IPC stream: {}", error);
            return;
        }

        let paths = self.parse_paths(&buffer);

        if paths.is_empty() {
            return;
        }

        if self.sender.send(Msg::App(App::PathsReceivedFromIpc(paths))).is_err() {
            eprintln!("Failed to send IPC message: receiver disconnected");
        }
    }

    fn parse_paths(&self, buffer: &str) -> Vec<PathBuf> {
        buffer
            .lines()
            .filter(|line| !line.trim().is_empty())
            .map(PathBuf::from)
            .collect()
    }
}
