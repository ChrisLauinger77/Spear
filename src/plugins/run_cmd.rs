use crate::plugins::{Action, ResultItem, Plugin};

pub struct RunCommandPlugin;

impl RunCommandPlugin {
    pub fn new() -> Self {
        Self
    }
}

impl Plugin for RunCommandPlugin {
    fn id(&self) -> &str {
        "command"
    }

    fn name(&self) -> &str {
        "Run Command"
    }

    fn query(&self, query_text: &str, settings: &crate::settings::AppSettings) -> Vec<ResultItem> {
        let query_trimmed = query_text.trim();
        let prefix = if settings.command_prefix.is_empty() {
            ">".to_string()
        } else {
            settings.command_prefix.clone()
        };

        if !query_trimmed.starts_with(&prefix) {
            return Vec::new();
        }

        let command_str = query_trimmed[prefix.len()..].trim();
        if command_str.is_empty() {
            return vec![ResultItem {
                id: "run-command-empty".to_string(),
                title: "Run Command".to_string(),
                subtitle: Some(format!("Type a command after '{}' to execute (e.g. {} gnome-terminal)", prefix, prefix)),
                icon: Some("terminal.svg".to_string()),
                category: "Run Command".to_string(),
                score: 120, // Sit at the top when user types prefix
                actions: Vec::new(),
            }];
        }

        vec![ResultItem {
            id: format!("run-command-{}", command_str),
            title: format!("Run: {}", command_str),
            subtitle: Some("Execute command in shell".to_string()),
            icon: Some("terminal.svg".to_string()),
            category: "Run Command".to_string(),
            score: 120, // Sit at the top when user types prefix
            actions: vec![Action {
                label: format!("Run '{}'", command_str),
                action_type: "run-command".to_string(),
                value: command_str.to_string(),
            }],
        }]
    }
}
