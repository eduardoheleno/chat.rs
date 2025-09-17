use crate::egui;
use crate::thread::http_thread::TaskWrapper;
use crate::task::TaskResult;
use crate::task::search_user_task::SearchUserTask;
use crate::task::search_user_task::SearchUserResponse;
use crate::task::send_invite_contact_task::SendInviteContactTask;
use crate::task::accept_invite_contact_task::AcceptInviteContactTask;
use crate::task::GenericResultError;
use crate::util::encryption::{encrypt_plain_text, generate_cipher};
use crate::thread::websocket_thread::{
    init_websocket,
    init_websocket_thread,
    ContentMessage,
    InviteMessage,
    AcceptInvite,
    MessageType
};
use super::ContactInfo;
use base64::prelude::*;
use serde_json::Value;
use tungstenite::{WebSocket, stream::MaybeTlsStream};
use serde::{Deserialize, Serialize};
use x25519_dalek::{StaticSecret, PublicKey};
use chacha20poly1305::aead::generic_array::GenericArray;
use chacha20poly1305::XNonce;
use chacha20poly1305::aead::Aead;
use std::cell::OnceCell;
use std::sync::mpsc::{Sender, Receiver};
use std::net::TcpStream;

pub struct ChatState {
    pub token: String,
    pub user_id: u64,
    pub private_key: Option<StaticSecret>,

    pub contacts: Vec<ContactInfo>,

    pub current_selected_id: u64,
    pub clicked_contact_id: Option<u64>,

    pub modal_error: String,
    pub typed_message: String,

    pub show_search_modal: bool,
    pub search_email: String,
    pub searched_users: Vec<SearchUserResponse>,
    pub should_search: bool,
    pub is_search_loading: bool,

    pub clicked_invite_contact_id: Option<u64>,
    pub clicked_invite_contact_email: Option<String>,
    pub is_send_invite_loading: bool,

    pub received_invites: Vec<InviteMessage>,
    pub sent_invites: Vec<InviteMessage>,
    pub accepted_contact_id: Option<u64>,

    pub show_invites_modal: bool,
    pub is_loading_accept_invite: bool,

    pub message_thread_sender: OnceCell<Sender<String>>,
    pub message_ui_receiver: OnceCell<Receiver<String>>
}

// TODO: keep this structs in another place
#[derive(Serialize, Deserialize)]
struct ChatWithMessages {
    chat_id: u64
}

#[derive(Serialize, Deserialize)]
struct FetchContactDataSuccess {
    contact_id: u64,
    chat_with_messages: ChatWithMessages
}

