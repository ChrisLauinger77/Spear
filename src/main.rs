mod utils;
mod plugins;
mod plugin_manager;
mod settings;
mod window;
mod layer_shell;

use std::sync::Arc;
use std::rc::Rc;
use std::cell::RefCell;
use libadwaita::prelude::*;
use crate::plugin_manager::PluginManager;
use crate::window::SpearWindow;
use crate::settings::AppSettings;

thread_local! {
    static WINDOW: std::cell::RefCell<Option<SpearWindow>> = const { std::cell::RefCell::new(None) };
    static HOLD_GUARD: std::cell::RefCell<Option<gio::ApplicationHoldGuard>> = const { std::cell::RefCell::new(None) };
}

fn run_init_setup() -> Result<(), String> {
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::process::Command;
    use std::io::Write;

    let home = std::env::var("HOME").map_err(|e| format!("HOME not set: {e}"))?;
    let home_path = PathBuf::from(&home);
    let marker_path = home_path.join(".config").join("spear").join(".init_setup_done");

    if marker_path.exists() {
        println!("Spear initial setup has already been run. Skipping.");
        return Ok(());
    }

    println!("🚀  Starting Spear Initial Setup...");

    // 1. Create config directory and save default settings if not exists
    let settings = AppSettings::load();
    let _ = settings.save();
    println!("✅  Created default config at {:?}", AppSettings::get_config_path());

    // 2. Copy packaged system icons to user directory if they exist
    let system_icons_dir = Path::new("/usr/share/spear/icons");
    let user_icons_dir = home_path.join(".config").join("spear").join("icons");
    if system_icons_dir.exists() {
        let _ = fs::create_dir_all(&user_icons_dir);
        if let Ok(entries) = fs::read_dir(system_icons_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    if let Some(name) = path.file_name() {
                        let dest = user_icons_dir.join(name);
                        let _ = fs::copy(&path, &dest);
                    }
                }
            }
            println!("🎨  Copied system icons to user config directory.");
        }
    }

    // 3. Create autostart entry
    let autostart_dir = home_path.join(".config").join("autostart");
    let _ = fs::create_dir_all(&autostart_dir);
    let autostart_file = autostart_dir.join("spear.desktop");

    let binary_path =
        std::env::current_exe().unwrap_or_else(|_| PathBuf::from("/usr/bin/spear"));

    let autostart_content = format!(
        "[Desktop Entry]\n\
         Name=Spear Daemon\n\
         Comment=Start Spear launcher daemon in background\n\
         Exec=bash -c \"{binary} --quit ; {binary}\"\n\
         Icon=com.antigravity.spear\n\
         Terminal=false\n\
         Type=Application\n\
         Categories=Utility;\n\
         StartupNotify=false\n\
         X-GNOME-Autostart-enabled=true\n",
        binary = binary_path.display()
    );

    let mut f = fs::File::create(&autostart_file)
        .map_err(|e| format!("Could not create autostart entry: {e}"))?;
    f.write_all(autostart_content.as_bytes())
        .map_err(|e| format!("Could not write autostart content: {e}"))?;
    println!("✅  Created startup entry at {:?}", autostart_file);

    // 4. Register GNOME global shortcut
    let shortcut = &settings.shortcut;
    println!("⌨️   Registering GNOME global shortcut ({shortcut})…");

    // Read existing custom keybindings
    let existing_output = Command::new("gsettings")
        .args([
            "get",
            "org.gnome.settings-daemon.plugins.media-keys",
            "custom-keybindings",
        ])
        .output();

    if let Ok(output) = existing_output {
        let existing = String::from_utf8_lossy(&output.stdout).trim().to_string();
        let mut paths: Vec<String> = if existing == "@as []" || existing == "[]" || existing.is_empty() {
            vec![]
        } else {
            existing
                .trim_matches(|c| c == '[' || c == ']')
                .split(',')
                .map(|s| s.trim().trim_matches('\'').to_string())
                .filter(|s| !s.is_empty())
                .collect()
        };

        // Check if we already have a "Spear" binding; find or create a slot
        let mut spear_path: Option<String> = None;
        for p in &paths {
            let out = Command::new("gsettings")
                .args([
                    "get",
                    &format!("org.gnome.settings-daemon.plugins.media-keys.custom-keybinding:{p}"),
                    "name",
                ])
                .output();
            if let Ok(o) = out {
                let name = String::from_utf8_lossy(&o.stdout)
                    .trim()
                    .trim_matches('\'')
                    .to_string();
                if name == "Spear" {
                    spear_path = Some(p.clone());
                    break;
                }
            }
        }

        if spear_path.is_none() {
            // Find next free custom index
            let indices: Vec<usize> = paths
                .iter()
                .filter_map(|p| {
                    p.trim_end_matches('/')
                        .rsplit("custom")
                        .next()
                        .and_then(|n| n.parse().ok())
                })
                .collect();
            let mut idx = 0usize;
            while indices.contains(&idx) {
                idx += 1;
            }
            let new_path = format!(
                "/org/gnome/settings-daemon/plugins/media-keys/custom-keybindings/custom{idx}/"
            );
            paths.push(new_path.clone());
            spear_path = Some(new_path);
        }

        let sp = spear_path.unwrap();
        let schema = format!(
            "org.gnome.settings-daemon.plugins.media-keys.custom-keybinding:{sp}"
        );

        let run = |args: &[&str]| -> Result<(), String> {
            let status = Command::new("gsettings")
                .args(args)
                .status()
                .map_err(|e| format!("gsettings error: {e}"))?;
            if !status.success() {
                return Err(format!("gsettings failed: {args:?}"));
            }
            Ok(())
        };

        let _ = run(&["set", &schema, "name", "Spear"]);
        let _ = run(&["set", &schema, "command", &binary_path.to_string_lossy()]);
        let _ = run(&["set", &schema, "binding", shortcut]);

        // Write back the updated paths list
        let list = paths
            .iter()
            .map(|p| format!("'{p}'"))
            .collect::<Vec<_>>()
            .join(", ");
        let list = format!("[{list}]");
        let _ = run(&[
            "set",
            "org.gnome.settings-daemon.plugins.media-keys",
            "custom-keybindings",
            &list,
        ]);
        println!("✅  GNOME shortcut '{shortcut}' registered.");
    } else {
        println!("⚠️  gsettings not available, skipping GNOME shortcut registration.");
    }

    // 5. Create the setup marker file
    let mut f = fs::File::create(&marker_path)
        .map_err(|e| format!("Could not create marker file: {e}"))?;
    f.write_all(b"done")
        .map_err(|e| format!("Could not write marker file: {e}"))?;
    println!("✅  Spear initial setup completed!");

    Ok(())
}

