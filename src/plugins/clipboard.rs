use crate::plugins::{Action, ResultItem, Plugin};
use std::sync::{Arc, Mutex};

pub struct ClipboardPlugin {
    history: Arc<Mutex<Vec<String>>>,
}

impl ClipboardPlugin {
    pub fn new() -> Self {
        // In a real implementation, this would subscribe to clipboard changes via GTK/GDK
        Self {
            history: Arc::new(Mutex::new(vec![
                "Example clipboard item 1".to_string(),
                "https://github.com/vicinaehq/vicinae".to_string(),
                "Spear Launcher is awesome!".to_string(),
            ])),
        }
    }
}

impl Plugin for ClipboardPlugin {
    fn id(&self) -> &str {
        "clipboard"
    }

    fn name(&self) -> &str {
        "Clipboard Manager"
    }

    fn mode_keyword(&self) -> Option<&str> {
        Some("clip")
    }

    fn is_mode_only(&self) -> bool {
        true
    }

    fn query(&self, query_text: &str, _settings: &crate::settings::AppSettings) -> Vec<ResultItem> {
        let history = self.history.lock().unwrap();
        let query_lower = query_text.to_lowercase();
        
        history.iter().enumerate()
            .filter(|(_, item)| item.to_lowercase().contains(&query_lower))
            .map(|(i, item)| {
                ResultItem {
                    id: format!("clip-{}", i),
                    title: if item.len() > 50 { format!("{}...", &item[..47]) } else { item.clone() },
                    subtitle: Some("Clipboard History".to_string()),
                    icon: Some("edit-paste-symbolic".to_string()),
                    category: "Clipboard".to_string(),
                    score: 70,
                    actions: vec![
                        Action {
                            label: "Copy to Clipboard".to_string(),
                            action_type: "copy-to-clipboard".to_string(),
                            value: item.clone(),
                        },
                        Action {
                            label: "Remove from History".to_string(),
                            action_type: "run-command".to_string(),
                            value: "echo 'Remove functionality not implemented'".to_string(),
                        }
                    ],
                }
            })
            .collect()
    }
}
