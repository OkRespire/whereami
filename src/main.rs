mod config_management;
mod hyprctl;
mod models;
use std::os::unix::fs::FileExt;
use std::{fs, path, process};

use fd_lock::{self, RwLock};
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
        .filter_map(|(idx, client)| {
            if client.title.as_deref().unwrap_or("No Title") != "whereami" {
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
                let item_content: iced_core::widget::text::Text<'_, _, Renderer> =
                    text(text_content);

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

                Some(Element::from(styled))
            } else {
                None
            }
        })
        .collect();

    Scrollable::new(column(items).width(Length::Fill))
        .id(state.scroll_id.clone())
        .into()
}

fn acquire_lock() -> fd_lock::RwLockWriteGuard<'static, std::fs::File> {
    let pid_loc = "/tmp/whereami.pid";
    let file = fs::OpenOptions::new()
        .write(true)
        .create(true)
        .open(pid_loc)
        .expect("Failed to open lock file");

    let lock = Box::leak(Box::new(RwLock::new(file)));

    match lock.try_write() {
        Ok(guard) => {
            let pid = process::id().to_string();
            guard
                .write_at(pid.as_bytes(), 0)
                .expect("Failed to write PID");
            guard
                .set_len(pid.len() as u64)
                .expect("Failed to truncate file");
            guard
        }
        Err(_) => {
            let old_pid = std::fs::read_to_string(pid_loc).expect("Failed to read file");
            eprintln!(
                "Another instance is already running using this PID {}, at location {}",
                old_pid, pid_loc
            );
            std::process::exit(1);
        }
    }
}

fn main() -> iced::Result {
    let _lock = acquire_lock();
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
