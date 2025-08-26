use crate::egui;
use crate::thread::http_thread::TaskWrapper;
use crate::task::login_task::LoginTask;
use crate::task::TaskResult;
use crate::state::Page;
use crate::util::keyring_handler::get_private_key;
use super::ContactInfoJSON;
use super::ContactInfo;
use super::ContactUser;
use super::chat::ChatState;
use x25519_dalek::StaticSecret;
use egui::{
    RichText,
    TextEdit,
    Color32
};
use serde::{Deserialize, Serialize};
use std::sync::mpsc::{self, Sender, Receiver};

#[derive(Serialize, Deserialize)]
struct HttpResponseError {
    message: String
}

#[derive(Serialize, Deserialize)]
struct HttpResponseSuccess {
    token: String,
    contacts: Vec<ContactInfoJSON>
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
                                let (task_channel_sender, task_channel_receiver) = mpsc::channel();
                                let task_wrapper = TaskWrapper::new(
                                    Box::new(login_task),
                                    task_channel_sender
                                );

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
        page_state: &mut Page
    ) {
        self.is_loading = false;

        if task_result.status_code >= 400 {
            let parsed_error: HttpResponseError = serde_json::from_str(task_result.response.as_ref()).unwrap();
            self.error = parsed_error.message;
            return;
        }

        let parsed_success: HttpResponseSuccess = serde_json::from_str(task_result.response.as_ref()).unwrap();
        let parsed_contacts = parsed_success.contacts.into_iter().map(|contact| {
            let contact_user = ContactUser {
                id: contact.contact_user.id,
                email: contact.contact_user.email
            };

            ContactInfo {
                contact: contact.contact,
                contact_user,
                chat_id: None,
                cipher: None
            }
        }).collect();
        let private_key_bytes: [u8; 32] = get_private_key(self.email.clone()).try_into().unwrap();
        let private_key = StaticSecret::from(private_key_bytes);

        chat_state.contacts = parsed_contacts;
        chat_state.token = parsed_success.token;
        chat_state.private_key = Some(private_key);

        *page_state = Page::Chat;
    }
}
