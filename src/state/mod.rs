use serde::{Deserialize, Serialize};
use chacha20poly1305::XChaCha20Poly1305;
use std::collections::LinkedList;

pub enum Page {
    Login,
    CreateAccount,
    Chat
}

#[derive(Serialize, Deserialize)]
pub struct ContactInfoJSON {
    pub id: u64,
    pub contact_id: u64,
    pub chat_id: u64,
    pub contact_email: String,
    pub contact_public_key: String
}

pub struct Message {
    pub id: Option<u64>,
    pub content: String,
    pub sender_id: u64
}

#[derive(Serialize, Deserialize)]
pub struct FetchMessage {
    pub id: u64,
    pub user_id: u64,
    pub chat_id: u64,
    pub content: String,
    pub nonce: String
}

pub struct ContactInfo {
    pub contact: ContactInfoJSON,
    pub cipher: XChaCha20Poly1305,
    pub messages: LinkedList<Message>
}

#[derive(Serialize, Deserialize)]
pub struct ChatInfoJSON {
    pub id: u64
}

pub mod login;
pub mod create_account;
pub mod chat;
