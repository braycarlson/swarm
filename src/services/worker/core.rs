use std::sync::{Arc, Mutex};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;

pub trait WorkerTask: Send + 'static {
    type Command: Send + 'static;
    type Result: Send + 'static;

    fn process(&mut self, command: Self::Command, result_sender: &Sender<Self::Result>);
}

pub struct Worker<T: WorkerTask> {
    receiver: Arc<Mutex<Receiver<T::Result>>>,
    sender: Sender<T::Command>,
}

impl<T: WorkerTask> Worker<T> {
    pub fn spawn(mut task: T) -> Self {
        let (command_sender, command_receiver) = mpsc::channel();
        let (result_sender, result_receiver) = mpsc::channel();

        thread::spawn(move || {
            loop {
                let command = command_receiver.recv();

                if command.is_err() {
                    break;
                }

                task.process(command.unwrap(), &result_sender);
            }
        });

        Self {
            receiver: Arc::new(Mutex::new(result_receiver)),
            sender: command_sender,
        }
    }

    pub fn send(&self, command: T::Command) -> bool {
        self.sender.send(command).is_ok()
    }

    pub fn try_recv(&self) -> Option<T::Result> {
        let receiver = self.receiver.lock().ok()?;
        receiver.try_recv().ok()
    }
}

impl<T: WorkerTask> Clone for Worker<T> {
    fn clone(&self) -> Self {
        Self {
            receiver: Arc::clone(&self.receiver),
            sender: self.sender.clone(),
        }
    }
}
