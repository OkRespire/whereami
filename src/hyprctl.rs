/*
 * All the commented out print statements are for debugging purporses
 * */

use std::process::Command;

// use crate::models::Client;

use hyprland::{
    self,
    data::Client,
    dispatch::DispatchType,
    shared::{Address, HyprData, HyprDataVec, HyprError},
};

/// Contacts hyprctl to get all clients and is processed by serde
/// and then it is sorted by workspace order
pub fn get_clients() -> Result<Vec<Client>, HyprError> {
    let mut clients = hyprland::data::Clients::get()?.to_vec();
    // let cmd = Command::new("hyprctl")
    //     .args(["clients", "-j"])
    //     .output()
    //     .expect("Failed");
    // let output = String::from_utf8_lossy(&cmd.stdout);
    // let mut clients: Vec<Client> = serde_json::from_str(&output).expect("Invalid");

    clients.retain(|client| client.title != "whereami");
    clients.sort_by(|a, b| {
        let ws_a = a.workspace.id;
        let ws_b = b.workspace.id;
        ws_a.cmp(&ws_b)
    });

    Ok(clients)
}

/// gets the workspace id and asks hyprctl (maybe politely) to switch to it
pub async fn focus_window(id: i32) {
    // // debug
    // println!("Workspace:{}\n", id);
    hyprland::dispatch::Dispatch::call_async(DispatchType::Workspace(
        hyprland::dispatch::WorkspaceIdentifierWithSpecial::Id(id),
    ))
    .await;
}

/// makes hyprctl close the window when the corresponding button is pressed.
pub async fn close_window(address: String) {
    hyprland::dispatch::Dispatch::call_async(DispatchType::CloseWindow(
        hyprland::dispatch::WindowIdentifier::Address(Address::new(&address)),
    ));

    // let command_arg = format!("closewindow address:{}", address);
    //
    // let _ = Command::new("hyprctl")
    //     .args(["dispatch", &command_arg])
    //     .spawn()
    //     .expect("Failed to close window");
}
