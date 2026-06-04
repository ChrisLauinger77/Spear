use std::path::PathBuf;
use std::fs;
use std::thread;
use std::process::Command;
use crate::plugins::{Plugin, ResultItem, Action};
use crate::plugins::apps::AppsPlugin;
use crate::plugins::calc::CalculatorPlugin;
use crate::plugins::web::WebSearchPlugin;
use crate::plugins::youtube::YouTubeSearchPlugin;
use crate::plugins::files::FilesPlugin;
use crate::plugins::run_cmd::RunCommandPlugin;
use crate::plugins::gnome_sys::GnomeSysPlugin;
use crate::plugins::recent::RecentFilesPlugin;
use crate::plugins::file_manager::FileManagerPlugin;

#[derive(Debug, Clone, serde::Deserialize)]
pub struct ExternalManifest {
    pub name: String,
    pub description: Option<String>,
    pub keyword: Option<String>,
    pub command: Vec<String>,
    pub icon: Option<String>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ExternalPlugin {
    pub id: String,
    pub name: String,
    pub description: String,
    pub keyword: Option<String>,
    pub command: Vec<String>,
    pub icon: Option<String>,
    pub folder_path: PathBuf,
}

pub struct PluginManager {
    pub internal_plugins: Vec<Box<dyn Plugin>>,
    pub external_plugins: Vec<ExternalPlugin>,
    pub plugins_dir: PathBuf,
}

impl PluginManager {
    pub fn new() -> Self {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/home/atharva".to_string());
        let config_dir = PathBuf::from(home).join(".config").join("spear");
        let plugins_dir = config_dir.join("plugins");
        
        let mut manager = Self {
            internal_plugins: Vec::new(),
            external_plugins: Vec::new(),
            plugins_dir,
        };

        let _ = fs::create_dir_all(&manager.plugins_dir);
        manager.load_internal_plugins();
        manager.reload_external_plugins();
        manager.create_sample_plugin_if_needed();

        manager
    }

    fn load_internal_plugins(&mut self) {
        self.internal_plugins = vec![
            Box::new(AppsPlugin::new()),
            Box::new(CalculatorPlugin::new()),
            Box::new(WebSearchPlugin::new()),
            Box::new(YouTubeSearchPlugin::new()),
            Box::new(FilesPlugin::new()),
            Box::new(RunCommandPlugin::new()),
            Box::new(GnomeSysPlugin::new()),
            Box::new(RecentFilesPlugin::new()),
            Box::new(FileManagerPlugin::new()),
        ];
    }

    pub fn reload_external_plugins(&mut self) {
        self.external_plugins.clear();
        if !self.plugins_dir.exists() {
            return;
        }

        if let Ok(entries) = fs::read_dir(&self.plugins_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if !path.is_dir() {
                    continue;
                }

                let manifest_path = path.join("manifest.json");
                if !manifest_path.exists() {
                    continue;
                }

                if let Ok(content) = fs::read_to_string(&manifest_path) {
                    if let Ok(manifest) = serde_json::from_str::<ExternalManifest>(&content) {
                        let id = path.file_name().unwrap().to_string_lossy().to_string();
                        self.external_plugins.push(ExternalPlugin {
                            id,
                            name: manifest.name,
                            description: manifest.description.unwrap_or_default(),
                            keyword: manifest.keyword.map(|k| k.trim().to_string()),
                            command: manifest.command,
                            icon: manifest.icon,
                            folder_path: path,
                        });
                    }
                }
            }
        }
    }

    pub fn query(
        &self,
        query_text: &str,
        settings: crate::settings::AppSettings,
        sender: async_channel::Sender<(String, Vec<ResultItem>, Option<String>)>,
    ) {
        let query_trimmed = query_text.trim();
        
        // Check if query starts with an external plugin's keyword + space
        let mut matched_keyword_plugin = None;
        let mut external_query = query_trimmed.to_string();

        for plugin in &self.external_plugins {
            if let Some(ref keyword) = plugin.keyword {
                if !keyword.is_empty() {
                    let prefix = format!("{} ", keyword);
                    if query_trimmed.starts_with(&prefix) {
                        matched_keyword_plugin = Some(plugin.clone());
                        external_query = query_trimmed[prefix.len()..].trim().to_string();
                        break;
                    }
                }
            }
        }

        // 1. If keyword matches an external plugin, run ONLY that plugin and skip internal
        if let Some(plugin) = matched_keyword_plugin {
            self.run_external_async(plugin, external_query, sender.clone());
            // Send empty results for internal to clear the screen
            let _ = sender.send_blocking(("internal-skipped".to_string(), Vec::new(), None));
            return;
        }

        // 2. Otherwise run all internal plugins immediately on the main thread (they are fast)
        for plugin in &self.internal_plugins {
            let is_enabled = match plugin.id() {
                "apps" => settings.apps_enabled,
                "calc" => settings.calc_enabled,
                "web" => settings.web_enabled,
                "youtube" => settings.youtube_enabled,
                "files" => settings.files_enabled,
                "command" => settings.command_enabled,
                "gnome" => settings.gnome_enabled,
                "recent" => settings.recent_enabled,
                _ => true,
            };

            if is_enabled {
                let results = plugin.query(query_trimmed, &settings);
                let _ = sender.send_blocking((plugin.id().to_string(), results, None));
            } else {
                // Send empty results to clear out previous results of disabled plugins
                let _ = sender.send_blocking((plugin.id().to_string(), Vec::new(), None));
            }
        }

        // 3. Also run global external plugins in parallel background threads
        for plugin in &self.external_plugins {
            if plugin.keyword.is_none() || plugin.keyword.as_ref().unwrap().is_empty() {
                self.run_external_async(plugin.clone(), query_trimmed.to_string(), sender.clone());
            }
        }
    }

