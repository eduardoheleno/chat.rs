use crate::egui;
use crate::thread::http_thread::TaskWrapper;
use crate::task::create_account_task::CreateAccountTask;
use crate::task::TaskResult;
use crate::state::Page;
use serde::{Deserialize, Serialize};
use egui::{
    RichText,
    TextEdit,
    Color32
};
use std::sync::mpsc::{self, Sender, Receiver};

#[derive(Serialize, Deserialize)]
struct HttpResponse {
    message: String
}

pub struct CreateAccountState {
    pub email: String,
    pub password: String,
    pub confirm_password: String,
    pub error: String,
    pub is_loading: bool
}

impl Default for CreateAccountState {
    fn default() -> Self {
        Self {
            email: String::new(),
            password: String::new(),
            confirm_password: String::new(),
            error: String::new(),
            is_loading: false
        }
    }
}

impl CreateAccountState {
    pub fn show_create_account_page(
        &mut self,
        http_thread: &Sender<TaskWrapper>,
        result_queue: &mut Vec<Receiver<TaskResult>>,
        current_page: &mut Page,
        ctx: &egui::Context
    ) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(50.);
                ui.label(RichText::new("Create account").size(20.));
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
                    ui.add(
                        TextEdit::singleline(&mut self.confirm_password)
                            .hint_text("Confirm password")
                            .password(true)
                    );

                    ui.add_space(15.);

                    if self.is_loading {
                        ui.spinner();
                    } else {
                        ui.columns(2, |columns| {
                            let create_account_button = egui::Button::new("Back to login");
                            if columns[0].add(create_account_button).clicked() {
                                *current_page = Page::Login;
                            }

                            let login_button = egui::Button::new("Create");
                            if columns[1].add(login_button).clicked() {
                                self.error.clear();
                                if self.password != self.confirm_password {
                                    self.error = "Your passwords are not equal".to_owned();
                                    return;
                                }

                                self.is_loading = true;

                                let create_account_task = CreateAccountTask::new(
                                    self.email.to_owned(),
                                    self.password.to_owned()
                                );

                                let (task_channel_sender, task_channel_receiver) = mpsc::channel();
                                let task_wrapper = TaskWrapper::new(
                                    Box::new(create_account_task),
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

    pub fn handle_task_result(&mut self, task_result: TaskResult) {
        self.is_loading = false;

        if task_result.status_code >= 400 {
            let parsed_response: HttpResponse = serde_json::from_str(task_result.response.as_ref()).unwrap();
            self.error = parsed_response.message;
        }
    }
}
