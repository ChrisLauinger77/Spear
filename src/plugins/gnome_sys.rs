use crate::plugins::{Action, ResultItem, Plugin};

pub struct GnomeSysPlugin {
    actions: Vec<GnomeAction>,
}

struct GnomeAction {
    id: &'static str,
    title: &'static str,
    keywords: Vec<&'static str>,
    subtitle: &'static str,
    icon: &'static str,
    command: &'static str,
}

impl GnomeSysPlugin {
    pub fn new() -> Self {
        Self {
            actions: vec![
                GnomeAction {
                    id: "gnome-lock-screen",
                    title: "Lock Screen",
                    keywords: vec!["lock", "screen", "saver", "suspend"],
                    subtitle: "Lock the current desktop session",
                    icon: "system-lock-screen-symbolic",
                    command: "dbus-send --type=method_call --dest=org.gnome.ScreenSaver /org/gnome/ScreenSaver org.gnome.ScreenSaver.Lock",
                },
                GnomeAction {
                    id: "gnome-logout",
                    title: "Log Out",
                    keywords: vec!["logout", "log out", "exit", "quit"],
                    subtitle: "Log out of the current desktop session",
                    icon: "system-log-out-symbolic",
                    command: "gnome-session-quit --logout --no-prompt",
                },
                GnomeAction {
                    id: "gnome-suspend",
                    title: "Suspend / Sleep",
                    keywords: vec!["suspend", "sleep", "hibernate"],
                    subtitle: "Put the system to sleep",
                    icon: "weather-clear-night-symbolic",
                    command: "systemctl suspend",
                },
                GnomeAction {
                    id: "gnome-reboot",
                    title: "Reboot / Restart",
                    keywords: vec!["reboot", "restart"],
                    subtitle: "Restart the computer",
                    icon: "system-reboot-symbolic",
                    command: "systemctl reboot",
                },
                GnomeAction {
                    id: "gnome-shutdown",
                    title: "Shutdown / Power Off",
                    keywords: vec!["shutdown", "poweroff", "power off", "halt", "turn off"],
                    subtitle: "Power off the computer",
                    icon: "system-shutdown-symbolic",
                    command: "systemctl poweroff",
                },
                GnomeAction {
                    id: "gnome-settings",
                    title: "GNOME Settings",
                    keywords: vec!["settings", "control center", "preferences", "gnome settings"],
                    subtitle: "Open system settings configuration window",
                    icon: "preferences-system-symbolic",
                    command: "gnome-control-center",
                },
                GnomeAction {
                    id: "gnome-settings-wifi",
                    title: "Settings: Wi-Fi",
                    keywords: vec!["wifi", "wi-fi", "wireless", "internet", "network", "lan"],
                    subtitle: "Configure wireless network connections",
                    icon: "network-wireless-symbolic",
                    command: "gnome-control-center wifi",
                },
                GnomeAction {
                    id: "gnome-settings-bluetooth",
                    title: "Settings: Bluetooth",
                    keywords: vec!["bluetooth", "blue", "wireless", "pair", "device", "headphones"],
                    subtitle: "Configure Bluetooth devices and pairing",
                    icon: "bluetooth-symbolic",
                    command: "gnome-control-center bluetooth",
                },
                GnomeAction {
                    id: "gnome-settings-display",
                    title: "Settings: Displays",
                    keywords: vec!["display", "monitor", "screen", "resolution", "projector", "scale"],
                    subtitle: "Configure monitors, resolution, and scaling",
                    icon: "video-display-symbolic",
                    command: "gnome-control-center display",
                },
                GnomeAction {
                    id: "gnome-settings-sound",
                    title: "Settings: Sound",
                    keywords: vec!["sound", "audio", "volume", "speaker", "microphone", "headphones"],
                    subtitle: "Configure system sound volume and audio devices",
                    icon: "audio-volume-high-symbolic",
                    command: "gnome-control-center sound",
                },
                GnomeAction {
                    id: "gnome-settings-power",
                    title: "Settings: Power & Battery",
                    keywords: vec!["power", "battery", "energy", "charging", "suspend", "sleep", "screen off"],
                    subtitle: "Configure power savings and screen timeout settings",
                    icon: "battery-symbolic",
                    command: "gnome-control-center power",
                },
                GnomeAction {
                    id: "gnome-settings-background",
                    title: "Settings: Background & Wallpaper",
                    keywords: vec!["background", "wallpaper", "desktop", "theme", "style", "appearance"],
                    subtitle: "Change desktop wallpaper and system theme settings",
                    icon: "preferences-desktop-wallpaper-symbolic",
                    command: "gnome-control-center background",
                },
                GnomeAction {
                    id: "gnome-settings-keyboard",
                    title: "Settings: Keyboard Shortcuts",
                    keywords: vec!["keyboard", "shortcuts", "hotkeys", "input", "layout"],
                    subtitle: "Configure keyboard layouts and system shortcuts",
                    icon: "input-keyboard-symbolic",
                    command: "gnome-control-center keyboard",
                },
                GnomeAction {
                    id: "gnome-settings-mouse",
                    title: "Settings: Mouse & Touchpad",
                    keywords: vec!["mouse", "touchpad", "trackpad", "pointer", "click", "scroll"],
                    subtitle: "Configure mouse speed and touchpad gestures",
                    icon: "input-mouse-symbolic",
                    command: "gnome-control-center mouse",
                },
                GnomeAction {
                    id: "gnome-settings-printers",
                    title: "Settings: Printers",
                    keywords: vec!["printer", "print", "scanner", "devices"],
                    subtitle: "Configure connected printers and print queues",
                    icon: "printer-symbolic",
                    command: "gnome-control-center printers",
                },
                GnomeAction {
                    id: "gnome-settings-info",
                    title: "Settings: About System Info",
                    keywords: vec!["about", "system info", "specifications", "info", "version", "hardware"],
                    subtitle: "View OS version, CPU, RAM, and hardware details",
                    icon: "dialog-information-symbolic",
                    command: "gnome-control-center info-overview",
                },
                GnomeAction {
                    id: "gnome-settings-datetime",
                    title: "Settings: Date & Time",
                    keywords: vec!["date", "time", "clock", "timezone", "calendar", "year"],
                    subtitle: "Configure system date, time, and timezone clock settings",
                    icon: "preferences-system-time-symbolic",
                    command: "gnome-control-center datetime",
                },
                GnomeAction {
                    id: "gnome-empty-trash",
                    title: "Empty Trash",
                    keywords: vec!["empty trash", "trash", "clear trash", "delete trash"],
                    subtitle: "Permanently delete all items in trash",
                    icon: "user-trash-symbolic",
                    command: "rm -rf ~/.local/share/Trash/files/* ~/.local/share/Trash/info/*",
                },
            ],
        }
    }
}

