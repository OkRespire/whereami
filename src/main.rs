/*
use iced::widget::shader::wgpu::core::identity::Input;
 * All the commented out print statements are for debugging purporses
 * */

mod config_management;
mod hyprctl;
mod models;
mod search;
use std::os::unix::fs::FileExt;
use std::sync::LazyLock;
use std::{fs, process};

use fd_lock::{self, RwLock};
use iced::keyboard::{self, Key};
use iced::widget::scrollable::AbsoluteOffset;
use iced::widget::text_input::focus;
use iced::widget::{Scrollable, column, container, mouse_area, row, scrollable, text, text_input};
use iced::{Border, Color, Element, Length, Task, Theme};
use models::Client;

use crate::config_management::parse_color;

static TEXT_INPUT_ID: LazyLock<iced::widget::text_input::Id> =
    LazyLock::new(|| iced::widget::text_input::Id::new("search_bar".to_string()));

/// Messages for allowing the application to understand what updates it has to do
#[derive(Debug, Clone)]
pub enum Message {
    LoadClients,
    ClientsLoaded(Vec<Client>),
    Quit,
    ClientSelected,
    Navigate(Direction),
    CloseWindow,
    SelectAndFocus(usize),
    SelectAndClose(usize),
    HoverWindow(usize),
    UpdateInput(String),
    FocusSearch,
    DoNothing,
}

/// All the goodies for whereami. stores literally everything
/// if you want to add something else you need to store,
/// put it here!
pub struct AppState {
    clients: Vec<Client>,
    clients_to_display: Vec<(Client, String)>,
    selected_idx: usize,
    scroll_id: scrollable::Id,
    config: config_management::Config,
    query: String,
    is_query: bool,
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
            clients_to_display: Vec::new(),
            selected_idx: 0,
            scroll_id: scrollable::Id::new("item_scroll"),
            config: config,
            query: "".to_string(),
            is_query: false,
        }
    }
}

