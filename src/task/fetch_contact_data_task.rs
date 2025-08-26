use super::{Task, TaskResult, TaskType};
use serde_json::json;
use reqwest::header::{HeaderMap, HeaderValue};

pub struct FetchContactDataTask {
    contact_id: u64,
    token: String
}

impl FetchContactDataTask {
    pub fn new(contact_id: u64, token: String) -> Self {
        Self { contact_id, token }
    }
}

impl Task for FetchContactDataTask {
    fn exec(&self, http_client: &crate::http::HttpClient) -> Result<TaskResult, std::io::Error> {
        let body = json!({
            "contact_id": self.contact_id
        });

        let mut headers = HeaderMap::new();
        headers.insert("authToken", HeaderValue::from_str(&self.token).unwrap());

        let response = http_client.post("contact/get-public-key", Some(body), Some(headers));
        match response {
            Ok(r) => {
                let result = TaskResult::new(
                    r.status().as_u16(),
                    r.text().unwrap(),
                    None,
                    TaskType::FetchContactData
                );
                Ok(result)
            },
            Err(_) => Err(std::io::Error::new(std::io::ErrorKind::HostUnreachable, "Couldn't send request"))
        }
    }
}
