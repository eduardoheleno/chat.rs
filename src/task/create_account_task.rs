use super::{Task, TaskResult, TaskType};
use crate::util::encryption::generate_assymetric_keypair;
use serde_json::json;
use rsa::pkcs1::EncodeRsaPublicKey;

pub struct CreateAccountTask {
    email: String,
    password: String
}

impl CreateAccountTask {
    pub fn new(email: String, password: String) -> Self {
        Self { email, password }
    }
}

impl Task for CreateAccountTask {
    fn exec(&self, http_client: &crate::http::HttpClient) -> Result<TaskResult, std::io::Error> {
        let keypair = generate_assymetric_keypair();
        let public_key_bytes = keypair.public_key
            .to_pkcs1_der()
            .expect("Failed to encode public key")
            .as_bytes()
            .to_vec();

        let body = json!({
            "email": self.email,
            "password": self.password,
            "public_key": public_key_bytes
        });

        let response = http_client.post("user/create", Some(body));
        match response {
            Ok(r) => {
                let result = TaskResult::new(
                    r.status().as_u16(),
                    r.text().unwrap(),
                    TaskType::CreateAccount
                );
                Ok(result)
            },
            Err(_) => Err(std::io::Error::new(std::io::ErrorKind::HostUnreachable, "Couldn't send request"))
        }
    }
}
