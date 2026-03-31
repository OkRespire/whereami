use std::{process, sync::Arc};

use iced::{
    Task,
    widget::operation::{self, AbsoluteOffset},
};
use iced_layershell::to_layer_message;

use crate::{compositor::Process, search::filter_search};

use super::{AppState, TEXT_INPUT_ID};

/// Messages for allowing the application to understand what updates it has to do
#[to_layer_message]
#[derive(Debug, Clone)]
pub enum Message {
    LoadClients,
    ClientsLoaded(Vec<Process>),
    Quit,
    ClientSelected,
    Navigate(Direction),
    CloseWindow,
    SelectAndFocus(usize),
    SelectAndClose(usize),
    HoverWindow(usize),
    UpdateInput(String),
    FocusSearch,
    None,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Up,
    Down,
}

impl AppState {
    pub fn update(&mut self, msg: Message) -> Task<Message> {
        match msg {
            Message::LoadClients => {
                let compositor = Arc::clone(&self.compositor);
                Task::perform(
                    async move { Result::expect(compositor.get_windows(), "err") },
                    Message::ClientsLoaded,
                )
            }
            Message::ClientsLoaded(clients) => {
                self.clients = clients;
                filter_search(self);
                Task::none()
            }
            Message::Quit => process::exit(0),
            Message::ClientSelected => {
                let (selected_client, _) = &self.clients_to_display[self.selected_idx];
                let compositor = Arc::clone(&self.compositor);
                let cl = selected_client.clone();
                // println!("{:?}", self.clients[client_idx].title);
                Task::perform(async move { compositor.focus_window(cl).await }, |_| {
                    Message::Quit
                })
            }
            Message::Navigate(dir) => {
                let item_height = self.config.layout.padding + self.config.font.size;
                if dir.eq(&Direction::Up) {
                    if self.selected_idx == 0 {
                        self.selected_idx = self.clients_to_display.len() - 1;
                    } else {
                        self.selected_idx -= 1;
                    }
                } else {
                    if self.selected_idx == self.clients_to_display.len() - 1 {
                        self.selected_idx = 0;
                    } else {
                        self.selected_idx += 1;
                    }
                }
                // // debug
                // println!(
                //     "{:?}\n{:?}",
                //     self.clients[self.selected_idx].title,
                //     self.clients[self.selected_idx].workspace
                // );

                operation::scroll_to::<Message>(
                    self.scroll_id.clone(),
                    AbsoluteOffset {
                        x: 0.0,
                        y: self.selected_idx as f32 * item_height,
                    },
                )
            }
            Message::CloseWindow => {
                let (selected_client, _) = &self.clients_to_display[self.selected_idx];
                let cl = selected_client.clone();
                let compositor = Arc::clone(&self.compositor);
                // println!("{}\n{:?}", address, self.clients[self.selected_idx].title);
                Task::perform(async move { compositor.close_window(cl).await }, |_| {
                    Message::LoadClients
                })
            }
            Message::SelectAndFocus(idx) => {
                self.selected_idx = idx;
                let compositor = Arc::clone(&self.compositor);
                let (selected_client, _) = &self.clients_to_display[self.selected_idx];
                let cl = selected_client.clone();
                Task::perform(async move { compositor.focus_window(cl).await }, |_| {
                    Message::Quit
                })
            }
            Message::SelectAndClose(idx) => {
                self.selected_idx = idx;
                let compositor = Arc::clone(&self.compositor);
                let (selected_client, _) = &self.clients_to_display[self.selected_idx];
                let cl = selected_client.clone();
                // println!("{}\n{:?}", address, self.clients[self.selected_idx].title);
                Task::perform(async move { compositor.close_window(cl).await }, |_| {
                    Message::LoadClients
                })
            }
            Message::HoverWindow(idx) => {
                self.selected_idx = idx;
                Task::none()
            }
            Message::UpdateInput(content) => {
                self.query = content;
                self.selected_idx = 0;
                // when input is empty it is false, so you can revert to not searching
                self.is_query = !self.query.is_empty();
                Task::none()
            }
            Message::FocusSearch => operation::focus(TEXT_INPUT_ID.clone()),
            Message::None => Task::none(),
            _ => unreachable!(),
        }
    }
}
