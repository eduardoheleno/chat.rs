use super::{Task, TaskResult, TaskType};
use serde_json::json;

pub struct LoginTask {
    email: String,
    password: String
}

impl LoginTask {
    pub fn new(email: String, password: String) -> Self {
        Self { email, password }
    }
}

impl Task for LoginTask {
    fn exec(&self, http_client: &crate::http::HttpClient) -> Result<TaskResult, std::io::Error> {
        let body = json!({
            "email": self.email,
            "password": self.password
        });

        let response = http_client.post("user_api/user/login", Some(body), None);
        match response {
            Ok(r) => {
                let result = TaskResult::new(
                    r.status().as_u16(),
                    r.text().unwrap(),
                    None,
                    TaskType::Login
                );
                Ok(result)
            },
            Err(_) => Err(std::io::Error::new(std::io::ErrorKind::HostUnreachable, "Couldn't send request"))
        }
    }
}
