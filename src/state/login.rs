use crate::egui;
use crate::thread::http_thread::TaskWrapper;
use crate::task::login_task::LoginTask;
use crate::task::TaskResult;
use egui::{
    RichText,
    TextEdit
};
use std::sync::mpsc::{self, Sender, Receiver};

pub struct LoginState {
    pub email: String,
    pub password: String,
    pub is_loading: bool
}

impl Default for LoginState {
    fn default() -> Self {
        Self {
            email: String::new(),
            password: String::new(),
            is_loading: false
        }
    }
}

impl LoginState {
    pub fn show_login_page(
        &mut self,
        http_thread: &Sender<TaskWrapper>,
        result_queue: &mut Vec<Receiver<TaskResult>>,
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

                    ui.columns(2, |columns| {
                        let login_button = egui::Button::new("Login");
                        if columns[1].add_enabled(!self.is_loading, login_button).clicked() {
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
                });
            });
        });
    }
}
