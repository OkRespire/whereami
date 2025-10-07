#[derive(serde::Deserialize, Debug, Clone)]
pub struct Client {
    pub address: String,
    pub title: Option<String>,
    pub class: String,
    pub workspace: Option<Workspace>,
}

#[derive(Debug, serde::Deserialize, Clone)]
pub struct Workspace {
    pub id: i32,
    pub name: Option<String>,
}
