use std::sync::{Arc, Mutex};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;

use super::command::TaskCommand;
use super::result::TaskResult;

pub struct TaskExecutor<T, R>
where
    T: Send + 'static,
    R: Send + Clone + 'static,
{
    event_receiver: Arc<Mutex<Receiver<TaskResult<R>>>>,
    task_sender: Sender<TaskCommand<T, R>>,
}

impl<T, R> TaskExecutor<T, R>
where
    T: Send + 'static,
    R: Send + Clone + 'static,
{
    pub fn new() -> Self {
        let (task_sender, task_receiver) = mpsc::channel();
        let (event_sender, event_receiver) = mpsc::channel();

        thread::spawn(move || {
            Self::worker_thread(task_receiver, event_sender);
        });

        Self {
            event_receiver: Arc::new(Mutex::new(event_receiver)),
            task_sender,
        }
    }

    pub fn check_events(&self) -> Vec<TaskResult<R>> {
        let mut results = Vec::new();

        if let Ok(receiver) = self.event_receiver.lock() {
            while let Ok(result) = receiver.try_recv() {
                results.push(result);
            }
        }

        results
    }

    pub fn execute<F>(&self, id: &str, params: T, task_function: F) -> Result<(), String>
    where
        F: FnOnce(T) -> Result<R, String> + Send + 'static,
    {
        let command = TaskCommand::Execute(id.to_string(), params, Box::new(task_function));

        self.task_sender
            .send(command)
            .map_err(|error| format!("Failed to send task command: {}", error))
    }

    fn worker_thread(task_receiver: Receiver<TaskCommand<T, R>>, event_sender: Sender<TaskResult<R>>) {
        while let Ok(command) = task_receiver.recv() {
            match command {
                TaskCommand::Execute(id, params, task_function) => {
                    let _ = event_sender.send(TaskResult::Started(id.clone()));

                    let event_sender_clone = event_sender.clone();
                    let id_clone = id.clone();

                    thread::spawn(move || match task_function(params) {
                        Ok(result) => {
                            let _ = event_sender_clone.send(TaskResult::Completed(id_clone, result));
                        }
                        Err(error) => {
                            let _ = event_sender_clone.send(TaskResult::Error(id_clone, error));
                        }
                    });
                }
                TaskCommand::Cancel(_) => {}
            }
        }
    }
}

impl<T, R> Clone for TaskExecutor<T, R>
where
    T: Send + 'static,
    R: Send + Clone + 'static,
{
    fn clone(&self) -> Self {
        Self {
            event_receiver: Arc::clone(&self.event_receiver),
            task_sender: self.task_sender.clone(),
        }
    }
}
