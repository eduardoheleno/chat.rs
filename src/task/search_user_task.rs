use super::{Task, TaskResult, TaskType};
use reqwest::header::{HeaderMap, HeaderValue};

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
        let url = format!("user/search?email={}", self.search_param);
        let mut headers = HeaderMap::new();
        headers.insert("authToken", HeaderValue::from_str(&self.token).unwrap());

        let response = http_client.post(&url, None, Some(headers));
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
