use super::{Task, TaskResult, TaskType};
use reqwest::header::{HeaderMap, HeaderValue};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct SearchUserResponse {
    pub id: u64,
    pub email: String
}

pub struct SearchUserTask {
    search_param: String,
    token: String
}

impl SearchUserTask {
    pub fn new(search_param: String, token: String) -> Self {
        Self { search_param, token }
    }
}

impl Task for SearchUserTask {
    fn exec(&self, http_client: &crate::http::HttpClient) -> Result<super::TaskResult, std::io::Error> {
        let mut headers = HeaderMap::new();
        headers.insert("authToken", HeaderValue::from_str(&self.token).unwrap());

        let query = vec![format!("email={}", self.search_param)];

        let response = http_client.get("user/search", Some(query), Some(headers));
        match response {
            Ok(r) => {
                let result = TaskResult::new(
                    r.status().as_u16(),
                    r.text().unwrap(),
                    None,
                    TaskType::SearchUser
                );
                Ok(result)
            },
            Err(_) => Err(std::io::Error::new(std::io::ErrorKind::HostUnreachable, "Couldn't send request"))
        }
    }
}
