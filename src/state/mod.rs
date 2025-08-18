use serde::{Deserialize, Serialize};
use rsa::RsaPublicKey;

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
    pub email: String,
    pub public_key: String
}

#[derive(Serialize, Deserialize)]
pub struct ContactInfoJSON {
    pub contact: Contact,
    pub contact_user: ContactUserJSON
}

pub struct ContactUser {
    pub id: u64,
    pub email: String,
    pub public_key: RsaPublicKey,
}

pub struct ContactInfo {
    pub contact: Contact,
    pub contact_user: ContactUser
}

pub mod login;
pub mod create_account;
pub mod chat;
