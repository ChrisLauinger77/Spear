pub mod apps;
pub mod calc;
pub mod web;
pub mod files;
pub mod gnome_sys;
pub mod run_cmd;
pub mod youtube;
pub mod recent;
pub mod file_manager;
pub mod packages;
pub mod fonts;
pub mod colors;
pub mod clipboard;
pub mod window_mgmt;
pub mod emojis;
pub mod window_switcher;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Action {
    pub label: String,
    #[serde(rename = "type")]
    pub action_type: String, // launch-app, open-url, copy-to-clipboard, run-command
    pub value: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ResultItem {
    pub id: String,
    pub title: String,
    pub subtitle: Option<String>,
    pub icon: Option<String>,
    pub category: String,
    pub score: i32,
    pub actions: Vec<Action>,
}

pub trait Plugin {
    fn id(&self) -> &str;
    #[allow(dead_code)]
    fn name(&self) -> &str;
    fn query(&self, query: &str, settings: &crate::settings::AppSettings) -> Vec<ResultItem>;
    fn reload_cache(&self) {}
    
    // New methods for Mode support
    fn mode_keyword(&self) -> Option<&str> { None }
    fn is_mode_only(&self) -> bool { false }
}
