use iced::keyboard::{self, Key};

use super::update::{Direction, Message};

use super::AppState;

impl AppState {
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
                _ => Some(Message::None),
            }
        }
        // how often the process list is refreshed
        iced::Subscription::batch(vec![
            iced::time::every(std::time::Duration::from_millis(
                self.config.behavior.refresh_interval,
            ))
            .map(|_| Message::LoadClients),
            iced::keyboard::listen().map(|event| match event {
                iced::keyboard::Event::KeyPressed { key, modifiers, .. } => {
                    handle_keys(key, modifiers).unwrap()
                }
                _ => Message::None,
            }),
        ])
    }
}
