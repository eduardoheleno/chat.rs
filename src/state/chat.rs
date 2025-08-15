use super::Contact;

pub struct ChatState {
    pub token: String,
    pub contacts: Vec<Contact>
}

impl Default for ChatState {
    fn default() -> Self {
        Self {
            token: String::new(),
            contacts: Vec::new()
        }
    }
}
