use std::process::Command;

use crate::models::Client;

pub fn get_clients() -> Vec<Client> {
    let cmd = Command::new("hyprctl")
        .args(["clients", "-j"])
        .output()
        .expect("Failed");
    let output = String::from_utf8_lossy(&cmd.stdout);
    let mut clients: Vec<Client> = serde_json::from_str(&output).expect("Invalid");

    clients.sort_by(|a, b| {
        let ws_a = a.workspace.as_ref().map(|w| w.id).unwrap_or(0);
        let ws_b = b.workspace.as_ref().map(|w| w.id).unwrap_or(0);
        ws_a.cmp(&ws_b)
    });

    clients
}

pub async fn focus_window(id: i32) {
    println!("Workspace:{}\n", id);
    let command_arg = format!("workspace {}", id);

    let _ = Command::new("hyprctl")
        .args(["dispatch", &command_arg])
        .spawn()
        .expect("Failed to switch");
}
