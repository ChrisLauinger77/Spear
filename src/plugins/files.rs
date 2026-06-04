use std::fs;
use std::path::{Path, PathBuf};
use crate::plugins::{Action, ResultItem, Plugin};

pub struct FilesPlugin;

impl FilesPlugin {
    pub fn new() -> Self {
        Self
    }
}

// Check if a directory should be skipped to avoid scanning giant build/cache dirs
fn should_skip_dir(name: &str) -> bool {
    name.starts_with('.') 
        || name == "node_modules" 
        || name == "target" 
        || name == "build" 
        || name == "dist" 
        || name == "venv" 
        || name == ".git" 
        || name == "Cache" 
        || name == "cache"
}

// Recursively traverse and find matching files/folders
fn search_dir(
    dir: &Path,
    query_lower: &str,
    depth: usize,
    max_depth: usize,
    results: &mut Vec<(PathBuf, bool)>, // (Path, is_dir)
    visited_count: &mut usize,
    max_visited: usize,
) {
    if depth > max_depth || *visited_count >= max_visited {
        return;
    }

    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            *visited_count += 1;
            if *visited_count >= max_visited {
                return;
            }

            let path = entry.path();
            let file_name = match path.file_name() {
                Some(name) => name.to_string_lossy(),
                None => continue,
            };

            let is_dir = path.is_dir();
            
            // Check match
            if file_name.to_lowercase().contains(query_lower) {
                results.push((path.clone(), is_dir));
            }

            if is_dir {
                if should_skip_dir(&file_name) {
                    continue;
                }
                search_dir(&path, query_lower, depth + 1, max_depth, results, visited_count, max_visited);
            }
        }
    }
}

impl Plugin for FilesPlugin {
    fn id(&self) -> &str {
        "files"
    }

    fn name(&self) -> &str {
        "File Search"
    }

    fn query(&self, query_text: &str, settings: &crate::settings::AppSettings) -> Vec<ResultItem> {
        let query_trimmed = query_text.trim();
        if query_trimmed.is_empty() {
            return Vec::new();
        }

        let query_lower = query_trimmed.to_lowercase();
        let home_str = std::env::var("HOME").unwrap_or_else(|_| "/home/atharva".to_string());
        let home_path = PathBuf::from(&home_str);

        // Parse search roots from settings
        let mut search_roots = Vec::new();
        for root_raw in settings.file_search_roots.split(',') {
            let root_trimmed = root_raw.trim();
            if root_trimmed.is_empty() {
                continue;
            }
            let resolved = if root_trimmed.starts_with('~') {
                root_trimmed.replacen('~', &home_str, 1)
            } else {
                root_trimmed.to_string()
            };
            let path = PathBuf::from(resolved);
            if path.exists() && path.is_dir() {
                search_roots.push(path);
            }
        }

        // If no directories configured, fallback
        if search_roots.is_empty() {
            search_roots = vec![
                home_path.join("Documents"),
                home_path.join("Downloads"),
                home_path.join("Desktop"),
            ];
        }

        let mut matched_items = Vec::new();
        let mut visited_count = 0;
        let max_visited = 3000; // Limit traversal to keep it fast

        // 1. Search the specific subdirectories with recursion
        for root in &search_roots {
            search_dir(root, &query_lower, 1, 3, &mut matched_items, &mut visited_count, max_visited);
        }

        // 2. Search home root folder, but at top level only (depth 1)
        search_dir(&home_path, &query_lower, 1, 1, &mut matched_items, &mut visited_count, max_visited);

        // Deduplicate path matches
        matched_items.sort_by(|a, b| a.0.cmp(&b.0));
        matched_items.dedup_by(|a, b| a.0 == b.0);

        let mut results = Vec::new();
        for (path, is_dir) in matched_items {
            let filename = path.file_name().unwrap_or_default().to_string_lossy().to_string();
            let filepath_str = path.to_string_lossy().to_string();
            
            // Score calculations
            let filename_lower = filename.to_lowercase();
            let score = if filename_lower == query_lower {
                80
            } else if filename_lower.starts_with(&query_lower) {
                70
            } else {
                50
            };

            // Determine appropriate standard symbolic icon
            let icon = if is_dir {
                "folder-symbolic".to_string()
            } else {
                let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
                match ext.as_str() {
                    "pdf" => "document-send-symbolic".to_string(),
                    "txt" | "md" | "json" | "rs" | "py" | "js" | "c" | "cpp" | "h" | "sh" | "xml" | "toml" | "yaml" | "yml" => {
                        "text-x-generic-symbolic".to_string()
                    }
                    "png" | "jpg" | "jpeg" | "gif" | "svg" | "webp" => "image-x-generic-symbolic".to_string(),
                    "mp3" | "wav" | "ogg" | "flac" | "m4a" => "audio-x-generic-symbolic".to_string(),
                    "mp4" | "mkv" | "avi" | "mov" | "webm" => "video-x-generic-symbolic".to_string(),
                    "zip" | "tar" | "gz" | "xz" | "rar" | "7z" => "package-x-generic-symbolic".to_string(),
                    _ => "text-x-generic-symbolic".to_string(),
                }
            };

            results.push(ResultItem {
                id: format!("file-{}", filepath_str),
                title: filename,
                subtitle: Some(filepath_str.clone()),
                icon: Some(icon),
                category: "Files".to_string(),
                score,
                actions: vec![
                    Action {
                        label: "Open File".to_string(),
                        action_type: "open-url".to_string(),
                        value: format!("file://{}", filepath_str),
                    },
                    Action {
                        label: "Open Parent Directory".to_string(),
                        action_type: "open-url".to_string(),
                        value: format!("file://{}", path.parent().unwrap_or(&path).to_string_lossy()),
                    },
                ],
            });
        }

        // Sort by score desc, then by title
        results.sort_by(|a, b| b.score.cmp(&a.score).then_with(|| a.title.cmp(&b.title)));
        results.truncate(20);

        results
    }
}