impl Default for ChatState {
    fn default() -> Self {
        Self {
            token: String::new(),
            user_id: 0,
            private_key: None,

            contacts: Vec::new(),

            current_selected_id: 0,
            clicked_contact_id: None,

            modal_error: String::new(),
            typed_message: String::new(),

            show_search_modal: false,
            search_email: String::new(),
            searched_users: Vec::new(),
            should_search: false,
            is_search_loading: false,

            clicked_invite_contact_id: None,
            clicked_invite_contact_email: None,
            is_send_invite_loading: false,

            received_invites: Vec::new(),
            sent_invites: Vec::new(),
            accepted_contact_id: None,

            show_invites_modal: false,
            is_loading_accept_invite: false,

            message_thread_sender: OnceCell::new(),
            message_ui_receiver: OnceCell::new()
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
        if self.modal_error.len() > 0 {
            self.show_error_modal(ctx);
        }
        if self.show_search_modal {
            self.show_search_modal(ctx);
        }
        if self.show_invites_modal {
            self.show_invites_modal(ctx);
        }

        egui::SidePanel::left("left_panel")
            .resizable(false)
            .show(ctx, |ui| {
                ui.add_space(5.);
                ui.horizontal_wrapped(|ui| {
                    if ui.button(format!("Invites ( {} )", self.received_invites.len())).clicked() {
                        self.show_invites_modal = true;
                    }
                    if ui.button("Search").clicked() {
                        self.show_search_modal = true;
                    }
                });
                ui.add_space(10.);

                ui.vertical(|ui| {
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
                                            let selected = self.current_selected_id == contact.contact.contact_id;
                                            let response = ui.add(
                                                egui::Button::selectable(
                                                    selected,
                                                    egui::RichText::new(contact.contact.contact_email.clone())
                                                ).wrap_mode(egui::TextWrapMode::Truncate)
                                            );
                                            if response.clicked() {
                                                self.clicked_contact_id = Some(contact.contact.contact_id);
                                                self.current_selected_id = contact.contact.contact_id;
                                            }
                                        });
                                }
                            }
                        );
                    });
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
                egui::TextEdit::multiline(&mut self.typed_message)
                    .hint_text("Type your message")
                    .desired_width(ui.available_width())
                    .show(ui);
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical(|ui| {
                ui.heading("Chat");
            });
        });

        self.handle_messages();
        self.handle_user_interaction(http_thread, result_queue, ctx);
    }

    fn show_error_modal(&mut self, ctx: &egui::Context) {
        egui::Modal::new(egui::Id::new("modal_error")).show(ctx, |ui| {
            ui.label(
                egui::RichText::new(self.modal_error.to_owned())
                    .color(egui::Color32::RED)
            );
            if ui.button("Ok").clicked() {
                self.modal_error.clear();
            }
        });
    }

    fn show_search_modal(&mut self, ctx: &egui::Context) {
        egui::Modal::new(egui::Id::new("modal_search")).show(ctx, |ui| {
            ui.set_width(250.);
            ui.horizontal(|ui| {
                ui.add(
                    egui::TextEdit::singleline(&mut self.search_email)
                        .hint_text("Search user by e-mail")
                );
                if ui.button("Search").clicked() {
                    if self.search_email.len() > 0 {
                        self.should_search = true;
                    }
                }
            });

            ui.add_space(8.);
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.set_height(100.);

                if self.is_search_loading || self.is_send_invite_loading || self.is_loading_accept_invite {
                    ui.vertical_centered(|ui| {
                        ui.spinner();
                    });
                    return;
                }
                if self.searched_users.is_empty() {
                    ui.label("No users were fetched.");
                    return;
                }

                for user in self.searched_users.iter() {
                     ui.horizontal(|ui| {
                        ui.label(user.email.clone());

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if self.sent_invites.iter().find(|i| i.receiver_id == user.id).is_some() {
                                ui.add_enabled(false, egui::Button::new("Sent"));
                            } else if let Some(invite) = self.received_invites.iter().find(|i| i.sender_id == user.id) {
                                if ui.button("Accept").clicked() {
                                    self.accepted_contact_id = Some(invite.id);
                                }
                            } else if let Some(_) = self.contacts.iter().find(|i| i.contact.contact_id == user.id) {
                                ui.add_enabled(false, egui::Button::new("Added"));
                            } else {
                                if ui.button("Add").clicked() {
                                    self.clicked_invite_contact_id = Some(user.id);
                                    self.clicked_invite_contact_email = Some(user.email.clone());
                                }
                            }
                        });
                    });
                    ui.separator();
                    ui.add_space(5.);
                }
            });
            ui.separator();

            if ui.button("Close").clicked() {
                self.show_search_modal = false;
            }
        });
    }

    fn show_invites_modal(&mut self, ctx: &egui::Context) {
        egui::Modal::new(egui::Id::new("modal_invite")).show(ctx, |ui| {
            ui.set_width(250.);

            // TODO: loading feedback
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.set_height(100.);

                if self.received_invites.is_empty() {
                    ui.label("No received invites.");
                    return;
                }

                for invite in self.received_invites.iter() {
                    ui.horizontal(|ui| {
                        ui.label(invite.sender_email.clone());

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.button("Accept").clicked() {
                                self.accepted_contact_id = Some(invite.id);
                            }
                        });
                    });
                    ui.separator();
                    ui.add_space(5.);
                }
            });
            ui.separator();

            if ui.button("Close").clicked() {
                self.show_invites_modal = false;
            }
        });
    }

    fn handle_user_interaction(
        &mut self,
        http_thread: &Sender<TaskWrapper>,
        result_queue: &mut Vec<Receiver<TaskResult>>,
        ctx: &egui::Context
    ) {
        self.handle_user_enter(ctx);
        if self.should_search {
            self.search_user(http_thread, result_queue);
        }
        if self.clicked_invite_contact_id.is_some() {
            self.send_invite(http_thread, result_queue);
        }
        if self.accepted_contact_id.is_some() {
            self.accept_invite(http_thread, result_queue);
        }
    }

    fn handle_user_enter(&mut self, ctx: &egui::Context) {
        ctx.input(|inp| {
            if inp.key_pressed(egui::Key::Enter) &&
                self.typed_message.len() > 0 &&
                self.current_selected_id > 0 {
                self.send_content_message();
            }
        });
    }

    fn search_user(
        &mut self,
        http_thread: &Sender<TaskWrapper>,
        result_queue: &mut Vec<Receiver<TaskResult>>
    ) {
        self.should_search = false;
        self.is_search_loading = true;

        let search_user_task = SearchUserTask::new(
            self.search_email.clone(),
            self.token.clone()
        );
        let (task_wrapper, task_channel_receiver) = TaskWrapper::new(Box::new(search_user_task));

        http_thread.send(task_wrapper).unwrap();
        result_queue.push(task_channel_receiver);
    }

    fn send_invite(
        &mut self,
        http_thread: &Sender<TaskWrapper>,
        result_queue: &mut Vec<Receiver<TaskResult>>
    ) {
        self.is_send_invite_loading = true;

        let receiver_id = self.clicked_invite_contact_id.take().unwrap();
        let receiver_email = self.clicked_invite_contact_email.take().unwrap();

        let send_invite_contact_task = SendInviteContactTask::new(
            self.user_id,
            receiver_id,
            receiver_email,
            self.token.clone()
        );
        let (task_wrapper, task_channel_receiver) = TaskWrapper::new(Box::new(send_invite_contact_task));

        http_thread.send(task_wrapper).unwrap();
        result_queue.push(task_channel_receiver);
    }

    fn accept_invite(
        &mut self,
        http_thread: &Sender<TaskWrapper>,
        result_queue: &mut Vec<Receiver<TaskResult>>
    ) {
        self.is_loading_accept_invite = true;

        let contact_id = self.accepted_contact_id.take().unwrap();

        let accept_invite_contact_task = AcceptInviteContactTask::new(
            contact_id,
            self.token.clone()
        );
        let (task_wrapper, task_channel_receiver) = TaskWrapper::new(Box::new(accept_invite_contact_task));

        http_thread.send(task_wrapper).unwrap();
        result_queue.push(task_channel_receiver);
    }

    fn handle_messages(&mut self) {
        // maybe decrypt actions should be outside main thread?
        let message_ui_receiver = self.message_ui_receiver.get().unwrap();
        if let Ok(msg) = message_ui_receiver.try_recv() {
            let parsed_msg: Value = serde_json::from_str(&msg).unwrap();
            if parsed_msg["type"] == MessageType::Content.as_str() {
                self.handle_content_message(msg);
            } else if parsed_msg["type"] == MessageType::Invite.as_str() {
                self.handle_invite_message(msg);
            } else if parsed_msg["type"] == MessageType::InviteAccepted.as_str() {
                self.handle_accept_invite_message(msg);
            }
        }
    }

    // TODO
    fn handle_content_message(&mut self, msg: String) {
        let received_message: ContentMessage = serde_json::from_str(&msg).unwrap();

        let contact = self.contacts.iter().find(|c| c.contact.contact_id == received_message.sender_id).unwrap();
        let nonce: XNonce = *GenericArray::from_slice(&received_message.nonce);
        let decrypted_message = contact.cipher.decrypt(&nonce, received_message.content.as_ref()).unwrap();
        println!("{}", String::from_utf8(decrypted_message).unwrap());
    }

    fn handle_invite_message(&mut self, msg: String) {
        self.received_invites.push(serde_json::from_str(&msg).unwrap());
    }

    fn handle_accept_invite_message(&mut self, msg: String) {
        let parsed_msg: AcceptInvite = serde_json::from_str(&msg).unwrap();
        let private_key = self.private_key.clone().unwrap();

        let contact_public_key_bytes: [u8; 32] = BASE64_STANDARD
            .decode(parsed_msg.contact.contact_public_key.clone())
            .unwrap()
            .try_into()
            .unwrap();
        let contact_public_key = PublicKey::from(contact_public_key_bytes);
        let cipher = generate_cipher(contact_public_key, private_key);

        self.sent_invites.retain(|i| i.id != parsed_msg.contact.id);
        self.contacts.push(
            ContactInfo {
                contact: parsed_msg.contact,
                cipher
            }
        );
    }

    fn get_mut_selected_contact(&mut self) -> &mut ContactInfo {
        self.contacts.iter_mut()
            .find(
                |contact_info|
                contact_info.contact.contact_id == self.current_selected_id
            ).unwrap()
    }

    fn send_content_message(&mut self) {
        let typed_message = self.typed_message.clone();
        let user_id = self.user_id.clone();
        let contact = self.get_mut_selected_contact();

        let (encrypted_content, nonce) = encrypt_plain_text(&contact.cipher, typed_message);

        // i dont know if its safe to send bytes serialized to JSON format
        // but it is what i am doing for now
        let ws_content_message = ContentMessage {
            sender_id: user_id,
            receiver_id: contact.contact.contact_id.clone(),
            chat_id: contact.contact.chat_id,
            receiver_email: contact.contact.contact_email.clone(),
            r#type: MessageType::Content,
            content: encrypted_content,
            nonce
        };
        let ws_content_message_json = serde_json::to_string(&ws_content_message).unwrap();

        let message_thread_sender = self.message_thread_sender.get().unwrap();
        message_thread_sender.send(ws_content_message_json).unwrap();

        self.typed_message.clear();
    }

    pub fn connect_websocket(&mut self, ctx: &egui::Context) -> Result<(), std::io::Error> {
        match init_websocket(self.token.clone()) {
            Ok(socket) => {
                self.websocket_thread(socket, ctx.to_owned());
                Ok(())
            },
            Err(e) => Err(e)
        }
    }

    fn websocket_thread(&mut self, mut socket: WebSocket<MaybeTlsStream<TcpStream>>, ctx: egui::Context) {
        let _ = match socket.get_mut() {
            tungstenite::stream::MaybeTlsStream::Plain(stream) => stream.set_nonblocking(true),
            _ => unimplemented!()
        };

        let (message_thread_sender, message_ui_receiver) = init_websocket_thread(socket, ctx);

        self.message_thread_sender.set(message_thread_sender).unwrap();
        self.message_ui_receiver.set(message_ui_receiver).unwrap();
    }

    fn handle_task_error(&mut self, response: String) {
        let parsed_error: GenericResultError = serde_json::from_str(&response).unwrap();
        self.modal_error = parsed_error.message;
        return;
    }

    pub fn handle_task_search_user(&mut self, task_result: TaskResult) {
        self.is_search_loading = false;

        if task_result.status_code >= 400 {
            self.handle_task_error(task_result.response);
            return;
        }

        let users: Vec<SearchUserResponse> = serde_json::from_str(&task_result.response).unwrap();
        self.searched_users = users;
    }

    pub fn handle_task_send_invite_contact(&mut self, task_result: TaskResult) {
        self.is_send_invite_loading = false;

        if task_result.status_code >= 400 {
            self.handle_task_error(task_result.response);
            return
        }

        self.sent_invites.push(serde_json::from_str(&task_result.response).unwrap());
    }

    pub fn handle_task_accept_invite_contact(&mut self, task_result: TaskResult, ctx: &egui::Context) {
        self.is_loading_accept_invite = false;

        if task_result.status_code >= 400 {
            self.handle_task_error(task_result.response);
            return
        }

        let response: AcceptInvite = serde_json::from_str(&task_result.response).unwrap();
        let private_key = self.private_key.clone().unwrap();

        let contact_public_key_bytes: [u8; 32] = BASE64_STANDARD
            .decode(response.contact.contact_public_key.clone())
            .unwrap()
            .try_into()
            .unwrap();
        let contact_public_key = PublicKey::from(contact_public_key_bytes);
        let cipher = generate_cipher(contact_public_key, private_key);

        self.received_invites.retain(|i| i.id != response.contact.id);
        self.contacts.push(
            ContactInfo {
                contact: response.contact,
                cipher
            }
        );
        ctx.request_repaint();
    }
}
