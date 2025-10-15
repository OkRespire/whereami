mod hyprctl;
mod models;
mod utils;
use std::process;

use iced::keyboard::{self, Key};
use iced::widget::{column, container, text};
use iced::{Border, Color, Element, Length, Renderer, Task, Theme};
use models::Client;

#[derive(Debug, Clone)]
pub enum Message {
    LoadClients,
    ClientsLoaded(Vec<Client>),
    Quit,
    ClientSelected,
    Navigate(Direction),
    DoNothing,
}

pub struct AppState {
    clients: Vec<Client>,
    selected_idx: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Up,
    Down,
}

impl Default for AppState {
    fn default() -> Self {
        AppState {
            clients: hyprctl::get_clients(),
            selected_idx: 0,
        }
    }
}

fn subscription(_state: &AppState) -> iced::Subscription<Message> {
    fn handle_keys(key: Key, _modifiers: keyboard::Modifiers) -> Option<Message> {
        match key.as_ref() {
            Key::Named(iced::keyboard::key::Named::ArrowUp) => {
                Some(Message::Navigate(Direction::Up))
            }
            Key::Named(iced::keyboard::key::Named::ArrowDown) => {
                Some(Message::Navigate(Direction::Down))
            }
            Key::Named(iced::keyboard::key::Named::Enter) => Some(Message::ClientSelected),
            _ => Some(Message::DoNothing),
        }
    }
    iced::Subscription::batch(vec![
        iced::time::every(std::time::Duration::from_secs(1)).map(|_| Message::LoadClients),
        keyboard::on_key_press(handle_keys),
    ])
}

fn update(state: &mut AppState, msg: Message) -> Task<Message> {
    match msg {
        Message::LoadClients => {
            Task::perform(async { hyprctl::get_clients() }, Message::ClientsLoaded)
        }
        Message::ClientsLoaded(clients) => {
            state.clients = clients;
            Task::none()
        }
        Message::Quit => process::exit(0),
        Message::ClientSelected => {
            let client_idx = state.selected_idx;
            let workspace_num = state.clients[client_idx]
                .workspace
                .as_ref()
                .map(|w| w.id)
                .unwrap_or(0);
            Task::perform(
                async move { hyprctl::focus_window(workspace_num).await },
                |_| Message::Quit,
            )
        }
        Message::Navigate(dir) => {
            if dir.eq(&Direction::Up) {
                if state.selected_idx == 0 {
                    state.selected_idx = state.clients.len() - 1;
                } else {
                    state.selected_idx -= 1;
                }
                Task::none()
            } else {
                if state.selected_idx == state.clients.len() - 1 {
                    state.selected_idx = 0;
                } else {
                    state.selected_idx += 1;
                }
                Task::none()
            }
        }
        Message::DoNothing => Task::none(),
    }
}

fn view(state: &AppState) -> Element<'_, Message> {
    let mut sorted_clients = state.clients.clone();
    sorted_clients.sort_by(|a, b| {
        let ws_a = a.workspace.as_ref().map(|w| w.id).unwrap_or(0);
        let ws_b = b.workspace.as_ref().map(|w| w.id).unwrap_or(0);
        ws_a.cmp(&ws_b)
    });

    let items: Vec<_> = sorted_clients
        .iter()
        .enumerate()
        .map(|(idx, client)| -> _ {
            let is_selected = idx == state.selected_idx;
            let title = client.title.as_deref().unwrap_or("No title");
            let workspace_id = client.workspace.as_ref().map(|w| w.id).unwrap_or(0);
            let status = match client.fullscreen {
                1 => "Fullscreen",
                2 => "Maximised",
                _ => {
                    if client.floating {
                        "Float"
                    } else {
                        "Tiled"
                    }
                }
            };

            let text_content = if workspace_id < 0 {
                format!("{}@Workspace: Special Workspace [{}]", title, status)
            } else {
                format!("{}@Workspace: {} [{}]", title, workspace_id, status)
            };
            let item_content: iced_core::widget::text::Text<'_, _, Renderer> = text(text_content);

            let styled = if is_selected {
                container(item_content)
                    .style(|theme: &Theme| container::Style {
                        // Using Iced's built-in palette for primary/background
                        background: Some(theme.palette().primary.into()),
                        text_color: Some(theme.palette().background.into()),
                        border: Border {
                            radius: 4.0.into(),
                            ..Default::default()
                        },
                        ..Default::default()
                    })
                    .padding(10)
            } else {
                container(item_content)
                    .style(|theme: &Theme| container::Style {
                        background: Some(Color::TRANSPARENT.into()),
                        text_color: Some(theme.palette().text.into()),
                        border: Border {
                            radius: 4.0.into(),
                            ..Default::default()
                        },
                        ..Default::default()
                    })
                    .padding(10)
            };

            Element::from(styled)
        })
        .collect();

    column(items).width(Length::Fill).into()
}

fn main() -> iced::Result {
    iced::application("whereami", update, view)
        .theme(|_| iced::Theme::GruvboxDark)
        .window(iced::window::Settings {
            position: iced::window::Position::Centered,
            decorations: false,
            transparent: true,
            size: iced::Size::new(400.0, 300.0),
            ..Default::default()
        })
        .subscription(subscription)
        .run()
}
