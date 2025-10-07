use std::sync::mpsc::{self, Sender, Receiver};
use eframe::egui;
use super::task::TaskResult;
use super::task::TaskType;
use super::state::login::LoginState;
use super::state::create_account::CreateAccountState;
use super::state::chat::ChatState;
use super::state::Page;
use super::thread::http_thread::{init_http_thread, TaskWrapper};

pub struct App {
    current_page: Page,
    login_state: LoginState,
    create_account_state: CreateAccountState,
    chat_state: ChatState,
    http_thread: Sender<TaskWrapper>,
    result_queue: Vec<Receiver<TaskResult>>
}

impl Default for App {
    fn default() -> Self {
        let (http_thread_sender, http_thread_receiver) = mpsc::channel();
        std::thread::spawn(|| init_http_thread(http_thread_receiver));

        Self {
            current_page: Page::Login,
            login_state: LoginState::default(),
            create_account_state: CreateAccountState::default(),
            chat_state: ChatState::default(),
            http_thread: http_thread_sender,
            result_queue: Vec::new()
        }
    }
}

impl App {
    fn process_result_queue(&mut self, ctx: &egui::Context) {
        for i in 0..self.result_queue.len() {
            match self.result_queue[i].try_recv() {
                Ok(task_result) => {
                    match task_result.task_type {
                        TaskType::Login => self.login_state.handle_task_login(
                            task_result,
                            &mut self.chat_state,
                            &mut self.current_page,
                            ctx
                        ),
                        TaskType::CreateAccount => self.create_account_state.handle_task_create_account(task_result),
                        TaskType::SearchUser => self.chat_state.handle_task_search_user(task_result),
                        TaskType::SendInviteContact => self.chat_state.handle_task_send_invite_contact(task_result),
                        TaskType::AcceptInviteContact => self.chat_state.handle_task_accept_invite_contact(task_result, ctx),
                        TaskType::FetchChatMessages => self.chat_state.handle_task_fetch_chat_messages(task_result, ctx)
                    }

                    self.result_queue.swap_remove(i);
                },
                Err(_e) => {}
            }
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
        match self.current_page {
            Page::Login => self.login_state.show_login_page(
                &self.http_thread,
                &mut self.result_queue,
                &mut self.current_page,
                ctx
            ),
            Page::CreateAccount => self.create_account_state.show_create_account_page(
                &self.http_thread,
                &mut self.result_queue,
                &mut self.current_page,
                ctx
            ),
            Page::Chat => self.chat_state.show_chat_page(
                &self.http_thread,
                &mut self.result_queue,
                ctx
            )
        }

        self.process_result_queue(ctx);
    }
}
