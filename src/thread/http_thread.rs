use crate::http::HttpClient;
use crate::task::{Task, TaskResult};
use std::collections::VecDeque;
use std::sync::mpsc::{Sender, Receiver};

pub struct TaskWrapper {
    task: Box<dyn Task>,
    result_channel: Sender<TaskResult>
}

impl TaskWrapper {
    pub fn new(task: Box<dyn Task>, result_channel: Sender<TaskResult>) -> Self {
        Self { task, result_channel }
    }
}

pub fn init_http_thread(http_thread_receiver: Receiver<TaskWrapper>) {
    let http_client = HttpClient::new("http://localhost:8080/");
    let mut task_queue: VecDeque<TaskWrapper> = VecDeque::new();

    loop {
        if !task_queue.is_empty() {
            if let Some(task_wrapper) = task_queue.pop_front() {
                match task_wrapper.task.exec(&http_client) {
                    Ok(result) => {
                        task_wrapper.result_channel.send(result).unwrap();
                    },
                    Err(_e) => {}
                }
            }
        }

        match http_thread_receiver.try_recv() {
            Ok(task_wrapper) => {
                task_queue.push_back(task_wrapper);
            },
            Err(_e) => {}
        }
    }
}
