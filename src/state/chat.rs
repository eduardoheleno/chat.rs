use crate::egui;
use super::ContactInfo;
use egui::{
    RichText,
    // TextEdit,
    // Color32
};

pub struct ChatState {
    pub token: String,
    pub contacts: Vec<ContactInfo>
}

impl Default for ChatState {
    fn default() -> Self {
        Self {
            token: String::new(),
            contacts: Vec::new()
        }
    }
}

impl ChatState {
    pub fn show_chat_page(
        &mut self,
        ctx: &egui::Context
    ) {
       egui::CentralPanel::default().show(ctx, |ui| {
           ui.label(RichText::new("Chat"));
       });
    }
}
