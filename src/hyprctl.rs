/*
 * All the commented out print statements are for debugging purporses
 * */

use std::process::Command;

use crate::models::Client;

/// Contacts hyprctl to get all clients and is processed by serde
/// and then it is sorted by workspace order
pub fn get_clients() -> Vec<Client> {
    let cmd = Command::new("hyprctl")
        .args(["clients", "-j"])
        .output()
        .expect("Failed");
    let output = String::from_utf8_lossy(&cmd.stdout);
    let mut clients: Vec<Client> = serde_json::from_str(&output).expect("Invalid");
    clients.retain(|client| client.title.as_deref() != Some("whereami"));

    clients.sort_by(|a, b| {
        let ws_a = a.workspace.as_ref().map(|w| w.id).unwrap_or(0);
        let ws_b = b.workspace.as_ref().map(|w| w.id).unwrap_or(0);
        ws_a.cmp(&ws_b)
    });

    clients
}

/// gets the workspace id and asks hyprctl (maybe politely) to switch to it
pub async fn focus_window(id: i32) {
    // // debug
    // println!("Workspace:{}\n", id);
    let command_arg = format!("workspace {}", id);

    let _ = Command::new("hyprctl")
        .args(["dispatch", &command_arg])
        .spawn()
        .expect("Failed to switch");
}

/// makes hyprctl close the window when the corresponding button is pressed.
pub async fn close_window(address: String) {
    let command_arg = format!("closewindow address:{}", address);

    let _ = Command::new("hyprctl")
        .args(["dispatch", &command_arg])
        .spawn()
        .expect("Failed to close window");
}
