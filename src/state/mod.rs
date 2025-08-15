use serde::{Deserialize, Serialize};

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
pub struct ContactUser {
    pub id: u64,
    pub email: String,
    #[serde(with="serde_bytes")]
    pub public_key: Vec<u8>
}

#[derive(Serialize, Deserialize)]
pub struct ContactInfo {
    pub contact: Contact,
    pub contact_user: ContactUser
}

pub mod login;
pub mod create_account;
pub mod chat;