/// allows for multi threaded handling
fn subscription(state: &AppState) -> iced::Subscription<Message> {
    /// Any key handlers will be added here
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
            Key::Named(iced::keyboard::key::Named::Delete) => Some(Message::CloseWindow),
            Key::Character(",") => Some(Message::FocusSearch),
            _ => Some(Message::DoNothing),
        }
    }
    // how often the process list is refreshed
    iced::Subscription::batch(vec![
        iced::time::every(std::time::Duration::from_millis(
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
            search::filter_search(state);
            Task::none()
        }
        Message::Quit => process::exit(0),
        Message::ClientSelected => {
            let (selected_client, _) = &state.clients_to_display[state.selected_idx];
            let workspace_num = selected_client
                .workspace
                .as_ref()
                .map(|w| w.id)
                .unwrap_or(0);
            // println!("{:?}", state.clients[client_idx].title);
            Task::perform(
                async move { hyprctl::focus_window(workspace_num).await },
                |_| Message::Quit,
            )
        }
        Message::Navigate(dir) => {
            let item_height = state.config.layout.padding + state.config.font.size;
            if dir.eq(&Direction::Up) {
                if state.selected_idx == 0 {
                    state.selected_idx = state.clients_to_display.len() - 1;
                } else {
                    state.selected_idx -= 1;
                }
            } else {
                if state.selected_idx == state.clients_to_display.len() - 1 {
                    state.selected_idx = 0;
                } else {
                    state.selected_idx += 1;
                }
            }
            // // debug
            // println!(
            //     "{:?}\n{:?}",
            //     state.clients[state.selected_idx].title,
            //     state.clients[state.selected_idx].workspace
            // );

            scrollable::scroll_to::<Message>(
                state.scroll_id.clone(),
                AbsoluteOffset {
                    x: 0.0,
                    y: state.selected_idx as f32 * item_height,
                },
            )
        }
        Message::CloseWindow => {
            let (selected_client, _) = &state.clients_to_display[state.selected_idx];
            let address = &selected_client.address;
            // println!("{}\n{:?}", address, state.clients[state.selected_idx].title);
            Task::perform(hyprctl::close_window(address.to_string()), |_| {
                Message::LoadClients
            })
        }
        Message::SelectAndFocus(idx) => {
            state.selected_idx = idx;
            let (selected_client, _) = &state.clients_to_display[state.selected_idx];
            let workspace_num = selected_client
                .workspace
                .as_ref()
                .map(|w| w.id)
                .unwrap_or(0);
            Task::perform(
                async move { hyprctl::focus_window(workspace_num).await },
                |_| Message::Quit,
            )
        }
        Message::SelectAndClose(idx) => {
            state.selected_idx = idx;
            let (selected_client, _) = &state.clients_to_display[state.selected_idx];
            let address = &selected_client.address;
            // println!("{}\n{:?}", address, state.clients[state.selected_idx].title);
            Task::perform(hyprctl::close_window(address.to_string()), |_| {
                Message::LoadClients
            })
        }
        Message::HoverWindow(idx) => {
            state.selected_idx = idx;
            Task::none()
        }
        Message::UpdateInput(content) => {
            state.query = content;
            state.selected_idx = 0;
            // when input is empty it is false, so you can revert to not searching
            state.is_query = !state.query.is_empty();
            Task::none()
        }
        Message::FocusSearch => focus(TEXT_INPUT_ID.clone()),
        Message::DoNothing => Task::none(),
    }
}

fn view(state: &AppState) -> Element<'_, Message> {
    let items: Vec<_> = state
        .clients_to_display
        .iter()
        .enumerate()
        .filter_map(|(idx, (client, name))| {
            let is_selected = idx == state.selected_idx;
            let title = name;
            let workspace_id = client.workspace.as_ref().map(|w| w.id).unwrap_or(0);
            let status_col = match client.fullscreen {
                1 => parse_color(&state.config.colors.status.fullscreen),
                2 => parse_color(&state.config.colors.status.maximized),
                _ => {
                    if client.floating {
                        parse_color(&state.config.colors.status.floating)
                    } else {
                        parse_color(&state.config.colors.status.tiled)
                    }
                }
            };
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

            // These are split into parts so they can have different colours.
            // implementation for ALL of these colours will be added sometime later.
            // Currently only supports status colours
            let title_part: iced_core::widget::Text<'_, _, _> = text(title);
            let workspace_part = if workspace_id < 0 {
                // atleast for me, my special workspace (in a
                // scratch pad) is on workspace -98 -
                // assuming it uses the same logic, any
                // special workspace is a negative number
                text("@Workspace: Special Workspace")
            } else {
                text(format!("@Workspace: {}", workspace_id))
            };
            let status_part = text(format!("[{}]", status)).style(move |_| text::Style {
                color: Some(status_col),
            });

            // brings all together
            let item_content: iced::widget::Row<'_, _, _, _> =
                row!(title_part, workspace_part, status_part).spacing(state.config.layout.spacing);

            let styled = if is_selected {
                container(item_content)
                    .width(Length::Fill)
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
                    // Honestly looks better like this. Tried making the edges wrap around the text but i couldn't
                    // figure it out so this is the best way to make it look "pretty"
                    // So example:
                    // **When it is not selected:**
                    //    item blah blah blah
                    // **When it is selected:**
                    // -----------------------
                    // | item blah blah blah |
                    // -----------------------
                    // Not the greatest, but it looks nice.
                    .padding(state.config.layout.padding)
            } else {
                container(item_content)
                    .width(Length::Fill)
                    .style(|theme: &Theme| container::Style {
                        background: Some(Color::TRANSPARENT.into()),
                        text_color: Some(theme.palette().text.into()),
                        border: Border {
                            radius: state.config.layout.border_radius.into(),
                            ..Default::default()
                        },
                        ..Default::default()
                    })
                    .padding([state.config.layout.padding, state.config.layout.margin])
            }
            .width(Length::Shrink);

            let clickable = mouse_area(styled)
                .on_press(Message::SelectAndFocus(idx))
                .on_right_press(Message::SelectAndClose(idx))
                .on_enter(Message::HoverWindow(idx))
                .interaction(iced::mouse::Interaction::Pointer);

            Some(Element::from(clickable))
        })
        .collect();
    let search_bar_widget = Element::from(
        text_input("Search", &state.query)
            .id(TEXT_INPUT_ID.clone())
            .style(|_, _| text_input::Style {
                background: parse_color(&state.config.colors.background).into(),
                border: Border {
                    color: Color::WHITE,
                    radius: state.config.layout.border_radius.into(),
                    ..Default::default()
                },
                selection: parse_color(&state.config.colors.selected_text),
                icon: parse_color(&state.config.colors.selected_background),
                placeholder: parse_color(&state.config.colors.text),
                value: parse_color(&state.config.colors.selected_text),
            })
            .on_input(Message::UpdateInput)
            .on_submit(Message::ClientSelected)
            .padding(state.config.layout.padding)
            .size(state.config.font.size),
    );

    let scrollable_list: Element<'_, Message> = iced::widget::Scrollable::new(
        column(items).spacing(state.config.layout.spacing), // Put all client elements in a column
    )
    .width(Length::Fill)
    .height(Length::Fill) // The scrollable part should fill available vertical space
    .id(state.scroll_id.clone())
    .into();

    column![search_bar_widget, scrollable_list,]
        .spacing(state.config.layout.spacing)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

/// Little function i added so only **one** instance of whereami can be launched
/// creates a pid file that is locked until the application has stopped running
/// the file contains the PID so if you really wanna kill it, you have the PID
/// NOTE: this will **ONLY** be output in the terminal
fn acquire_lock() -> fd_lock::RwLockWriteGuard<'static, std::fs::File> {
    let pid_loc = "/tmp/whereami.pid";
    let file = fs::OpenOptions::new()
        .write(true)
        .create(true)
        .open(pid_loc)
        .expect("Failed to open lock file");

    // this is the main lock
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
            decorations: config.window.decorations, //these may not be needed
            transparent: config.window.transparent, // this too
            size: iced::Size::new(config.window.width, config.window.height),
            platform_specific: iced::window::settings::PlatformSpecific {
                application_id: "whereami".to_string(),
                override_redirect: false,
            },
            ..Default::default()
        })
        .subscription(subscription)
        .run()
}
