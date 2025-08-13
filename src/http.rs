use reqwest::{
    Url,
    blocking::{Client, Response},
    Error
};
use serde_json::Value;

pub struct HttpClient {
    base_url: Url,
    http_client: Client
}

impl HttpClient {
    pub fn new(base_url: &str) -> Self {
        Self {
            base_url: base_url.parse::<Url>().expect("Bad base_url"),
            http_client: Client::new()
        }
    }

    pub fn post(&self, path: &str, body: Option<Value>) -> Result<Response, Error> {
        let full_url = self.base_url.join(path).expect("Bad path");
        self.http_client.post(full_url)
            .json(&body)
            .send()
    }
}
