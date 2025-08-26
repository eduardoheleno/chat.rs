use super::{Task, TaskResult, TaskType};
use crate::util::encryption::generate_assymetric_keypair;
use serde_json::json;
use x25519_dalek::StaticSecret;

pub struct CreateAccountTask {
    email: String,
    password: String
}

impl CreateAccountTask {
    pub fn new(email: String, password: String) -> Self {
        Self { email, password }
    }
}

pub struct PrivateKeyParams {
    pub email: String,
    pub private_key: StaticSecret
}

impl PrivateKeyParams {
    pub fn new(email: String, private_key: StaticSecret) -> Self {
        Self { email, private_key }
    }
}

impl Task for CreateAccountTask {
    fn exec(&self, http_client: &crate::http::HttpClient) -> Result<TaskResult, std::io::Error> {
        let keypair = generate_assymetric_keypair();
        let public_key_bytes = keypair.public_key.as_bytes();

        let body = json!({
            "email": self.email,
            "password": self.password,
            "public_key": public_key_bytes
        });

        let response = http_client.post("user/create", Some(body), None);
        match response {
            Ok(r) => {
                let private_key_params = PrivateKeyParams::new(self.email.clone(), keypair.private_key);
                let result = TaskResult::new(
                    r.status().as_u16(),
                    r.text().unwrap(),
                    Some(private_key_params),
                    TaskType::CreateAccount
                );
                Ok(result)
            },
            Err(_) => Err(std::io::Error::new(std::io::ErrorKind::HostUnreachable, "Couldn't send request"))
        }
    }
}
