use crate::http::HttpClient;
use create_account_task::PrivateKeyParams;

pub enum TaskType {
    Login,
    CreateAccount,
    FetchContactData
}

pub struct TaskResult {
    pub status_code: u16,
    pub response: String,
    pub task_type: TaskType,
    pub private_key_params: Option<PrivateKeyParams>
}

impl TaskResult {
    pub fn new(
        status_code: u16,
        response: String,
        private_key_params: Option<PrivateKeyParams>,
        task_type: TaskType
    ) -> Self {
        Self { status_code, response, private_key_params, task_type }
    }
}

pub trait Task: Send {
    fn exec(&self, http_client: &HttpClient) -> Result<TaskResult, std::io::Error>;
}

pub mod login_task;
pub mod create_account_task;
pub mod fetch_contact_data_task;
