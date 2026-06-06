use std::cell::RefCell;
use std::fs;
use std::path::PathBuf;
use std::rc::Rc;
use gtk4::prelude::*;
use libadwaita::prelude::*;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AppSettings {
    pub shortcut: String,
    pub search_font_size: i32,
    pub title_font_size: i32,
    pub window_width: i32,
    pub window_height: i32,
    pub color_scheme: String, // default, light, dark
    pub theme: String, // adwaita, tokyonight, dracula, catppuccin, gruvbox
    #[serde(default = "default_layout_mode")]
    pub layout_mode: String, // standard, focused, minimal
    pub file_preview_enabled: bool,
    #[serde(default = "default_true")]
    pub apps_enabled: bool,
    #[serde(default = "default_true")]
    pub calc_enabled: bool,
    #[serde(default = "default_true")]
    pub web_enabled: bool,
    #[serde(default = "default_true")]
    pub youtube_enabled: bool,
    #[serde(default = "default_true")]
    pub files_enabled: bool,
    #[serde(default = "default_true")]
    pub command_enabled: bool,
    #[serde(default = "default_true")]
    pub gnome_enabled: bool,
    #[serde(default = "default_true")]
    pub recent_enabled: bool,

    // New configuration fields:
    #[serde(default = "default_web_name")]
    pub web_custom_name: String,
    #[serde(default = "default_web_url")]
    pub web_custom_url: String,
    #[serde(default = "default_file_roots")]
    pub file_search_roots: String,
    #[serde(default = "default_cmd_prefix")]
    pub command_prefix: String,
    #[serde(default = "default_yt_url")]
    pub youtube_url: String,
}

fn default_true() -> bool {
    true
}

fn default_layout_mode() -> String {
    "standard".to_string()
}

fn default_web_name() -> String {
    "Google".to_string()
}

fn default_web_url() -> String {
    "https://google.com/search?q=".to_string()
}

fn default_file_roots() -> String {
    "~/Documents, ~/Downloads, ~/Desktop".to_string()
}

fn default_cmd_prefix() -> String {
    ">".to_string()
}

fn default_yt_url() -> String {
    "https://youtube.com/results?search_query=".to_string()
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            shortcut: "<Alt>space".to_string(),
            search_font_size: 16,
            title_font_size: 14,
            window_width: 650,
            window_height: 480,
            color_scheme: "default".to_string(),
            theme: "adwaita".to_string(),
            layout_mode: "standard".to_string(),
            file_preview_enabled: true,
            apps_enabled: true,
            calc_enabled: true,
            web_enabled: true,
            youtube_enabled: true,
            files_enabled: true,
            command_enabled: true,
            gnome_enabled: true,
            recent_enabled: true,
            web_custom_name: "Google".to_string(),
            web_custom_url: "https://google.com/search?q=".to_string(),
            file_search_roots: "~/Documents, ~/Downloads, ~/Desktop".to_string(),
            command_prefix: ">".to_string(),
            youtube_url: "https://youtube.com/results?search_query=".to_string(),
        }
    }
}

impl AppSettings {
    pub fn get_config_path() -> PathBuf {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/home/atharva".to_string());
        PathBuf::from(home).join(".config").join("spear").join("config.json")
    }

    pub fn load() -> Self {
        let path = Self::get_config_path();

        // Ensure themes directory and sample theme exist
        if let Some(parent) = path.parent() {
            let themes_dir = parent.join("themes");
            let _ = fs::create_dir_all(&themes_dir);
            let sample_theme = themes_dir.join("sample.json");
            if !sample_theme.exists() {
                 let sample_content = r##"{
  "window_bg_color": "#181a1f",
  "window_fg_color": "#abb2bf",
  "accent_bg_color": "#61afef",
  "accent_fg_color": "#282c34",
  "popover_bg_color": "#21252b",
  "popover_fg_color": "#abb2bf",
  "entry_bg_color": "#24283b",
  "text_color": "#5c6370"
}"##;
                let _ = fs::write(sample_theme, sample_content);
            }
        }

        if !path.exists() {
            let default_settings = Self::default();
            let _ = default_settings.save();
            return default_settings;
        }

