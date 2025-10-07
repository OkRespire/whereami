use std::process::Command;
use tokio;

use crate::models::Client;

pub fn get_clients() -> Vec<Client> {
    let cmd = Command::new("hyprctl")
        .args(["clients", "-j"])
        .output()
        .expect("Failed");
    let output = String::from_utf8_lossy(&cmd.stdout);
    let clients: Vec<Client> = serde_json::from_str(&output).expect("Invalid");

    clients
}

pub async fn focus_window(id: i32) {
    let command_arg = format!("workspace {}", id);

    let _ = Command::new("hyprctl")
        .args(["dispatch", &command_arg])
        .spawn()
        .expect("Failed to switch");
}
