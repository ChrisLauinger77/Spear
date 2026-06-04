use std::fs;
use std::path::PathBuf;
use crate::plugins::{Action, ResultItem, Plugin};

pub struct FileManagerPlugin;

impl FileManagerPlugin {
    pub fn new() -> Self {
        Self
    }
}

impl Plugin for FileManagerPlugin {
    fn id(&self) -> &str {
        "file_manager"
    }

    fn name(&self) -> &str {
        "File Manager"
    }

    fn query(&self, query_text: &str, _settings: &crate::settings::AppSettings) -> Vec<ResultItem> {
        let query_trimmed = query_text.trim();
        if !query_trimmed.starts_with("ff") {
            return Vec::new();
        }

        let path_part = if query_trimmed == "ff" {
            ""
        } else if query_trimmed.starts_with("ff ") {
            query_trimmed["ff ".len()..].trim()
        } else {
            return Vec::new();
        };

        let home_str = std::env::var("HOME").unwrap_or_else(|_| "/home/atharva".to_string());
        let mut resolved_path = if path_part.is_empty() {
            home_str.clone()
        } else if path_part.starts_with('~') {
            path_part.replacen('~', &home_str, 1)
        } else {
            path_part.to_string()
        };

        // If path doesn't start with / and is not empty, check if we should resolve relative to home
        if !resolved_path.starts_with('/') && !path_part.is_empty() {
            resolved_path = format!("{}/{}", home_str, resolved_path);
        }

        let mut dir_to_list = PathBuf::from(&resolved_path);
        let mut filter = String::new();

        if !dir_to_list.exists() || !dir_to_list.is_dir() {
            // Check if parent directory exists to allow tab-like filtering
            if let Some(parent) = dir_to_list.parent() {
                if parent.exists() && parent.is_dir() {
                    filter = dir_to_list.file_name().unwrap_or_default().to_string_lossy().to_string().to_lowercase();
                    dir_to_list = parent.to_path_buf();
                } else {
                    return Vec::new();
                }
            } else {
                return Vec::new();
            }
        }

        let mut results = Vec::new();

        // 1. Add "Go Up (..)" entry if we have a parent
        if let Some(parent) = dir_to_list.parent() {
            let parent_str = parent.to_string_lossy().to_string();
            results.push(ResultItem {
                id: format!("fm-dir-up-{}", parent_str),
                title: "..".to_string(),
                subtitle: Some(format!("Go up to: {}", parent_str)),
                icon: Some("go-up-symbolic".to_string()),
                category: "File Manager".to_string(),
                score: 200, // Parent link always sits at top
                actions: vec![Action {
                    label: "Navigate Up".to_string(),
                    action_type: "navigate-dir".to_string(),
                    value: format!("ff {}/", parent_str),
                }],
            });
        }

        // 2. Read directory entries
        if let Ok(entries) = fs::read_dir(&dir_to_list) {
            for entry in entries.flatten() {
                let path = entry.path();
                let filename = path.file_name().unwrap_or_default().to_string_lossy().to_string();
                
                // Skip hidden files if filter is not looking for them
                if filename.starts_with('.') && !filter.starts_with('.') {
                    continue;
                }

                if !filter.is_empty() && !filename.to_lowercase().starts_with(&filter) {
                    continue;
                }

                let filepath_str = path.to_string_lossy().to_string();
                let is_dir = path.is_dir();

                let (icon, actions, score) = if is_dir {
                    let dir_actions = vec![
                        Action {
                            label: "Navigate into Folder".to_string(),
                            action_type: "navigate-dir".to_string(),
                            value: format!("ff {}/", filepath_str),
                        },
                        Action {
                            label: "Open in File Manager".to_string(),
                            action_type: "open-url".to_string(),
                            value: format!("file://{}", filepath_str),
                        }
                    ];
                    ("folder-symbolic".to_string(), dir_actions, 180)
                } else {
                    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
                    let file_icon = match ext.as_str() {
                        "pdf" => "document-send-symbolic".to_string(),
                        "txt" | "md" | "json" | "rs" | "py" | "js" | "c" | "cpp" | "h" | "sh" | "xml" | "toml" | "yaml" | "yml" => {
                            "text-x-generic-symbolic".to_string()
                        }
                        "png" | "jpg" | "jpeg" | "gif" | "svg" | "webp" => "image-x-generic-symbolic".to_string(),
                        "mp3" | "wav" | "ogg" | "flac" | "m4a" => "audio-x-generic-symbolic".to_string(),
                        "mp4" | "mkv" | "avi" | "mov" | "webm" => "video-x-generic-symbolic".to_string(),
                        "zip" | "tar" | "gz" | "xz" | "rar" | "7z" => "package-x-generic-symbolic".to_string(),
                        _ => "text-x-generic-symbolic".to_string(),
                    };
                    let file_actions = vec![
                        Action {
                            label: "Open File".to_string(),
                            action_type: "open-url".to_string(),
                            value: format!("file://{}", filepath_str),
                        },
                        Action {
                            label: "Open Parent Directory".to_string(),
                            action_type: "open-url".to_string(),
                            value: format!("file://{}", dir_to_list.to_string_lossy()),
                        }
                    ];
                    (file_icon, file_actions, 170)
                };

                results.push(ResultItem {
                    id: format!("fm-{}", filepath_str),
                    title: filename,
                    subtitle: Some(filepath_str),
                    icon: Some(icon),
                    category: "File Manager".to_string(),
                    score,
                    actions,
                });
            }
        }

        // Sort folders first, then alphabetically
        results.sort_by(|a, b| {
            b.score.cmp(&a.score).then_with(|| a.title.to_lowercase().cmp(&b.title.to_lowercase()))
        });

        results
    }
}