        if let Ok(content) = fs::read_to_string(path) {
            if let Ok(settings) = serde_json::from_str::<Self>(&content) {
                return settings;
            }
        }
        Self::default()
    }

    pub fn save(&self) -> Result<(), std::io::Error> {
        let path = Self::get_config_path();
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let content = serde_json::to_string_pretty(self)?;
        fs::write(path, content)
    }
}

pub fn apply_color_scheme(scheme: &str) {
    let style_manager = libadwaita::StyleManager::default();
    match scheme {
        "dark" => style_manager.set_color_scheme(libadwaita::ColorScheme::PreferDark),
        "light" => style_manager.set_color_scheme(libadwaita::ColorScheme::PreferLight),
        _ => style_manager.set_color_scheme(libadwaita::ColorScheme::Default),
    }
}

pub fn show_settings_window(
    _parent: &libadwaita::ApplicationWindow,
    settings: Rc<RefCell<AppSettings>>,
    on_changed: impl Fn() + 'static,
) {
    let prefs_window = libadwaita::PreferencesWindow::builder()
        .title("Spear Settings")
        .default_width(450)
        .default_height(450)
        .build();

    let page = libadwaita::PreferencesPage::builder()
        .title("General")
        .icon_name("preferences-other-symbolic")
        .build();
    prefs_window.add(&page);

    let plugins_page = libadwaita::PreferencesPage::builder()
        .title("Plugins")
        .icon_name("preferences-system-symbolic")
        .build();
    prefs_window.add(&plugins_page);

    let plugins_group = libadwaita::PreferencesGroup::builder()
        .title("Built-in Plugins")
        .description("Enable or disable specific search plugins")
        .build();
    plugins_page.add(&plugins_group);

    // Applications Search Row
    let apps_row = libadwaita::ActionRow::builder()
        .title("Applications")
        .subtitle("Search and launch desktop applications")
        .build();
    let apps_switch = gtk4::Switch::builder()
        .active(settings.borrow().apps_enabled)
        .valign(gtk4::Align::Center)
        .build();
    apps_row.add_suffix(&apps_switch);
    plugins_group.add(&apps_row);

    // Calculator Row
    let calc_row = libadwaita::ActionRow::builder()
        .title("Calculator")
        .subtitle("Evaluate mathematical expressions on-the-fly")
        .build();
    let calc_switch = gtk4::Switch::builder()
        .active(settings.borrow().calc_enabled)
        .valign(gtk4::Align::Center)
        .build();
    calc_row.add_suffix(&calc_switch);
    plugins_group.add(&calc_row);

    // Web Search Row (Expander)
    let web_row = libadwaita::ExpanderRow::builder()
        .title("Web Search")
        .subtitle("Perform searches using Google, GitHub, or DuckDuckGo")
        .build();
    let web_switch = gtk4::Switch::builder()
        .active(settings.borrow().web_enabled)
        .valign(gtk4::Align::Center)
        .build();
    web_row.add_action(&web_switch);

    let web_url_subrow = libadwaita::ActionRow::builder()
        .title("Custom Search Engine URL")
        .subtitle("Base URL for queries (default: https://google.com/search?q=)")
        .build();
    let web_url_entry = gtk4::Entry::builder()
        .text(&settings.borrow().web_custom_url)
        .valign(gtk4::Align::Center)
        .width_request(220)
        .build();
    web_url_subrow.add_suffix(&web_url_entry);
    web_row.add_row(&web_url_subrow);
    plugins_group.add(&web_row);

    // YouTube Search Row (Expander)
    let youtube_row = libadwaita::ExpanderRow::builder()
        .title("YouTube Search")
        .subtitle("Quickly search for videos on YouTube")
        .build();
    let youtube_switch = gtk4::Switch::builder()
        .active(settings.borrow().youtube_enabled)
        .valign(gtk4::Align::Center)
        .build();
    youtube_row.add_action(&youtube_switch);

    let youtube_url_subrow = libadwaita::ActionRow::builder()
        .title("Target Search URL")
        .subtitle("Link template (default: https://youtube.com/results?search_query=)")
        .build();
    let youtube_url_entry = gtk4::Entry::builder()
        .text(&settings.borrow().youtube_url)
        .valign(gtk4::Align::Center)
        .width_request(220)
        .build();
    youtube_url_subrow.add_suffix(&youtube_url_entry);
    youtube_row.add_row(&youtube_url_subrow);
    plugins_group.add(&youtube_row);

    // File Search Row (Expander)
    let files_row = libadwaita::ExpanderRow::builder()
        .title("File Search / Manager")
        .subtitle("Find files and folders in home directories")
        .build();
    let files_switch = gtk4::Switch::builder()
        .active(settings.borrow().files_enabled)
        .valign(gtk4::Align::Center)
        .build();
    files_row.add_action(&files_switch);

    let files_roots_subrow = libadwaita::ActionRow::builder()
        .title("Indexed Directories")
        .subtitle("Comma-separated paths to search (e.g. ~/Documents, ~/Downloads)")
        .build();
    let files_roots_entry = gtk4::Entry::builder()
        .text(&settings.borrow().file_search_roots)
        .valign(gtk4::Align::Center)
        .width_request(220)
        .build();
    files_roots_subrow.add_suffix(&files_roots_entry);
    files_row.add_row(&files_roots_subrow);
    plugins_group.add(&files_row);

    // Run Command Row (Expander)
    let command_row = libadwaita::ExpanderRow::builder()
        .title("Run Command")
        .subtitle("Run shell commands using a prefix")
        .build();
    let command_switch = gtk4::Switch::builder()
        .active(settings.borrow().command_enabled)
        .valign(gtk4::Align::Center)
        .build();
    command_row.add_action(&command_switch);

    let command_prefix_subrow = libadwaita::ActionRow::builder()
        .title("Trigger Prefix")
        .subtitle("Trigger character (default: >)")
        .build();
    let command_prefix_entry = gtk4::Entry::builder()
        .text(&settings.borrow().command_prefix)
        .valign(gtk4::Align::Center)
        .width_request(80)
        .build();
    command_prefix_subrow.add_suffix(&command_prefix_entry);
    command_row.add_row(&command_prefix_subrow);
    plugins_group.add(&command_row);

    // GNOME Controls Row
    let gnome_row = libadwaita::ActionRow::builder()
        .title("GNOME System Controls")
        .subtitle("Power off, reboot, lock screen, and system actions")
        .build();
    let gnome_switch = gtk4::Switch::builder()
        .active(settings.borrow().gnome_enabled)
        .valign(gtk4::Align::Center)
        .build();
    gnome_row.add_suffix(&gnome_switch);
    plugins_group.add(&gnome_row);

    // Recent Files Row
    let recent_row = libadwaita::ActionRow::builder()
        .title("Recent Files Search")
        .subtitle("Search files opened recently across GNOME apps")
        .build();
    let recent_switch = gtk4::Switch::builder()
        .active(settings.borrow().recent_enabled)
        .valign(gtk4::Align::Center)
        .build();
    recent_row.add_suffix(&recent_switch);
    plugins_group.add(&recent_row);

    // Group 1: Appearance
    let appearance_group = libadwaita::PreferencesGroup::builder()
        .title("Appearance")
        .build();
    page.add(&appearance_group);

    // Color scheme combo row
    let color_row = libadwaita::ActionRow::builder()
        .title("Color Scheme")
        .subtitle("Select dark, light, or system default mode")
        .build();

    let combo = gtk4::ComboBoxText::new();
    combo.append(Some("default"), "System Default");
    combo.append(Some("light"), "Prefer Light");
    combo.append(Some("dark"), "Prefer Dark");
    combo.set_active_id(Some(&settings.borrow().color_scheme));
    combo.set_valign(gtk4::Align::Center);
    color_row.add_suffix(&combo);
    appearance_group.add(&color_row);

    // Theme combo row
    let theme_row = libadwaita::ActionRow::builder()
        .title("Theme Palette")
        .subtitle("Select a custom color palette")
        .build();

    let theme_combo = gtk4::ComboBoxText::new();
    theme_combo.append(Some("adwaita"), "Default (Adwaita)");
    theme_combo.append(Some("tokyonight"), "Tokyo Night");
    theme_combo.append(Some("dracula"), "Dracula");
    theme_combo.append(Some("catppuccin"), "Catppuccin Mocha");
    theme_combo.append(Some("gruvbox"), "Gruvbox Material");
    theme_combo.append(Some("jellybeans"), "Jellybeans");
    theme_combo.append(Some("min"), "Minimalist");

    // Dynamic custom themes from ~/.config/spear/themes/
    if let Ok(home) = std::env::var("HOME") {
        let themes_dir = PathBuf::from(home)
            .join(".config")
            .join("spear")
            .join("themes");
        if let Ok(entries) = fs::read_dir(themes_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() && path.extension().and_then(|e| e.to_str()) == Some("json") {
                    if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                        // Exclude sample.json from selection if you want, or include it
                        let mut chars = stem.chars();
                        let display_name = match chars.next() {
                            None => String::new(),
                            Some(c) => c.to_uppercase().collect::<String>() + chars.as_str(),
                        };
                        theme_combo.append(Some(stem), &display_name);
                    }
                }
            }
        }
    }

    theme_combo.set_active_id(Some(&settings.borrow().theme));
    theme_combo.set_valign(gtk4::Align::Center);
    theme_row.add_suffix(&theme_combo);
    appearance_group.add(&theme_row);

    // Layout mode combo row
    let layout_row = libadwaita::ActionRow::builder()
        .title("Layout Mode")
        .subtitle("Select launcher visual mode (Standard, Focused, Minimal)")
        .build();

    let layout_combo = gtk4::ComboBoxText::new();
    layout_combo.append(Some("standard"), "Standard");
    layout_combo.append(Some("focused"), "Focused");
    layout_combo.append(Some("minimal"), "Minimal");
    layout_combo.set_active_id(Some(&settings.borrow().layout_mode));
    layout_combo.set_valign(gtk4::Align::Center);
    layout_row.add_suffix(&layout_combo);
    appearance_group.add(&layout_row);


    // Window Width
    let width_row = libadwaita::ActionRow::builder()
        .title("Window Width")
        .subtitle("Set launcher window width in pixels (500-900)")
        .build();
    let width_adj = gtk4::Adjustment::new(
        settings.borrow().window_width as f64,
        500.0,
        900.0,
        10.0,
        50.0,
        0.0,
    );
    let width_spin = gtk4::SpinButton::new(Some(&width_adj), 1.0, 0);
    width_spin.set_valign(gtk4::Align::Center);
    width_row.add_suffix(&width_spin);
    appearance_group.add(&width_row);

    // Window Height
    let height_row = libadwaita::ActionRow::builder()
        .title("Window Height")
        .subtitle("Set launcher window default height (300-700)")
        .build();
    let height_adj = gtk4::Adjustment::new(
        settings.borrow().window_height as f64,
        300.0,
        700.0,
        10.0,
        50.0,
        0.0,
    );
    let height_spin = gtk4::SpinButton::new(Some(&height_adj), 1.0, 0);
    height_spin.set_valign(gtk4::Align::Center);
    height_row.add_suffix(&height_spin);
    appearance_group.add(&height_row);

    // File Preview Toggle
    let preview_row = libadwaita::ActionRow::builder()
        .title("Enable File Preview")
        .subtitle("Show a rich details pane for selected files and items")
        .build();
    let preview_switch = gtk4::Switch::builder()
        .active(settings.borrow().file_preview_enabled)
        .valign(gtk4::Align::Center)
        .build();
    preview_row.add_suffix(&preview_switch);
    appearance_group.add(&preview_row);

    // Group 2: Typography
    let typo_group = libadwaita::PreferencesGroup::builder()
        .title("Typography")
        .build();
    page.add(&typo_group);

    // Search Font Size
    let search_font_row = libadwaita::ActionRow::builder()
        .title("Search Font Size")
        .subtitle("Set font size of the main search input (12-24)")
        .build();
    let s_font_adj = gtk4::Adjustment::new(
        settings.borrow().search_font_size as f64,
        12.0,
        24.0,
        1.0,
        5.0,
        0.0,
    );
    let s_font_spin = gtk4::SpinButton::new(Some(&s_font_adj), 1.0, 0);
    s_font_spin.set_valign(gtk4::Align::Center);
    search_font_row.add_suffix(&s_font_spin);
    typo_group.add(&search_font_row);

    // Title Font Size
    let title_font_row = libadwaita::ActionRow::builder()
        .title("Result Font Size")
        .subtitle("Set font size of search items titles (10-18)")
        .build();
    let t_font_adj = gtk4::Adjustment::new(
        settings.borrow().title_font_size as f64,
        10.0,
        18.0,
        1.0,
        5.0,
        0.0,
    );
    let t_font_spin = gtk4::SpinButton::new(Some(&t_font_adj), 1.0, 0);
    t_font_spin.set_valign(gtk4::Align::Center);
    title_font_row.add_suffix(&t_font_spin);
    typo_group.add(&title_font_row);

    // Group 3: Shortcuts
    let shortcut_group = libadwaita::PreferencesGroup::builder()
        .title("Shortcuts")
        .build();
    page.add(&shortcut_group);

    // Keyboard Shortcut Entry
    let shortcut_row = libadwaita::ActionRow::builder()
        .title("GNOME Global Hotkey")
        .subtitle("Re-registers shortcut (e.g. Super+space, Alt+space)")
        .build();
    let shortcut_entry = gtk4::Entry::builder()
        .text(&settings.borrow().shortcut)
        .valign(gtk4::Align::Center)
        .width_request(120)
        .build();
    shortcut_row.add_suffix(&shortcut_entry);
    shortcut_group.add(&shortcut_row);

    // Button to Apply Shortcut Settings
    let apply_btn_row = libadwaita::ActionRow::builder()
        .title("Apply Shortcut Change")
        .subtitle("Saves config and registers hotkey with GNOME Settings")
        .build();
    let apply_btn = gtk4::Button::builder()
        .label("Register Hotkey")
        .valign(gtk4::Align::Center)
        .build();
    apply_btn_row.add_suffix(&apply_btn);
    shortcut_group.add(&apply_btn_row);

    // Connect handlers for change and save events
    let settings_clone = settings.clone();
    let on_changed_rc = Rc::new(on_changed);

    // Helper closure to save settings and run callback
    let save_and_notify = {
        let settings = settings_clone.clone();
        let on_changed = on_changed_rc.clone();
        move || {
            let _ = settings.borrow().save();
            on_changed();
        }
    };

    // Color combo change
    let sn_color = save_and_notify.clone();
    let settings_color = settings_clone.clone();
    combo.connect_changed(move |cb| {
        if let Some(id) = cb.active_id() {
            settings_color.borrow_mut().color_scheme = id.to_string();
            apply_color_scheme(&id);
            sn_color();
        }
    });

    // Width spin change

    // Theme combo change
    let sn_theme = save_and_notify.clone();
    let settings_theme = settings_clone.clone();
    theme_combo.connect_changed(move |cb| {
        if let Some(id) = cb.active_id() {
            settings_theme.borrow_mut().theme = id.to_string();
            sn_theme();
        }
    });

    // Layout combo change
    let sn_layout = save_and_notify.clone();
    let settings_layout = settings_clone.clone();
    layout_combo.connect_changed(move |cb| {
        if let Some(id) = cb.active_id() {
            settings_layout.borrow_mut().layout_mode = id.to_string();
            sn_layout();
        }
    });

    let sn_width = save_and_notify.clone();
    let settings_width = settings_clone.clone();
    width_spin.connect_value_changed(move |sb| {
        settings_width.borrow_mut().window_width = sb.value() as i32;
        sn_width();
    });

    // Height spin change
    let sn_height = save_and_notify.clone();
    let settings_height = settings_clone.clone();
    height_spin.connect_value_changed(move |sb| {
        settings_height.borrow_mut().window_height = sb.value() as i32;
        sn_height();
    });

    // File preview change
    let sn_preview = save_and_notify.clone();
    let settings_preview = settings_clone.clone();
    preview_switch.connect_active_notify(move |sw| {
        settings_preview.borrow_mut().file_preview_enabled = sw.is_active();
        sn_preview();
    });

    // Apps switch change
    let sn_apps = save_and_notify.clone();
    let settings_apps = settings_clone.clone();
    apps_switch.connect_active_notify(move |sw| {
        settings_apps.borrow_mut().apps_enabled = sw.is_active();
        sn_apps();
    });

    // Calc switch change
    let sn_calc = save_and_notify.clone();
    let settings_calc = settings_clone.clone();
    calc_switch.connect_active_notify(move |sw| {
        settings_calc.borrow_mut().calc_enabled = sw.is_active();
        sn_calc();
    });

    // Web switch change
    let sn_web = save_and_notify.clone();
    let settings_web = settings_clone.clone();
    web_switch.connect_active_notify(move |sw| {
        settings_web.borrow_mut().web_enabled = sw.is_active();
        sn_web();
    });

    // YouTube switch change
    let sn_youtube = save_and_notify.clone();
    let settings_youtube = settings_clone.clone();
    youtube_switch.connect_active_notify(move |sw| {
        settings_youtube.borrow_mut().youtube_enabled = sw.is_active();
        sn_youtube();
    });

    // Files switch change
    let sn_files = save_and_notify.clone();
    let settings_files = settings_clone.clone();
    files_switch.connect_active_notify(move |sw| {
        settings_files.borrow_mut().files_enabled = sw.is_active();
        sn_files();
    });

    // Command switch change
    let sn_command = save_and_notify.clone();
    let settings_command = settings_clone.clone();
    command_switch.connect_active_notify(move |sw| {
        settings_command.borrow_mut().command_enabled = sw.is_active();
        sn_command();
    });

    // GNOME switch change
    let sn_gnome = save_and_notify.clone();
    let settings_gnome = settings_clone.clone();
    gnome_switch.connect_active_notify(move |sw| {
        settings_gnome.borrow_mut().gnome_enabled = sw.is_active();
        sn_gnome();
    });

    // Recent switch change
    let sn_recent = save_and_notify.clone();
    let settings_recent = settings_clone.clone();
    recent_switch.connect_active_notify(move |sw| {
        settings_recent.borrow_mut().recent_enabled = sw.is_active();
        sn_recent();
    });

    // Web Custom URL entry change
    let settings_web_url = settings_clone.clone();
    let sn_web_url = save_and_notify.clone();
    web_url_entry.connect_changed(move |entry| {
        settings_web_url.borrow_mut().web_custom_url = entry.text().to_string();
        sn_web_url();
    });

    // YouTube URL entry change
    let settings_yt_url = settings_clone.clone();
    let sn_yt_url = save_and_notify.clone();
    youtube_url_entry.connect_changed(move |entry| {
        settings_yt_url.borrow_mut().youtube_url = entry.text().to_string();
        sn_yt_url();
    });

    // File search roots entry change
    let settings_file_roots = settings_clone.clone();
    let sn_file_roots = save_and_notify.clone();
    files_roots_entry.connect_changed(move |entry| {
        settings_file_roots.borrow_mut().file_search_roots = entry.text().to_string();
        sn_file_roots();
    });

    // Command prefix entry change
    let settings_cmd_prefix = settings_clone.clone();
    let sn_cmd_prefix = save_and_notify.clone();
    command_prefix_entry.connect_changed(move |entry| {
        settings_cmd_prefix.borrow_mut().command_prefix = entry.text().to_string();
        sn_cmd_prefix();
    });

    // Search Font size change
    let sn_sfont = save_and_notify.clone();
    let settings_sfont = settings_clone.clone();
    s_font_spin.connect_value_changed(move |sb| {
        settings_sfont.borrow_mut().search_font_size = sb.value() as i32;
        sn_sfont();
    });

    // Title Font size change
    let sn_tfont = save_and_notify.clone();
    let settings_tfont = settings_clone.clone();
    t_font_spin.connect_value_changed(move |sb| {
        settings_tfont.borrow_mut().title_font_size = sb.value() as i32;
        sn_tfont();
    });

    // Apply button handler to trigger python installer shortcut binding update
    let settings_shortcut = settings_clone.clone();
    apply_btn.connect_clicked(move |_| {
        let shortcut_val = shortcut_entry.text().to_string();
        settings_shortcut.borrow_mut().shortcut = shortcut_val.clone();
        let _ = settings_shortcut.borrow().save();

        // Run install.py asynchronously in background to re-bind keyboard keys in GNOME settings!
        let home = std::env::var("HOME").unwrap_or_else(|_| "/home/atharva".to_string());
        let project_dir = format!("{}/Projects/Spear", home);

        let _ = std::process::Command::new("python3")
            .arg("install.py")
            .arg(&shortcut_val)
            .current_dir(project_dir)
            .spawn();

        println!("GNOME hotkey update triggered: {}", shortcut_val);
    });

    prefs_window.present();
}