fn main() -> glib::ExitCode {
    let args: Vec<String> = std::env::args().collect();
    if args.contains(&"--init-setup".to_string()) {
        if let Err(e) = run_init_setup() {
            eprintln!("❌  Setup failed: {e}");
            std::process::exit(1);
        }
        std::process::exit(0);
    }

    let app = libadwaita::Application::builder()
        .application_id("com.antigravity.spear")
        .flags(gio::ApplicationFlags::HANDLES_COMMAND_LINE)
        .build();

    let plugin_manager = Arc::new(PluginManager::new());
    let settings = Rc::new(RefCell::new(AppSettings::load()));

    let settings_startup = settings.clone();
    app.connect_startup(move |app| {
        // Apply color scheme initially (GTK is now initialized)
        settings::apply_color_scheme(&settings_startup.borrow().color_scheme);

        // Hold the application to keep it running in the background using RAII guard
        let guard = app.hold();
        HOLD_GUARD.with(|cell| *cell.borrow_mut() = Some(guard));
    });

    let pm_clone = plugin_manager.clone();
    let settings_clone = settings.clone();
    app.connect_command_line(move |app, cmd_line| {
        let args = cmd_line.arguments();
        let args_str: Vec<String> = args
            .iter()
            .map(|s| s.to_string_lossy().to_string())
            .collect();

        // Check command-line options
        if args_str.contains(&"--help".to_string()) || args_str.contains(&"-h".to_string()) {
            let help_msg = "Spear Launcher Daemon\n\n\
                            Usage: spear [options]\n\n\
                            Options:\n\
                              -t, --toggle          Toggle window visibility (default if no options passed)\n\
                              -q, --quit            Quit the launcher background daemon\n\
                              -s, --status          Print the daemon running status\n\
                              -l, --list-plugins    List all enabled search plugins\n\
                              -c, --config          Print the configuration file path\n\
                              --init-setup          Run initial startup and hotkey setup\n\
                              -h, --help            Print this help message\n";
            cmd_line.print_literal(help_msg);
            return 0;
        }

        if args_str.contains(&"--quit".to_string()) || args_str.contains(&"-q".to_string()) {
            cmd_line.print_literal("Quitting Spear Launcher daemon...\n");
            HOLD_GUARD.with(|cell| {
                *cell.borrow_mut() = None; // Drops the guard, releasing the hold
            });
            app.quit();
            return 0;
        }

        if args_str.contains(&"--status".to_string()) || args_str.contains(&"-s".to_string()) {
            cmd_line.print_literal("Spear Launcher daemon is running.\n");
            return 0;
        }

        if args_str.contains(&"--config".to_string()) || args_str.contains(&"-c".to_string()) {
            let path = AppSettings::get_config_path();
            cmd_line.print_literal(&format!("Configuration file: {}\n", path.display()));
            return 0;
        }

        if args_str.contains(&"--list-plugins".to_string()) || args_str.contains(&"-l".to_string()) {
            let mut output = String::new();
            output.push_str("Internal Plugins:\n");
            for plugin in &pm_clone.internal_plugins {
                let is_enabled = match plugin.id() {
                    "apps" => settings_clone.borrow().apps_enabled,
                    "calc" => settings_clone.borrow().calc_enabled,
                    "web" => settings_clone.borrow().web_enabled,
                    "youtube" => settings_clone.borrow().youtube_enabled,
                    "files" => settings_clone.borrow().files_enabled,
                    "command" => settings_clone.borrow().command_enabled,
                    "gnome" => settings_clone.borrow().gnome_enabled,
                    "recent" => settings_clone.borrow().recent_enabled,
                    _ => true,
                };
                let status_str = if is_enabled { "enabled" } else { "disabled" };
                output.push_str(&format!("  - {} [{}] ({})\n", plugin.name(), plugin.id(), status_str));
            }
            output.push_str("\nExternal Plugins:\n");
            if pm_clone.external_plugins.is_empty() {
                output.push_str("  (None)\n");
            } else {
                for plugin in &pm_clone.external_plugins {
                    let kw_str = plugin.keyword.as_deref().unwrap_or("none");
                    output.push_str(&format!("  - {} [{}] (keyword: {})\n", plugin.name, plugin.id, kw_str));
                }
            }
            cmd_line.print_literal(&output);
            return 0;
        }

        // Initialize and toggle window
        let pm = pm_clone.clone();
        let settings = settings_clone.clone();
        WINDOW.with(|cell| {
            let mut opt = cell.borrow_mut();
            if opt.is_none() {
                *opt = Some(SpearWindow::new(app, pm, settings));
            }
            opt.as_ref().unwrap().toggle_visibility();
        });

        0
    });

    app.run()
}
