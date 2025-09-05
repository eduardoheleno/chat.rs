use http::Uri;
use std::net::TcpStream;
use std::sync::mpsc::{self, Sender, Receiver};
use std::thread;
use tungstenite::{
    connect,
    ClientRequestBuilder,
    Message,
    WebSocket,
    stream::MaybeTlsStream
};
use serde::{Deserialize, Serialize};
use eframe::egui;

#[derive(Serialize, Deserialize)]
pub struct WsContentMessage {
    pub sender_id: u64,
    pub receiver_id: u64,
    pub chat_id: u64,
    pub receiver_email: String,
    pub r#type: MessageType,
    pub content: Vec<u8>,
    pub nonce: [u8; 24]
}

#[derive(Serialize, Deserialize)]
pub struct WsInviteMessage {
    pub sender_id: u64,
    pub receiver_id: u64,
    pub receiver_email: String,
    pub r#type: MessageType
}

#[derive(Serialize, Deserialize)]
pub enum MessageType {
    Content,
    Invite
}

impl MessageType {
    pub fn as_str(self) -> String {
        match self {
            Self::Content => "Content".to_string(),
            Self::Invite => "Invite".to_string()
        }
    }
}

pub fn init_websocket(jwt_token: String) -> Result<WebSocket<MaybeTlsStream<TcpStream>>, std::io::Error> {
    let uri: Uri = "ws://localhost:8000/ws".parse().unwrap();
    let request = ClientRequestBuilder::new(uri)
        .with_header("authToken", jwt_token);

    match connect(request) {
        Ok((socket, response)) => {
            if response.status() != 101 {
                Err(std::io::Error::new(std::io::ErrorKind::Other, "Server denied connection upgrade"))
            } else {
                Ok(socket)
            }
        },
        Err(_) => Err(std::io::Error::new(std::io::ErrorKind::Other, "Couldn't send websocket handshake"))
    }
}

pub fn init_websocket_thread(
    mut socket: WebSocket<MaybeTlsStream<TcpStream>>,
    ctx: egui::Context
) -> (Sender<String>, Receiver<String>)
{
    let (message_thread_sender, message_thread_receiver)
        :(Sender<String>, Receiver<String>) = mpsc::channel();
    let (message_ui_sender, message_ui_receiver)
        :(Sender<String>, Receiver<String>) = mpsc::channel();

    thread::spawn(move || {
        loop {
            match socket.read() {
                Ok(msg) => {
                    message_ui_sender.send(msg.to_string()).unwrap();
                    ctx.request_repaint();
                },
                Err(_e) => {
                }
            }

            if let Ok(msg) = message_thread_receiver.try_recv() {
                socket.send(Message::Text(msg.into())).unwrap();
            }
        }
    });

    (message_thread_sender, message_ui_receiver)
}
