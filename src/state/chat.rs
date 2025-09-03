use crate::egui;
use crate::thread::http_thread::TaskWrapper;
use crate::task::TaskResult;
use crate::task::search_user_task::SearchUserTask;
use crate::util::encryption::encrypt_plain_text;
use super::ContactInfo;
use http::Uri;
use tungstenite::{connect, ClientRequestBuilder, Message};
use tungstenite::{WebSocket, stream::MaybeTlsStream};
use x25519_dalek::StaticSecret;
use serde::{Deserialize, Serialize};
use chacha20poly1305::aead::generic_array::GenericArray;
use chacha20poly1305::XNonce;
use chacha20poly1305::aead::Aead;
use std::cell::OnceCell;
use std::sync::mpsc::{self, Sender, Receiver};
use std::net::TcpStream;
use std::thread;

pub struct ChatState {
    pub token: String,
    pub user_id: u64,
    pub private_key: Option<StaticSecret>,

    pub contacts: Vec<ContactInfo>,

    pub current_selected_id: u64,
    pub clicked_contact_id: Option<u64>,

    pub fetch_contact_error: String,
    pub typed_message: String,

    pub show_search_modal: bool,
    pub search_email: String,
    pub should_search: bool,
    pub is_search_loading: bool,

    pub message_thread_sender: OnceCell<Sender<WsMessage>>,
    pub message_ui_receiver: OnceCell<Receiver<WsMessage>>
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

#[derive(Serialize, Deserialize)]
struct FetchContactDataError {
    message: String
}

#[derive(Serialize, Deserialize)]
pub struct WsMessage {
    sender_id: u64,
    receiver_id: u64,
    chat_id: u64,
    receiver_email: String,
    r#type: String,
    content: Vec<u8>,
    nonce: [u8; 24]
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

            fetch_contact_error: String::new(),
            typed_message: String::new(),

            show_search_modal: false,
            search_email: String::new(),
            should_search: false,
            is_search_loading: false,

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
        if self.fetch_contact_error.len() > 0 {
            self.show_error_modal(ctx);
        }
        if self.show_search_modal {
            self.show_search_modal(ctx);
        }

        egui::SidePanel::left("left_panel")
            .resizable(false)
            .show(ctx, |ui| {
                ui.add_space(5.);
                ui.horizontal_wrapped(|ui| {
                    ui.button("Invites ( )");
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
                        // println!("{}", self.typed_message);
                        self.send_message();
                        // self.test();
                        // self.fetch_contact_error = "Teste".to_owned();
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

            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.set_height(100.);
            });
            ui.separator();

            if ui.button("Close").clicked() {
                self.show_search_modal = false;
            }
        });
    }

    fn handle_user_interaction(
        &mut self,
        http_thread: &Sender<TaskWrapper>,
        result_queue: &mut Vec<Receiver<TaskResult>>
    ) {
        if self.should_search {
            self.handle_search_user(http_thread, result_queue);
        }
    }

    fn handle_search_user(
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

    fn handle_messages(&mut self) {
        // maybe decrypt actions should be outside main thread?
        let message_ui_receiver = self.message_ui_receiver.get().unwrap();
        if let Ok(msg) = message_ui_receiver.try_recv() {
            let nonce: XNonce = *GenericArray::from_slice(&msg.nonce);
            let contact = self.contacts.iter_mut()
                .find(
                    |contact_info|
                    contact_info.contact_user.id == msg.sender_id
                ).unwrap();
            let decrypted_message = contact.cipher.decrypt(&nonce, msg.content.as_ref()).unwrap();
            println!("{}", String::from_utf8(decrypted_message).unwrap());
        }
    }

    fn get_mut_selected_contact(&mut self) -> &mut ContactInfo {
        self.contacts.iter_mut()
            .find(
                |contact_info|
                contact_info.contact_user.id == self.current_selected_id
            ).unwrap()
    }

    fn send_message(&mut self) {
        if self.typed_message.len() <= 0 || self.current_selected_id <= 0 {
            return;
        }

        let typed_message = self.typed_message.clone();
        let user_id = self.user_id.clone();
        let contact = self.get_mut_selected_contact();

        let (encrypted_content, nonce) = encrypt_plain_text(&contact.cipher, typed_message);

        // i dont know if its safe to send bytes serialized to JSON format
        // but it is what i am doing for now
        let ws_message = WsMessage {
            sender_id: user_id,
            receiver_id: contact.contact_user.id.clone(),
            chat_id: 1,
            receiver_email: contact.contact_user.email.clone(),
            r#type: "content".to_string(),
            content: encrypted_content,
            nonce
        };
        let message_thread_sender = self.message_thread_sender.get().unwrap();
        message_thread_sender.send(ws_message).unwrap();
    }

    pub fn connect_websocket(&mut self, ctx: &egui::Context) -> Result<(), std::io::Error> {
        let uri: Uri = "ws://localhost:8000/ws".parse().unwrap();
        let request = ClientRequestBuilder::new(uri)
            .with_header("Authorization", self.token.clone());

        match connect(request) {
            Ok((socket, response)) => {
                if response.status() != 101 {
                    Err(std::io::Error::new(std::io::ErrorKind::Other, "Server denied connection upgrade"))
                } else {
                    self.websocket_thread(socket, ctx.to_owned());
                    Ok(())
                }
            },
            Err(_) => Err(std::io::Error::new(std::io::ErrorKind::Other, "Couldn't send websocket handshake"))
        }
    }

    fn websocket_thread(&mut self, mut socket: WebSocket<MaybeTlsStream<TcpStream>>, ctx: egui::Context) {
        let (message_thread_sender, message_thread_receiver): (Sender<WsMessage>, Receiver<WsMessage>) = mpsc::channel();
        let (message_ui_sender, message_ui_receiver): (Sender<WsMessage>, Receiver<WsMessage>) = mpsc::channel();

        let _ = match socket.get_mut() {
            tungstenite::stream::MaybeTlsStream::Plain(stream) => stream.set_nonblocking(true),
            _ => unimplemented!()
        };

        self.message_thread_sender.set(message_thread_sender).unwrap();
        self.message_ui_receiver.set(message_ui_receiver).unwrap();

        thread::spawn(move || {
            loop {
                match socket.read() {
                    Ok(msg) => {
                        let test: WsMessage = serde_json::from_str(&msg.to_string()).unwrap();
                        message_ui_sender.send(test).unwrap();
                        ctx.request_repaint();
                    },
                    Err(_e) => {
                    }
                }

                if let Ok(msg) = message_thread_receiver.try_recv() {
                    let json_ws_message = serde_json::to_string(&msg).unwrap();
                    socket.send(Message::Text(json_ws_message.into())).unwrap();
                }
            }
        });
    }

    pub fn handle_task_fetch_contact_data(&mut self, task_result: TaskResult) {
        if task_result.status_code >= 400 {
            let parsed_error: FetchContactDataError = serde_json::from_str(task_result.response.as_ref()).unwrap();
            self.fetch_contact_error = parsed_error.message;
            return;
        }

        let parsed_response: FetchContactDataSuccess = serde_json::from_str(task_result.response.as_ref()).unwrap();
        let target_contact = self.get_mut_selected_contact();

        target_contact.chat_id = Some(parsed_response.chat_with_messages.chat_id);
    }
}
