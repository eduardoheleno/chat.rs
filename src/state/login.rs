use crate::egui;
use crate::thread::http_thread::TaskWrapper;
use crate::task::login_task::LoginTask;
use crate::task::TaskResult;
use crate::state::Page;
use crate::util::keyring_handler::get_private_key;
use crate::util::encryption::generate_cipher;
use super::ContactInfoJSON;
use super::ContactInfo;
use super::chat::ChatState;
use crate::thread::websocket_thread::InviteMessage;
use base64::prelude::*;
use x25519_dalek::{StaticSecret, PublicKey};
use egui::{
    RichText,
    TextEdit,
    Color32
};
use serde::{Deserialize, Serialize};
use std::sync::mpsc::{Sender, Receiver};

#[derive(Serialize, Deserialize)]
struct HttpResponseError {
    message: String
}

#[derive(Serialize, Deserialize)]
struct HttpResponseSuccess {
    token: String,
    user_id: u64,
    contacts: Vec<ContactInfoJSON>,
    pending_sent_invites: Vec<InviteMessage>,
    pending_received_invites: Vec<InviteMessage>
}

pub struct LoginState {
    pub email: String,
    pub password: String,
    pub error: String,
    pub is_loading: bool
}

impl Default for LoginState {
    fn default() -> Self {
        Self {
            email: String::new(),
            password: String::new(),
            error: String::new(),
            is_loading: false
        }
    }
}

impl LoginState {
    pub fn show_login_page(
        &mut self,
        http_thread: &Sender<TaskWrapper>,
        result_queue: &mut Vec<Receiver<TaskResult>>,
        current_page: &mut Page,
        ctx: &egui::Context
    ) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(50.);
                ui.label(RichText::new("Login").size(20.));
                ui.add_space(20.);

                ui.scope(|ui| {
                    ui.set_width(250.);
                    ui.add(
                        TextEdit::singleline(&mut self.email)
                            .hint_text("E-mail")
                    );

                    ui.add(
                        TextEdit::singleline(&mut self.password)
                            .hint_text("Password")
                            .password(true)
                    );

                    ui.add_space(15.);

                    if self.is_loading {
                        ui.spinner();
                    } else {
                        ui.columns(2, |columns| {
                            let create_account_button = egui::Button::new("Create account");
                            if columns[0].add(create_account_button).clicked() {
                                *current_page = Page::CreateAccount;
                            }

                            let login_button = egui::Button::new("Login");
                            if columns[1].add_enabled(!self.is_loading, login_button).clicked() {
                                self.error.clear();
                                self.is_loading = true;

                                let login_task = LoginTask::new(self.email.to_owned(), self.password.to_owned());
                                let (task_wrapper, task_channel_receiver) = TaskWrapper::new(Box::new(login_task));

                                http_thread.send(task_wrapper).unwrap();
                                result_queue.push(task_channel_receiver);
                            }
                        });
                    }

                    ui.label(
                        RichText::new(self.error.to_owned())
                            .color(Color32::RED)
                    );
                });
            });
        });
    }

    pub fn handle_task_login(
        &mut self,
        task_result: TaskResult,
        chat_state: &mut ChatState,
        page_state: &mut Page,
        ctx: &egui::Context
    ) {
        self.is_loading = false;

        if task_result.status_code >= 400 {
            let parsed_error: HttpResponseError = serde_json::from_str(task_result.response.as_ref()).unwrap();
            self.error = parsed_error.message;
            return;
        }

        let private_key_bytes: [u8; 32] = get_private_key(self.email.clone()).try_into().unwrap();
        let private_key = StaticSecret::from(private_key_bytes);

        let parsed_success: HttpResponseSuccess = serde_json::from_str(task_result.response.as_ref()).unwrap();
        let parsed_contacts = parsed_success.contacts.into_iter().map(|contact| {
            let contact_public_key_bytes: [u8; 32] = BASE64_STANDARD
                .decode(contact.contact_public_key.clone())
                .unwrap()
                .try_into()
                .unwrap();
            let contact_public_key = PublicKey::from(contact_public_key_bytes);
            let cipher = generate_cipher(contact_public_key, private_key.clone());

            ContactInfo {
                contact,
                cipher,
                messages: Vec::new()
            }
        }).collect();

        chat_state.contacts = parsed_contacts;
        chat_state.token = parsed_success.token;
        chat_state.user_id = parsed_success.user_id;
        chat_state.sent_invites = parsed_success.pending_sent_invites;
        chat_state.received_invites = parsed_success.pending_received_invites;
        chat_state.private_key = Some(private_key);

        match chat_state.connect_websocket(ctx) {
            Ok(()) => *page_state = Page::Chat,
            Err(e) => self.error = e.to_string()
        }
    }
}
