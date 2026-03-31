/*
 * All the commented out print statements are for debugging purporses
 * */

mod compositor;
mod config_management;
mod search;
mod ui;
use std::os::unix::fs::FileExt;
use std::{fs, process};

use crate::ui::AppState;
use fd_lock::RwLock;
use iced_layershell::{application, reexport};
use iced_layershell::reexport::{Anchor, KeyboardInteractivity};
use iced_layershell::settings::{LayerShellSettings, StartMode};

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

fn namespace() -> String {
    String::from("whereami")
}

fn main() -> iced_layershell::Result {
    let _lock = acquire_lock();
    let config = config_management::Config::new().expect("Failed to load config");

    let theme = config.get_theme();
    application(
        AppState::default,
        namespace,
        AppState::update,
        AppState::view,
    )
    .theme(move |_state: &AppState| theme.clone())
    .layer_settings(LayerShellSettings {
        anchor: Anchor::empty(),
        layer: reexport::Layer::Top,
        exclusive_zone: 0,
        start_mode: StartMode::Active,
        size: Some((config.window.width as u32, config.window.height as u32)),
        keyboard_interactivity: KeyboardInteractivity::Exclusive,
        ..Default::default()
    })
    .subscription(AppState::subscription)
    .run()
}
