mod hyprctl;
mod models;
mod utils;
use std::process;

use iced::widget::{button, column, text};
use iced::{Element, Task};
use models::Client;

#[derive(Debug, Clone)]
pub enum Message {
    LoadClients,
    ClientsLoaded(Vec<Client>),
    Quit,
    ClientSelected(i32),
}

pub struct AppState {
    clients: Vec<Client>,
}
impl Default for AppState {
    fn default() -> Self {
        AppState {
            clients: hyprctl::get_clients(),
        }
    }
}

fn subscription(_state: &AppState) -> iced::Subscription<Message> {
    iced::Subscription::batch(vec![
        iced::time::every(std::time::Duration::from_secs(1)).map(|_| Message::LoadClients),
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
        Message::ClientSelected(workspace_num) => Task::perform(
            async move { hyprctl::focus_window(workspace_num).await },
            |_| Message::Quit,
        ),
    }
}
fn view(state: &AppState) -> Element<Message> {
    let mut content = column![text("Hello from App").size(24),];

    let mut sorted_clients = state.clients.clone();
    sorted_clients.sort_by(|a, b| {
        let ws_a = a.workspace.as_ref().map(|w| w.id).unwrap_or(0);
        let ws_b = b.workspace.as_ref().map(|w| w.id).unwrap_or(0);
        ws_a.cmp(&ws_b)
    });

    for client in &sorted_clients {
        let title = client.title.as_deref().unwrap_or("No title");
        let workspace_id = client.workspace.as_ref().map(|w| w.id).unwrap_or(0);

        let client_button = button(text(format!("{}@Workspace: {}", title, workspace_id)))
            .on_press(Message::ClientSelected(workspace_id));
        content = content.push(client_button);
    }
    content.padding(10).spacing(4).into()
}
fn main() -> iced::Result {
    iced::application("HyprSwitch", update, view)
        .theme(|_| iced::Theme::GruvboxDark)
        .window(iced::window::Settings {
            position: iced::window::Position::Centered,
            decorations: false,
            transparent: true,
            level: iced::window::Level::AlwaysOnTop,

            ..Default::default()
        })
        .subscription(subscription)
        .run()
}
