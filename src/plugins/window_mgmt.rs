use crate::plugins::{Action, ResultItem, Plugin};

pub struct WindowManagementPlugin;

impl WindowManagementPlugin {
    pub fn new() -> Self {
        Self
    }
}

impl Plugin for WindowManagementPlugin {
    fn id(&self) -> &str {
        "window-mgmt"
    }

    fn name(&self) -> &str {
        "Window Management"
    }

    fn mode_keyword(&self) -> Option<&str> {
        Some("win")
    }

    fn is_mode_only(&self) -> bool {
        true
    }

    fn query(&self, query_text: &str, _settings: &crate::settings::AppSettings) -> Vec<ResultItem> {
        let actions = vec![
            ("Tile Left", "Split window to the left half", "Super+Left"),
            ("Tile Right", "Split window to the right half", "Super+Right"),
            ("Maximize", "Make window fullscreen", "Super+Up"),
            ("Minimize", "Hide window", "Super+H"),
            ("Close Window", "Close the active window", "Alt+F4"),
            ("Center Window", "Move window to center", ""),
        ];

        let query_lower = query_text.to_lowercase();
        actions.into_iter()
            .filter(|(name, desc, _)| name.to_lowercase().contains(&query_lower) || desc.to_lowercase().contains(&query_lower))
            .map(|(name, desc, shortcut)| {
                ResultItem {
                    id: format!("win-{}", name.to_lowercase().replace(' ', "-")),
                    title: name.to_string(),
                    subtitle: Some(format!("{} {}", desc, if shortcut.is_empty() { "".to_string() } else { format!("({})", shortcut) })),
                    icon: Some("window-new-symbolic".to_string()),
                    category: "Window Management".to_string(),
                    score: 100,
                    actions: vec![Action {
                        label: "Apply Layout".to_string(),
                        action_type: "run-command".to_string(),
                        value: format!("echo 'Tiling action {} triggered'", name),
                    }],
                }
            })
            .collect()
    }
}
