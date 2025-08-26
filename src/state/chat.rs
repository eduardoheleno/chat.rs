use crate::egui;
use crate::thread::http_thread::TaskWrapper;
use crate::task::fetch_contact_data_task::FetchContactDataTask;
use crate::task::TaskResult;
use crate::util::encryption::generate_cipher;
use super::ContactInfo;
use http::Uri;
use tungstenite::{connect, Message, ClientRequestBuilder};
use x25519_dalek::StaticSecret;
use serde::{Deserialize, Serialize};
use base64::prelude::*;
use x25519_dalek::PublicKey;
use std::sync::mpsc::{self, Sender, Receiver};

pub struct ChatState {
    pub token: String,
    pub private_key: Option<StaticSecret>,
    pub contacts: Vec<ContactInfo>,
    pub current_selected_id: u64,
    pub clicked_contact_id: Option<u64>,
    pub fetch_contact_error: String,
    pub test: String
}

// TODO: keep this structs in another place
#[derive(Serialize, Deserialize)]
struct ChatWithMessages {
    chat_id: u64
}

#[derive(Serialize, Deserialize)]
struct FetchContactDataSuccess {
    contact_id: u64,
    public_key: String,
    chat_with_messages: ChatWithMessages
}

#[derive(Serialize, Deserialize)]
struct FetchContactDataError {
    message: String
}

impl Default for ChatState {
    fn default() -> Self {
        Self {
            token: String::new(),
            private_key: None,
            contacts: Vec::new(),
            current_selected_id: 0,
            clicked_contact_id: None,
            fetch_contact_error: String::new(),
            test: String::new()
        }
    }
}

impl ChatState {
    pub fn show_chat_page(
        &mut self,
        http_thread: &Sender<TaskWrapper>,
        result_queue: &mut Vec<Receiver<TaskResult>>,
        ctx: &egui::Context
    ) {
        if self.fetch_contact_error.len() > 0 {
            self.show_error_modal(ctx);
        }
        egui::SidePanel::left("left_panel")
            .resizable(false)
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(5.);
                    ui.heading("Contacts");

                    ui.add_space(5.);
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        ui.with_layout(
                            egui::Layout::top_down(egui::Align::LEFT).with_cross_justify(true),
                            |ui| {
                                for contact in self.contacts.iter() {
                                    egui::Frame::NONE
                                        .show(ui, |ui| {
                                            let selected = self.current_selected_id == contact.contact_user.id;
                                            let response = ui.add(
                                                egui::Button::selectable(
                                                    selected,
                                                    egui::RichText::new(contact.contact_user.email.clone())
                                                ).wrap_mode(egui::TextWrapMode::Truncate)
                                            );
                                            if response.clicked() {
                                                self.clicked_contact_id = Some(contact.contact_user.id);
                                                self.current_selected_id = contact.contact_user.id;
                                            }
                                        });
                                }
                            }
                        );
                    });

                    if ui.button("debug").clicked() {
                        self.fetch_contact_error = "Teste".to_owned();
                    }
                });
            });

        egui::TopBottomPanel::bottom("bottom_panel")
            .resizable(false)
            .min_height(0.)
            .show_separator_line(false)
            .frame(egui::Frame {
                inner_margin: egui::Margin::same(1),
                outer_margin: egui::Margin::same(1),
                ..Default::default()
            })
            .show(ctx, |ui| {
                egui::TextEdit::multiline(&mut self.test)
                    .hint_text("Type your message")
                    .desired_width(ui.available_width())
                    .show(ui);
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.heading("Chat");
            });
        });

        self.handle_user_interaction(http_thread, result_queue);
    }

    fn show_error_modal(&mut self, ctx: &egui::Context) {
        egui::Modal::new(egui::Id::new("modal_error")).show(ctx, |ui| {
            ui.label(
                egui::RichText::new(self.fetch_contact_error.to_owned())
                    .color(egui::Color32::RED)
            );
            if ui.button("Ok").clicked() {
                self.fetch_contact_error.clear();
            }
        });
    }

    fn handle_user_interaction(&mut self, http_thread: &Sender<TaskWrapper>, result_queue: &mut Vec<Receiver<TaskResult>>) {
        self.fetch_contact_data(http_thread, result_queue);
    }

    fn fetch_contact_data(
        &mut self,
        http_thread: &Sender<TaskWrapper>,
        result_queue: &mut Vec<Receiver<TaskResult>>
    ) {
        if self.clicked_contact_id.is_none() {
            return;
        }

        let contact = self.contacts.iter_mut()
            .find(
                |contact_info|
                contact_info.contact_user.id == self.clicked_contact_id.unwrap()
            ).unwrap();
        if contact.chat_id.is_some() {
            self.clicked_contact_id = None;
            return;
        }

        let fetch_contact_data_task = FetchContactDataTask::new(
            contact.contact_user.id,
            self.token.clone()
        );
        let (task_channel_sender, task_channel_receiver) = mpsc::channel();
        let task_wrapper = TaskWrapper::new(
            Box::new(fetch_contact_data_task),
            task_channel_sender
        );

        http_thread.send(task_wrapper).unwrap();
        result_queue.push(task_channel_receiver);

        self.clicked_contact_id = None;
    }

    pub fn handle_task_fetch_contact_data(&mut self, task_result: TaskResult) {
        if task_result.status_code >= 400 {
            let parsed_error: FetchContactDataError = serde_json::from_str(task_result.response.as_ref()).unwrap();
            self.fetch_contact_error = parsed_error.message;
            return;
        }

        let parsed_response: FetchContactDataSuccess = serde_json::from_str(task_result.response.as_ref()).unwrap();
        let target_contact = self.contacts.iter_mut()
            .find(
                |contact_info|
                contact_info.contact_user.id == parsed_response.contact_id
            ).unwrap();
        let contact_public_key_bytes: [u8; 32] = BASE64_STANDARD
            .decode(parsed_response.public_key)
            .unwrap()
            .try_into()
            .unwrap();
        let public_key = PublicKey::from(contact_public_key_bytes);

        target_contact.chat_id = Some(parsed_response.chat_with_messages.chat_id);
        target_contact.cipher = Some(generate_cipher(public_key, self.private_key.clone().unwrap()));
    }
}
