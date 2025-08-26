use serde::{Deserialize, Serialize};
use chacha20poly1305::XChaCha20Poly1305;

pub enum Page {
    Login,
    CreateAccount,
    Chat
}

#[derive(Serialize, Deserialize)]
pub struct Contact {
    pub id: u64,
    pub user1_id: u64,
    pub user2_id: u64,
    pub status: String
}

#[derive(Serialize, Deserialize)]
pub struct ContactUserJSON {
    pub id: u64,
    pub email: String
}

#[derive(Serialize, Deserialize)]
pub struct ContactInfoJSON {
    pub contact: Contact,
    pub contact_user: ContactUserJSON
}

pub struct ContactUser {
    pub id: u64,
    pub email: String
}

pub struct ContactInfo {
    pub contact: Contact,
    pub contact_user: ContactUser,
    pub chat_id: Option<u64>,
    pub cipher: Option<XChaCha20Poly1305>
}

pub mod login;
pub mod create_account;
pub mod chat;
