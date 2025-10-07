use super::{Task, TaskResult, TaskType};
use reqwest::header::{HeaderMap, HeaderValue};
use serde_json::json;

pub struct SendInviteContactTask {
    sender_id: u64,
    receiver_id: u64,
    receiver_email: String,
    token: String
}

impl SendInviteContactTask {
    pub fn new(sender_id: u64, receiver_id: u64, receiver_email: String, token: String) -> Self {
        Self { sender_id, receiver_id, receiver_email, token }
    }
}

impl Task for SendInviteContactTask {
    fn exec(&self, http_client: &crate::http::HttpClient) -> Result<TaskResult, std::io::Error> {
        let mut headers = HeaderMap::new();
        headers.insert("authToken", HeaderValue::from_str(&self.token).unwrap());

        let body = json!({
            "sender_id": self.sender_id,
            "receiver_id": self.receiver_id,
            "receiver_email": self.receiver_email
        });

        let response = http_client.post("user_api/contact/create", Some(body), Some(headers));
        match response {
            Ok(r) => {
                let result = TaskResult::new(
                    r.status().as_u16(),
                    r.text().unwrap(),
                    None,
                    TaskType::SendInviteContact
                );
                Ok(result)
            },
            Err(_) => Err(std::io::Error::new(std::io::ErrorKind::HostUnreachable, "Couldn't send request"))
        }
    }
}
