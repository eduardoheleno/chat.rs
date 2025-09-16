use super::{Task, TaskResult, TaskType};
use reqwest::header::{HeaderMap, HeaderValue};

pub struct AcceptInviteContactTask {
    contact_id: u64,
    token: String
}

impl AcceptInviteContactTask {
    pub fn new(contact_id: u64, token: String) -> Self {
        Self { contact_id, token }
    }
}

impl Task for AcceptInviteContactTask {
    fn exec(&self, http_client: &crate::http::HttpClient) -> Result<TaskResult, std::io::Error> {
        let mut headers = HeaderMap::new();
        headers.insert("authToken", HeaderValue::from_str(&self.token).unwrap());

        let path = format!("contact/accept-invite/{}", self.contact_id);
        let response = http_client.post(&path, None, Some(headers));
        match response {
            Ok(r) => {
                let result = TaskResult::new(
                    r.status().as_u16(),
                    r.text().unwrap(),
                    None,
                    TaskType::AcceptInviteContact
                );
                Ok(result)
            },
            Err(_) => Err(std::io::Error::new(std::io::ErrorKind::HostUnreachable, "Couldn't send request"))
        }
    }
}