impl Plugin for GnomeSysPlugin {
    fn id(&self) -> &str {
        "gnome"
    }

    fn name(&self) -> &str {
        "GNOME System Controls"
    }

    fn query(&self, query_text: &str, _settings: &crate::settings::AppSettings) -> Vec<ResultItem> {
        let query_trimmed = query_text.trim();
        if query_trimmed.is_empty() {
            return Vec::new();
        }

        let query_lower = query_trimmed.to_lowercase();
        let mut results = Vec::new();

        for action in &self.actions {
            let matches_title = action.title.to_lowercase().contains(&query_lower);
            let matches_keyword = action.keywords.iter().any(|k| k.contains(&query_lower));

            if matches_title || matches_keyword {
                let score = if action.title.to_lowercase() == query_lower {
                    95
                } else if action.title.to_lowercase().starts_with(&query_lower) {
                    85
                } else {
                    75
                };

                results.push(ResultItem {
                    id: action.id.to_string(),
                    title: action.title.to_string(),
                    subtitle: Some(action.subtitle.to_string()),
                    icon: Some(action.icon.to_string()),
                    category: "GNOME Controls".to_string(),
                    score,
                    actions: vec![Action {
                        label: action.title.to_string(),
                        action_type: "run-command".to_string(),
                        value: action.command.to_string(),
                    }],
                });
            }
        }

        results
    }
}
