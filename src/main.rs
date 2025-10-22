mod config_management;
mod hyprctl;
mod models;
use std::{path, process};

use config_management::Config;
use iced::keyboard::{self, Key};
use iced::widget::scrollable::AbsoluteOffset;
use iced::widget::{Scrollable, column, container, scrollable, text};
use iced::{Border, Color, Element, Length, Renderer, Task, Theme};
use models::Client;

use crate::config_management::parse_color;

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
    scroll_id: scrollable::Id,
    config: config_management::Config,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Up,
    Down,
}

impl Default for AppState {
    fn default() -> Self {
        let config = config_management::Config::new().expect("Failed to load config");
        AppState {
            clients: hyprctl::get_clients(),
            selected_idx: 0,

            scroll_id: scrollable::Id::new("item_scroll"),
            config: config,
        }
    }
}

fn subscription(state: &AppState) -> iced::Subscription<Message> {
    fn handle_keys(key: Key, _modifiers: keyboard::Modifiers) -> Option<Message> {
        match key.as_ref() {
            Key::Named(iced::keyboard::key::Named::ArrowUp) => {
                Some(Message::Navigate(Direction::Up))
            }
            Key::Named(iced::keyboard::key::Named::ArrowDown) => {
                Some(Message::Navigate(Direction::Down))
            }
            Key::Named(iced::keyboard::key::Named::Enter) => Some(Message::ClientSelected),
            Key::Named(iced::keyboard::key::Named::Escape) => Some(Message::Quit),
            _ => Some(Message::DoNothing),
        }
    }
    iced::Subscription::batch(vec![
        iced::time::every(std::time::Duration::from_secs(
            state.config.behavior.refresh_interval,
        ))
        .map(|_| Message::LoadClients),
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
            println!("{:?}", state.clients[client_idx].title);
            Task::perform(
                async move { hyprctl::focus_window(workspace_num).await },
                |_| Message::Quit,
            )
        }
        Message::Navigate(dir) => {
            let item_height = state.config.layout.padding + state.config.font.size;
            if dir.eq(&Direction::Up) {
                if state.selected_idx == 0 {
                    state.selected_idx = state.clients.len() - 1;
                } else {
                    state.selected_idx -= 1;
                }
            } else {
                if state.selected_idx == state.clients.len() - 1 {
                    state.selected_idx = 0;
                } else {
                    state.selected_idx += 1;
                }
            }

            scrollable::scroll_to::<Message>(
                state.scroll_id.clone(),
                AbsoluteOffset {
                    x: 0.0,
                    y: state.selected_idx as f32 * item_height,
                },
            )
        }
        Message::DoNothing => Task::none(),
    }
}

fn view(state: &AppState) -> Element<'_, Message> {
    let items: Vec<_> = state
        .clients
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
                            radius: state.config.layout.border_radius.into(),
                            ..Default::default()
                        },
                        ..Default::default()
                    })
                    .padding(state.config.layout.padding)
            } else {
                container(item_content)
                    .style(|theme: &Theme| container::Style {
                        background: Some(Color::TRANSPARENT.into()),
                        text_color: Some(theme.palette().text.into()),
                        border: Border {
                            radius: state.config.layout.border_radius.into(),
                            ..Default::default()
                        },
                        ..Default::default()
                    })
                    .padding(state.config.layout.padding)
            };

            Element::from(styled)
        })
        .collect();

    Scrollable::new(column(items).width(Length::Fill))
        .id(state.scroll_id.clone())
        .into()
}

fn main() -> iced::Result {
    let config = config_management::Config::new().expect("Failed to load config");

    let theme = config.get_theme();
    iced::application("whereami", update, view)
        .theme(move |_| theme.clone())
        .window(iced::window::Settings {
            position: iced::window::Position::Centered,
            decorations: config.window.decorations,
            transparent: config.window.transparent,
            size: iced::Size::new(config.window.width, config.window.height),
            ..Default::default()
        })
        .subscription(subscription)
        .run()
}
