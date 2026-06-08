use crate::plugins::{Action, ResultItem, Plugin};

pub struct ColorPalettePlugin;

impl ColorPalettePlugin {
    pub fn new() -> Self {
        Self
    }

    fn get_predefined_colors(&self) -> Vec<(&str, &str)> {
        vec![
            ("White", "#FFFFFF"),
            ("Black", "#000000"),
            ("Red", "#FF0000"),
            ("Green", "#00FF00"),
            ("Blue", "#0000FF"),
            ("Yellow", "#FFFF00"),
            ("Cyan", "#00FFFF"),
            ("Magenta", "#FF00FF"),
            ("Spear Blue", "#61afef"),
            ("Adwaita Blue", "#3584e4"),
            ("Success Green", "#2ec27e"),
            ("Warning Orange", "#f5c211"),
            ("Error Red", "#e01b24"),
        ]
    }
}

impl Plugin for ColorPalettePlugin {
    fn id(&self) -> &str {
        "colors"
    }

    fn name(&self) -> &str {
        "Color Palette"
    }

    fn mode_keyword(&self) -> Option<&str> {
        Some("color")
    }

    fn is_mode_only(&self) -> bool {
        true
    }

    fn query(&self, query_text: &str, _settings: &crate::settings::AppSettings) -> Vec<ResultItem> {
        let query_lower = query_text.to_lowercase();
        let mut results = Vec::new();

        // Check if query is a hex code
        if query_text.starts_with('#') && (query_text.len() == 4 || query_text.len() == 7) {
            results.push(ResultItem {
                id: format!("color-hex-{}", query_text),
                title: format!("Color: {}", query_text),
                subtitle: Some("Hex Code".to_string()),
                icon: Some("color-management-symbolic".to_string()),
                category: "Colors".to_string(),
                score: 100,
                actions: vec![Action {
                    label: "Copy Hex Code".to_string(),
                    action_type: "copy-to-clipboard".to_string(),
                    value: query_text.to_string(),
                }],
            });
        }

        for (name, hex) in self.get_predefined_colors() {
            if name.to_lowercase().contains(&query_lower) || hex.to_lowercase().contains(&query_lower) {
                results.push(ResultItem {
                    id: format!("color-{}", name),
                    title: name.to_string(),
                    subtitle: Some(hex.to_string()),
                    icon: Some("color-select-symbolic".to_string()),
                    category: "Colors".to_string(),
                    score: 90,
                    actions: vec![Action {
                        label: format!("Copy {}", hex),
                        action_type: "copy-to-clipboard".to_string(),
                        value: hex.to_string(),
                    }],
                });
            }
        }

        results
    }
}
