use std::process;
use std::sync::LazyLock;

use iced::keyboard::{self, Key};
use iced::widget::scrollable::AbsoluteOffset;
use iced::widget::text_input::focus;
use iced::widget::{column, container, mouse_area, row, scrollable, text, text_input};
use iced::{Border, Color, Element, Length, Task, Theme};

use crate::config_management::{Config, parse_colour};
use crate::hyprctl::{close_window, focus_window, get_clients};
use crate::models::Client;
use crate::search::filter_search;

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
    pub clients: Vec<Client>,
    pub clients_to_display: Vec<(Client, String)>,
    pub selected_idx: usize,
    pub scroll_id: scrollable::Id,
    pub config: Config,
    pub query: String,
    pub is_query: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Up,
    Down,
}

impl Default for AppState {
    fn default() -> Self {
        let config = Config::new().expect("Failed to load config");
        AppState {
            clients: get_clients(),
            clients_to_display: Vec::new(),
            selected_idx: 0,
            scroll_id: scrollable::Id::new("item_scroll"),
            config: config,
            query: "".to_string(),
            is_query: false,
        }
    }
}

impl AppState {
    /// allows for multi threaded handling
    pub fn subscription(&self) -> iced::Subscription<Message> {
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
                self.config.behavior.refresh_interval,
            ))
            .map(|_| Message::LoadClients),
            keyboard::on_key_press(handle_keys),
        ])
    }

    pub fn update(&mut self, msg: Message) -> Task<Message> {
        match msg {
            Message::LoadClients => Task::perform(async { get_clients() }, Message::ClientsLoaded),
            Message::ClientsLoaded(clients) => {
                self.clients = clients;
                filter_search(self);
                Task::none()
            }
            Message::Quit => process::exit(0),
            Message::ClientSelected => {
                let (selected_client, _) = &self.clients_to_display[self.selected_idx];
                let workspace_num = selected_client
                    .workspace
                    .as_ref()
                    .map(|w| w.id)
                    .unwrap_or(0);
                // println!("{:?}", self.clients[client_idx].title);
                Task::perform(async move { focus_window(workspace_num).await }, |_| {
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

                scrollable::scroll_to::<Message>(
                    self.scroll_id.clone(),
                    AbsoluteOffset {
                        x: 0.0,
                        y: self.selected_idx as f32 * item_height,
                    },
                )
            }
            Message::CloseWindow => {
                let (selected_client, _) = &self.clients_to_display[self.selected_idx];
                let address = &selected_client.address;
                // println!("{}\n{:?}", address, self.clients[self.selected_idx].title);
                Task::perform(close_window(address.to_string()), |_| Message::LoadClients)
            }
            Message::SelectAndFocus(idx) => {
                self.selected_idx = idx;
                let (selected_client, _) = &self.clients_to_display[self.selected_idx];
                let workspace_num = selected_client
                    .workspace
                    .as_ref()
                    .map(|w| w.id)
                    .unwrap_or(0);
                Task::perform(async move { focus_window(workspace_num).await }, |_| {
                    Message::Quit
                })
            }
            Message::SelectAndClose(idx) => {
                self.selected_idx = idx;
                let (selected_client, _) = &self.clients_to_display[self.selected_idx];
                let address = &selected_client.address;
                // println!("{}\n{:?}", address, self.clients[self.selected_idx].title);
                Task::perform(close_window(address.to_string()), |_| Message::LoadClients)
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
            Message::FocusSearch => focus(TEXT_INPUT_ID.clone()),
            Message::DoNothing => Task::none(),
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        let items: Vec<_> = self
            .clients_to_display
            .iter()
            .enumerate()
            .filter_map(|(idx, (client, name))| {
                let is_selected = idx == self.selected_idx;
                let title = name;
                let workspace_id = client.workspace.as_ref().map(|w| w.id).unwrap_or(0);
                let status_col = match client.fullscreen {
                    1 => parse_colour(&self.config.colours.status.fullscreen),
                    2 => parse_colour(&self.config.colours.status.maximized),
                    _ => {
                        if client.floating {
                            parse_colour(&self.config.colours.status.floating)
                        } else {
                            parse_colour(&self.config.colours.status.tiled)
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
                    row!(title_part, workspace_part, status_part)
                        .spacing(self.config.layout.spacing);

                let styled = if is_selected {
                    container(item_content)
                        .width(Length::Fill)
                        .style(|theme: &Theme| container::Style {
                            // Using Iced's built-in palette for primary/background
                            background: Some(theme.palette().primary.into()),
                            text_color: Some(theme.palette().background.into()),
                            border: Border {
                                radius: self.config.layout.border_radius.into(),
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
                        .padding(self.config.layout.padding)
                } else {
                    container(item_content)
                        .width(Length::Fill)
                        .style(|theme: &Theme| container::Style {
                            background: Some(Color::TRANSPARENT.into()),
                            text_color: Some(theme.palette().text.into()),
                            border: Border {
                                radius: self.config.layout.border_radius.into(),
                                ..Default::default()
                            },
                            ..Default::default()
                        })
                        .padding([self.config.layout.padding, self.config.layout.margin])
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
            text_input("Search", &self.query)
                .id(TEXT_INPUT_ID.clone())
                .style(|_, _| text_input::Style {
                    background: parse_colour(&self.config.colours.search_background).into(),
                    border: Border {
                        color: parse_colour(&self.config.colours.search_border_col),
                        radius: self.config.layout.border_radius.into(),
                        width: 1.0,
                    },
                    selection: parse_colour(&self.config.colours.selected_text),
                    icon: parse_colour(&self.config.colours.selected_background),
                    placeholder: parse_colour(&self.config.colours.text),
                    value: parse_colour(&self.config.colours.selected_text),
                })
                .on_input(Message::UpdateInput)
                .on_submit(Message::ClientSelected)
                .padding(self.config.layout.padding)
                .size(self.config.font.size),
        );

        let scrollable_list: Element<'_, Message> = iced::widget::Scrollable::new(
            column(items).spacing(self.config.layout.spacing), // Put all client elements in a column
        )
        .width(Length::Fill)
        .height(Length::Fill) // The scrollable part should fill available vertical space
        .id(self.scroll_id.clone())
        .into();

        column![search_bar_widget, scrollable_list,]
            .spacing(self.config.layout.spacing)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}
