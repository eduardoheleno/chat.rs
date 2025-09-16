use serde::{Deserialize, Serialize};
use chacha20poly1305::XChaCha20Poly1305;

pub enum Page {
    Login,
    CreateAccount,
    Chat
}

#[derive(Serialize, Deserialize)]
pub struct ContactInfoJSON {
    pub id: u64,
    pub contact_id: u64,
    pub contact_email: String,
    pub contact_public_key: String
}

pub struct ContactInfo {
    pub contact: ContactInfoJSON,
    pub chat_id: Option<u64>,
    pub cipher: XChaCha20Poly1305
}

#[derive(Serialize, Deserialize)]
pub struct ChatInfoJSON {
    pub id: u64
}

#[derive(Serialize, Deserialize)]
pub struct AcceptInviteJSON {
    pub chat: ChatInfoJSON,
    pub contact: ContactInfoJSON
}

#[derive(Serialize, Deserialize)]
pub struct InviteMessage {
    pub id: u64,
    pub sender_id: u64,
    pub sender_email: String,
    pub receiver_id: u64,
    pub receiver_email: String,
    pub status: String
}

pub mod login;
pub mod create_account;
pub mod chat;
