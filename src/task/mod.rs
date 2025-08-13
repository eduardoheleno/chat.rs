use crate::http::HttpClient;

pub enum TaskType {
    Login,
    CreateAccount
}

pub struct TaskResult {
    pub status_code: u16,
    pub response: String,
    pub task_type: TaskType
}

impl TaskResult {
    pub fn new(status_code: u16, response: String, task_type: TaskType) -> Self {
        Self { status_code, response, task_type }
    }
}

pub trait Task: Send {
    fn exec(&self, http_client: &HttpClient) -> Result<TaskResult, std::io::Error>;
}

pub mod login_task;
pub mod create_account_task;
