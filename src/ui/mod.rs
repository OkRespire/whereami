use crate::compositor::HyprlandCompositor;
use crate::compositor::NiriCompositor;
use std::sync::Arc;
use std::sync::LazyLock;

use iced::widget;

use crate::{
    compositor::{Compositor, Process},
    config_management::Config,
};

pub mod subscribe;
pub mod update;
pub mod view;

pub static TEXT_INPUT_ID: LazyLock<widget::Id> = LazyLock::new(|| widget::Id::new("search_bar"));

/// All the goodies for whereami. stores literally everything
/// if you want to add something else you need to store,
/// put it here!
pub struct AppState {
    pub clients: Vec<Process>,
    pub clients_to_display: Vec<(Process, String)>,
    pub selected_idx: usize,
    pub scroll_id: widget::Id,
    pub config: Config,
    pub query: String,
    pub is_query: bool,
    pub compositor: Arc<dyn Compositor + Send + Sync>,
}

impl Default for AppState {
    fn default() -> Self {
        let config = Config::new().expect("Failed to load config");
        let compositor = get_compositor();
        AppState {
            clients: Result::expect(compositor.get_windows(), "Failed"),
            clients_to_display: Vec::new(),
            selected_idx: 0,
            scroll_id: widget::Id::new("item_scroll"),
            config,
            query: String::new(),
            is_query: false,
            compositor,
        }
    }
}

/// Gets the current compositor used
/// Currently only supports Hyprland and Niri
fn get_compositor() -> Arc<dyn Compositor + Send + Sync> {
    if std::env::var("NIRI_SOCKET").is_ok() {
        return Arc::new(NiriCompositor) as Arc<dyn Compositor + Send + Sync>;
    } else if std::env::var("HYPRLAND_INSTANCE_SIGNATURE").is_ok() {
        return Arc::new(HyprlandCompositor) as Arc<dyn Compositor + Send + Sync>;
    }
    panic!("No supported compositor found");
}
