/// All the information needed from hyprctl clients -j
#[derive(serde::Deserialize, Debug, Clone)]
pub struct Client {
    pub address: String,
    pub title: Option<String>,
    pub class: String,
    pub workspace: Option<Workspace>,
    pub fullscreen: i32,
    pub floating: bool,
}

#[derive(Debug, serde::Deserialize, Clone)]
pub struct Workspace {
    pub id: i32,
    pub name: Option<String>,
}
