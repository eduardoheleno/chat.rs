use crate::http::HttpClient;
use crate::task::{Task, TaskResult};
use std::collections::VecDeque;
use std::sync::mpsc::{self, Sender, Receiver};

pub struct TaskWrapper {
    task: Box<dyn Task>,
    result_channel: Sender<TaskResult>
}

impl TaskWrapper {
    pub fn new(task: Box<dyn Task>) -> (Self, Receiver<TaskResult>) {
        let (task_channel_sender, task_channel_receiver) = mpsc::channel();
        (Self { task, result_channel: task_channel_sender }, task_channel_receiver)
    }
}

pub fn init_http_thread(http_thread_receiver: Receiver<TaskWrapper>) {
    let http_client = HttpClient::new("http://localhost/");
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
