use super::{Task, TaskResult, TaskType};
use reqwest::header::{HeaderMap, HeaderValue};

pub struct FetchChatMessagesTask {
    chat_id: u64,
    offset: u64,
    token: String
}

impl FetchChatMessagesTask {
    pub fn new(chat_id: u64, offset: u64, token: String) -> Self {
        Self {
            chat_id,
            offset,
            token
        }
    }
}

impl Task for FetchChatMessagesTask {
    fn exec(&self, http_client: &crate::http::HttpClient) -> Result<TaskResult, std::io::Error> {
        let mut headers = HeaderMap::new();
        headers.insert("authToken", HeaderValue::from_str(&self.token).unwrap());

        let path = format!("chat_api/message/{}/{}", self.chat_id, self.offset);
        let response = http_client.get(&path, None, Some(headers));
        match response {
            Ok(r) => {
                let result = TaskResult::new(
                    r.status().as_u16(),
                    r.text().unwrap(),
                    None,
                    TaskType::FetchChatMessages
                );
                Ok(result)
            },
            Err(_) => Err(std::io::Error::new(std::io::ErrorKind::HostUnreachable, "Couldn't send request"))
        }
    }
}