    fn run_external_async(
        &self,
        plugin: ExternalPlugin,
        query: String,
        sender: async_channel::Sender<(String, Vec<ResultItem>, Option<String>)>,
    ) {
        thread::spawn(move || {
            let mut cmd_args = plugin.command.clone();
            if cmd_args.is_empty() {
                let _ = sender.send_blocking((plugin.id.clone(), Vec::new(), Some("Empty command".to_string())));
                return;
            }

            let program = cmd_args.remove(0);
            let mut command = Command::new(&program);
            command.args(cmd_args);
            command.arg(&query);
            command.current_dir(&plugin.folder_path);

            match command.output() {
                Ok(output) => {
                    if output.status.success() {
                        let stdout_str = String::from_utf8_lossy(&output.stdout);
                        // Parse JSON output
                        match serde_json::from_str::<Vec<serde_json::Value>>(&stdout_str) {
                            Ok(json_items) => {
                                let mut results = Vec::new();
                                for (idx, json_item) in json_items.into_iter().enumerate() {
                                    let title = json_item
                                        .get("title")
                                        .and_then(|v| v.as_str())
                                        .unwrap_or("No Title")
                                        .to_string();
                                    let subtitle = json_item
                                        .get("subtitle")
                                        .and_then(|v| v.as_str())
                                        .map(|s| s.to_string());
                                    let icon = json_item
                                        .get("icon")
                                        .and_then(|v| v.as_str())
                                        .map(|s| s.to_string())
                                        .or_else(|| plugin.icon.clone());
                                    let score = json_item
                                        .get("score")
                                        .and_then(|v| v.as_i64())
                                        .unwrap_or(30) as i32;

                                    let mut actions = Vec::new();
                                    if let Some(json_actions) = json_item.get("actions").and_then(|v| v.as_array()) {
                                        for action_val in json_actions {
                                            let label = action_val
                                                .get("label")
                                                .and_then(|v| v.as_str())
                                                .unwrap_or("Execute")
                                                .to_string();
                                            let action_type = action_val
                                                .get("type")
                                                .and_then(|v| v.as_str())
                                                .unwrap_or("run-command")
                                                .to_string();
                                            let value = action_val
                                                .get("value")
                                                .and_then(|v| v.as_str())
                                                .unwrap_or("")
                                                .to_string();

                                            actions.push(Action { label, action_type, value });
                                        }
                                    }

                                    results.push(ResultItem {
                                        id: json_item
                                            .get("id")
                                            .and_then(|v| v.as_str())
                                            .map(|s| s.to_string())
                                            .unwrap_or_else(|| format!("ext-{}-{}", plugin.id, idx)),
                                        title,
                                        subtitle,
                                        icon,
                                        category: plugin.name.clone(),
                                        score,
                                        actions,
                                    });
                                }
                                let _ = sender.send_blocking((plugin.id, results, None));
                            }
                            Err(e) => {
                                let _ = sender.send_blocking((
                                    plugin.id,
                                    Vec::new(),
                                    Some(format!("JSON Parse Error: {}", e)),
                                ));
                            }
                        }
                    } else {
                        let stderr_str = String::from_utf8_lossy(&output.stderr);
                        let _ = sender.send_blocking((
                            plugin.id,
                            Vec::new(),
                            Some(format!("Exit code {}: {}", output.status, stderr_str)),
                        ));
                    }
                }
                Err(e) => {
                    let _ = sender.send_blocking((plugin.id, Vec::new(), Some(format!("Execute Error: {}", e))));
                }
            }
        });
    }

    fn create_sample_plugin_if_needed(&self) {
        let sample_dir = self.plugins_dir.join("hello-world");
        if sample_dir.exists() {
            return;
        }

        let _ = fs::create_dir_all(&sample_dir);

        // 1. manifest.json
        let manifest_content = r#"{
    "name": "Hello World",
    "description": "A sample Python plugin for Spear Launcher",
    "keyword": "hello",
    "command": ["python3", "main.py"],
    "icon": "emblem-favorite-symbolic"
}"#;
        let _ = fs::write(sample_dir.join("manifest.json"), manifest_content);

        // 2. main.py
        let python_code = r#"import sys
import json

def main():
    query = sys.argv[1] if len(sys.argv) > 1 else ""
    
    results = [
        {
            "title": f"Hello, {query}!" if query else "Hello World Plugin",
            "subtitle": "This is a custom plugin result from Python!",
            "icon": "emblem-favorite-symbolic",
            "actions": [
                {
                    "label": "Copy Greeting",
                    "type": "copy-to-clipboard",
                    "value": f"Hello, {query}!" if query else "Hello World!"
                },
                {
                    "label": "Search on Google",
                    "type": "open-url",
                    "value": f"https://google.com/search?q={query}"
                }
            ]
        }
    ]
    print(json.dumps(results))

if __name__ == "__main__":
    main()
"#;
        let _ = fs::write(sample_dir.join("main.py"), python_code);

        // Make executable if on Unix
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if let Ok(meta) = fs::metadata(sample_dir.join("main.py")) {
                let mut perms = meta.permissions();
                perms.set_mode(0o755);
                let _ = fs::set_permissions(sample_dir.join("main.py"), perms);
            }
        }
    }
}
