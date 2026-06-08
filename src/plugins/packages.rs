use crate::plugins::{Action, ResultItem, Plugin};
use std::process::Command;

pub struct PackageSearchPlugin;

impl PackageSearchPlugin {
    pub fn new() -> Self {
        Self
    }

    fn search_apt(&self, query: &str) -> Vec<ResultItem> {
        let output = Command::new("apt-cache")
            .args(["search", "--names-only", query])
            .output();

        if let Ok(out) = output {
            let s = String::from_utf8_lossy(&out.stdout);
            s.lines()
                .take(10)
                .filter_map(|line| {
                    let parts: Vec<&str> = line.splitn(2, " - ").collect();
                    if parts.len() == 2 {
                        let pkg_name = parts[0].trim();
                        let description = parts[1].trim();
                        Some(ResultItem {
                            id: format!("apt-{}", pkg_name),
                            title: pkg_name.to_string(),
                            subtitle: Some(format!("[APT] {}", description)),
                            icon: Some("system-software-install-symbolic".to_string()),
                            category: "Packages".to_string(),
                            score: 80,
                            actions: vec![Action {
                                label: "Install Package".to_string(),
                                action_type: "run-command".to_string(),
                                value: format!("kgx -e 'sudo apt install {}'", pkg_name),
                            }],
                        })
                    } else {
                        None
                    }
                })
                .collect()
        } else {
            vec![]
        }
    }

    fn search_flatpak(&self, query: &str) -> Vec<ResultItem> {
        let output = Command::new("flatpak")
            .args(["search", "--columns=name,application,description", query])
            .output();

        if let Ok(out) = output {
            let s = String::from_utf8_lossy(&out.stdout);
            s.lines()
                .skip(1) // Skip header
                .take(10)
                .filter_map(|line| {
                    let parts: Vec<&str> = line.split('\t').collect();
                    if parts.len() >= 3 {
                        let name = parts[0].trim();
                        let app_id = parts[1].trim();
                        let description = parts[2].trim();
                        Some(ResultItem {
                            id: format!("flatpak-{}", app_id),
                            title: name.to_string(),
                            subtitle: Some(format!("[Flatpak] {}", description)),
                            icon: Some("flatpak-discover-symbolic".to_string()),
                            category: "Packages".to_string(),
                            score: 85,
                            actions: vec![Action {
                                label: "Install Flatpak".to_string(),
                                action_type: "run-command".to_string(),
                                value: format!("flatpak install -y {}", app_id),
                            }],
                        })
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

impl Plugin for PackageSearchPlugin {
    fn id(&self) -> &str {
        "packages"
    }

    fn name(&self) -> &str {
        "Package Search"
    }

    fn mode_keyword(&self) -> Option<&str> {
        Some("pkg")
    }

    fn is_mode_only(&self) -> bool {
        true
    }

    fn query(&self, query_text: &str, _settings: &crate::settings::AppSettings) -> Vec<ResultItem> {
        if query_text.len() < 3 {
            return vec![];
        }

        let mut results = Vec::new();
        results.extend(self.search_apt(query_text));
        results.extend(self.search_flatpak(query_text));
        
        results.sort_by(|a, b| b.score.cmp(&a.score));
        results
    }
}
