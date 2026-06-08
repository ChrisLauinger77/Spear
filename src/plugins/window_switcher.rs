use crate::plugins::{Action, ResultItem, Plugin};
use std::process::Command;

pub struct WindowSwitcherPlugin;

impl WindowSwitcherPlugin {
    pub fn new() -> Self {
        Self
    }

    fn get_open_windows(&self) -> Vec<(String, String)> {
        // Using wmctrl as a reliable way to list windows in GNOME (X11/XWayland)
        // For pure Wayland, we'd use the Introspection API or a Shell Extension
        let output = Command::new("wmctrl")
            .arg("-l")
            .output();

        if let Ok(out) = output {
            let s = String::from_utf8_lossy(&out.stdout);
            s.lines()
                .filter_map(|line| {
                    let parts: Vec<&str> = line.splitn(4, ' ').collect();
                    if parts.len() >= 4 {
                        let id = parts[0].to_string();
                        let title = parts[3..].join(" ").trim().to_string();
                        if !title.is_empty() && title != "Spear Launcher" {
                            Some((id, title))
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                })
                .collect()
        } else {
            vec![]
        }
    }
}

impl Plugin for WindowSwitcherPlugin {
    fn id(&self) -> &str {
        "window-switcher"
    }

    fn name(&self) -> &str {
        "Window Switcher"
    }

    fn mode_keyword(&self) -> Option<&str> {
        Some("sw")
    }

    fn is_mode_only(&self) -> bool {
        true
    }

    fn query(&self, query_text: &str, _settings: &crate::settings::AppSettings) -> Vec<ResultItem> {
        let query_lower = query_text.to_lowercase();
        let windows = self.get_open_windows();

        windows.into_iter()
            .filter(|(_, title)| title.to_lowercase().contains(&query_lower))
            .map(|(id, title)| {
                ResultItem {
                    id: format!("window-{}", id),
                    title,
                    subtitle: Some("Open Window".to_string()),
                    icon: Some("window-new-symbolic".to_string()),
                    category: "Windows".to_string(),
                    score: 95,
                    actions: vec![
                        Action {
                            label: "Switch to Window".to_string(),
                            action_type: "run-command".to_string(),
                            value: format!("wmctrl -i -a {}", id),
                        },
                        Action {
                            label: "Close Window".to_string(),
                            action_type: "run-command".to_string(),
                            value: format!("wmctrl -i -c {}", id),
                        }
                    ],
                }
            })
            .collect()
    }
}
