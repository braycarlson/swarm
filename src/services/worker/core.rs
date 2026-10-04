use std::sync::mpsc::{self, Receiver, SyncSender, TrySendError};
use std::thread;

use crate::constants::WORKER_QUEUE_COUNT_MAX;

pub trait WorkerTask: Send + 'static {
    type Command: Send + 'static;
    type Result: Send + 'static;

    fn process(&mut self, command: Self::Command, results: &SyncSender<Self::Result>);
}

pub struct Worker<Task: WorkerTask> {
    name: &'static str,
    receiver: Receiver<Task::Result>,
    sender: SyncSender<Task::Command>,
}

impl<Task: WorkerTask> Worker<Task> {
    pub fn send(&self, command: Task::Command) -> bool {
        match self.sender.try_send(command) {
            Ok(()) => true,
            Err(TrySendError::Full(_)) => {
                eprintln!("The {} worker queue is full; a command was dropped", self.name);

                false
            }
            Err(TrySendError::Disconnected(_)) => {
                eprintln!("The {} worker has stopped; a command was dropped", self.name);

                false
            }
        }
    }

    pub fn spawn(name: &'static str, task: Task) -> Self {
        assert_ne!(name, "");

        let capacity = WORKER_QUEUE_COUNT_MAX as usize;
        let (command_sender, command_receiver) = mpsc::sync_channel(capacity);
        let (result_sender, result_receiver) = mpsc::sync_channel(capacity);

        thread::Builder::new()
            .name(name.to_owned())
            .spawn(move || run(task, &command_receiver, &result_sender))
            .expect("the worker thread spawns");

        Self {
            name,
            receiver: result_receiver,
            sender: command_sender,
        }
    }

    pub fn try_recv(&self) -> Option<Task::Result> {
        self.receiver.try_recv().ok()
    }
}

fn run<Task>(mut task: Task, commands: &Receiver<Task::Command>, results: &SyncSender<Task::Result>)
where
    Task: WorkerTask,
{
    let mut connected = true;

    while connected {
        match commands.recv() {
            Ok(command) => task.process(command, results),
            Err(_) => connected = false,
        }
    }
}

#[cfg(test)]
mod tests {
    use core::time::Duration;

    use super::*;

    struct Doubler;

    impl WorkerTask for Doubler {
        type Command = u32;
        type Result = u32;

        fn process(&mut self, command: Self::Command, results: &SyncSender<Self::Result>) {
            let _ = results.send(command * 2);
        }
    }

    #[test]
    fn a_command_produces_a_result() {
        let worker = Worker::spawn("doubler", Doubler);

        assert!(worker.send(21));

        let mut result = None;

        for _ in 0..1_000 {
            result = worker.try_recv();

            if result.is_some() {
                break;
            }

            thread::sleep(Duration::from_millis(1));
        }

        assert_eq!(result, Some(42));
    }
}
