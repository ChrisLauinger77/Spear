use gtk4::prelude::RecentManagerExt;
use crate::plugins::{Action, ResultItem, Plugin};

pub struct RecentFilesPlugin;

impl RecentFilesPlugin {
    pub fn new() -> Self {
        Self
    }
}

impl Plugin for RecentFilesPlugin {
    fn id(&self) -> &str {
        "recent"
    }

    fn name(&self) -> &str {
        "Recent Files"
    }

    fn query(&self, query_text: &str, _settings: &crate::settings::AppSettings) -> Vec<ResultItem> {
        let query_trimmed = query_text.trim();
        if query_trimmed.is_empty() {
            return Vec::new();
        }

        let query_lower = query_trimmed.to_lowercase();
        let manager = gtk4::RecentManager::default();
        let items = manager.items();
        let mut results = Vec::new();

        for item in items {
            let display_name = item.display_name().to_string();
            let uri = item.uri().to_string();
            
            if display_name.to_lowercase().contains(&query_lower) {
                let filename_lower = display_name.to_lowercase();
                let score = if filename_lower == query_lower {
                    75
                } else if filename_lower.starts_with(&query_lower) {
                    65
                } else {
                    45
                };

                // Determine icon based on mime type
                let mime = item.mime_type().to_string();
                let is_dir = mime == "inode/directory";
                
                let icon = if is_dir {
                    "folder-symbolic".to_string()
                } else if mime.starts_with("image/") {
                    "image-x-generic-symbolic".to_string()
                } else if mime.starts_with("audio/") {
                    "audio-x-generic-symbolic".to_string()
                } else if mime.starts_with("video/") {
                    "video-x-generic-symbolic".to_string()
                } else if mime.contains("pdf") {
                    "x-office-document-symbolic".to_string()
                } else if mime.starts_with("text/") || mime.contains("json") || mime.contains("javascript") || mime.contains("python") {
                    "text-x-generic-symbolic".to_string()
                } else if mime.contains("compressed") || mime.contains("archive") || mime.contains("zip") || mime.contains("tar") {
                    "package-x-generic-symbolic".to_string()
                } else {
                    "text-x-generic-symbolic".to_string()
                };

                results.push(ResultItem {
                    id: format!("recent-{}", uri),
                    title: display_name,
                    subtitle: Some(uri.replace("file://", "").replace("%20", " ")),
                    icon: Some(icon),
                    category: "Recent Files".to_string(),
                    score,
                    actions: vec![
                        Action {
                            label: "Open File".to_string(),
                            action_type: "open-url".to_string(),
                            value: uri,
                        }
                    ],
                });
            }
        }

        // Sort by score desc, limit to 10 items
        results.sort_by(|a, b| b.score.cmp(&a.score).then_with(|| a.title.cmp(&b.title)));
        results.truncate(10);

        results
    }
}
