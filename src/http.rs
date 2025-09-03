use reqwest::{
    Url,
    header::HeaderMap,
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

    pub fn post(&self, path: &str, body: Option<Value>, headers: Option<HeaderMap>) -> Result<Response, Error> {
        let full_url = self.base_url.join(path).expect("Bad path");
        let mut request_builder = self.http_client.post(full_url);

        if let Some(h) = headers {
            request_builder = request_builder.headers(h);
        }

        request_builder
            .json(&body)
            .send()
    }

    // pub fn get(&self, path: &str, headers: Option<HeaderMap>) -> Result<Response, Error> {
    //     let full_url = self.base_url.join(path).expect("Bad path");
    //     full_url.set_query()
    // }
}
