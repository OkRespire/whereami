use std::sync::{Mutex, MutexGuard};

use anyhow::{Context, Result};
use hyprland::{
    dispatch::DispatchType,
    shared::{HyprData, HyprDataVec},
};
use niri_ipc::{Action, Request, Response, socket::Socket};

/// All the information needed from both niri and hyprland's windows
#[derive(Debug, Clone, Default)]
pub struct Process {
    pub pid: i32,
    pub title: String,
    pub window_id: Option<u64>,
    pub workspace: u64,
    pub fullscreen: FullscreenStatus,
    pub floating: bool,
}

#[derive(Debug, Clone, Default)]
pub enum FullscreenStatus {
    Fullscreen,
    Maximised,

    #[default]
    None,
}

#[async_trait::async_trait]
pub(crate) trait Compositor {
    fn get_windows(&self) -> Result<Vec<Process>>;

    async fn focus_window(&self, process: Process) -> Result<()>;
    async fn close_window(&self, process: Process) -> Result<()>;
}

pub struct HyprlandCompositor;

pub struct NiriCompositor {
    pub socket: Mutex<Socket>,
}

#[async_trait::async_trait]
impl Compositor for HyprlandCompositor {
    fn get_windows(&self) -> Result<Vec<Process>> {
        let mut clients = hyprland::data::Clients::get()
            .context("Could not get clients")?
            .to_vec();

        clients.retain(|client| client.title != "whereami");
        clients.sort_by(|a, b| {
            let ws_a = a.workspace.id;
            let ws_b = b.workspace.id;
            ws_a.cmp(&ws_b)
        });

        let processes = clients
            .iter()
            .map(|cl| {
                let fs_mode = match cl.fullscreen {
                    hyprland::data::FullscreenMode::Fullscreen => FullscreenStatus::Fullscreen,
                    hyprland::data::FullscreenMode::Maximized => FullscreenStatus::Maximised,
                    _ => FullscreenStatus::None,
                };
                let workspace_id = u64::from(cl.workspace.id.unsigned_abs());
                Process {
                    pid: cl.pid,
                    title: cl.title.clone(),
                    window_id: None,
                    workspace: workspace_id,
                    fullscreen: fs_mode,
                    floating: cl.floating,
                }
            })
            .collect::<Vec<Process>>();

        Ok(processes)
    }

    async fn focus_window(&self, process: Process) -> Result<()> {
        hyprland::dispatch::Dispatch::call_async(DispatchType::Workspace(
            hyprland::dispatch::WorkspaceIdentifierWithSpecial::Id(
                i32::try_from(process.workspace).unwrap_or(1),
            ),
        ))
        .await
        .context(format!(
            "Could not switch to workspace {}",
            process.workspace
        ))?;
        Ok(())
    }

    async fn close_window(&self, process: Process) -> Result<()> {
        hyprland::dispatch::Dispatch::call_async(DispatchType::CloseWindow(
            hyprland::dispatch::WindowIdentifier::ProcessId(process.pid.cast_unsigned()),
        ))
        .await
        .context(format!("Could not close window {}", process.pid))?;
        Ok(())
    }
}

impl NiriCompositor {
    pub fn new() -> Result<Self> {
        Ok(Self {
            socket: Mutex::new(Socket::connect().context("failed to connect to niri socket")?),
        })
    }
    fn get_socket(&self) -> Result<MutexGuard<'_, Socket>> {
        let mut socket = self.socket.lock().unwrap();
        if socket.send(Request::Version).is_err() {
            *socket = Socket::connect().context("failed to reconnect to niri socket")?;
        }

        Ok(socket)
    }
}
#[async_trait::async_trait]
impl Compositor for NiriCompositor {
    fn get_windows(&self) -> Result<Vec<Process>> {
        let mut socket = self.get_socket()?;
        let reply = socket
            .send(Request::Windows)
            .context("Failed to send windows request")?;

        let mut res = match reply {
            Ok(Response::Windows(win)) => win,
            Ok(_) => anyhow::bail!("unexpected response"),
            Err(e) => anyhow::bail!("niri returned error {e}"),
        };
        res.retain(|client| client.title != Some("whereami".to_string()));
        let mut active_workspaces: Vec<u64> = res.iter().filter_map(|c| c.workspace_id).collect();
        active_workspaces.sort_unstable();
        active_workspaces.dedup();
        res.sort_by(|a, b| a.workspace_id.cmp(&b.workspace_id));

        let processes = res
            .iter()
            .filter_map(|c| {
                let fs_mode = match c.layout.window_offset_in_tile {
                    (0.0, 0.0) => FullscreenStatus::Fullscreen,
                    _ => FullscreenStatus::None,
                };
                // Niri's workspace IDs increment infinitely (e.g., closing and opening
                // workspaces might leave you with active IDs like 1, 3, and 6).
                // To prevent the UI from displaying jumping numbers, we map these raw IDs
                // to a sequential visual index.
                //
                // Example:
                // 1. Collect unique active workspaces -> [1, 3, 6]
                // 2. Find the raw ID's position in that list (1->0, 3->1, 6->2)
                // 3. Add 1 so the UI displays them neatly as Workspaces 1, 2, and 3.
                let ws_id = c
                    .workspace_id
                    .and_then(|id| active_workspaces.iter().position(|&x| x == id))
                    .map_or(0, |pos| (pos + 1) as u64); // +1 because programmers count from 0, humans from 1
                let pid = c.pid?;
                Some(Process {
                    pid,
                    title: c.title.as_deref().unwrap_or("Unknown").to_string(),
                    window_id: Some(c.id),
                    workspace: ws_id,
                    fullscreen: fs_mode,
                    floating: c.is_floating,
                })
            })
            .collect::<Vec<Process>>();

        Ok(processes)
    }

    async fn focus_window(&self, process: Process) -> Result<()> {
        let mut socket = self.get_socket()?;
        let reply = socket
            .send(Request::Action(Action::FocusWindow {
                id: process.window_id.unwrap(),
            }))
            .context("failed to send focus request")?;
        match reply {
            Ok(_) => Ok(()),
            Err(e) => anyhow::bail!("niri returned error: {e}"),
        }
    }

    async fn close_window(&self, process: Process) -> Result<()> {
        let mut socket = self.get_socket()?;
        let reply = socket
            .send(Request::Action(Action::CloseWindow {
                id: process.window_id,
            }))
            .context("Failed to close window")?;

        match reply {
            Ok(_) => Ok(()),
            Err(e) => anyhow::bail!("niri returned error: {e}"),
        }
    }
}
