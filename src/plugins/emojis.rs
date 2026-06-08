use crate::plugins::{Action, ResultItem, Plugin};

pub struct EmojiPlugin;

impl EmojiPlugin {
    pub fn new() -> Self {
        Self
    }

    fn get_emojis(&self) -> Vec<(&str, &str, &str)> {
        vec![
            ("Grinning Face", "😀", "smile, happy"),
            ("Smiling Face with Heart-Eyes", "😍", "love, heart"),
            ("Laughing", "😂", "lol, haha"),
            ("Thumbs Up", "👍", "yes, ok"),
            ("Sparkles", "✨", "magic, shiny"),
            ("Rocket", "🚀", "fast, start"),
            ("Fire", "🔥", "hot, lit"),
            ("Party Popper", "🎉", "celebrate, party"),
            ("Heart", "❤️", "love"),
            ("Check Mark", "✅", "done, correct"),
        ]
    }
}

impl Plugin for EmojiPlugin {
    fn id(&self) -> &str {
        "emojis"
    }

    fn name(&self) -> &str {
        "Emoji Picker"
    }

    fn mode_keyword(&self) -> Option<&str> {
        Some("emoji")
    }

    fn is_mode_only(&self) -> bool {
        true
    }

    fn query(&self, query_text: &str, _settings: &crate::settings::AppSettings) -> Vec<ResultItem> {
        let query_lower = query_text.to_lowercase();
        
        self.get_emojis().into_iter()
            .filter(|(name, _, tags)| name.to_lowercase().contains(&query_lower) || tags.to_lowercase().contains(&query_lower))
            .map(|(name, emoji, _)| {
                ResultItem {
                    id: format!("emoji-{}", name),
                    title: format!("{} {}", emoji, name),
                    subtitle: Some("Emoji".to_string()),
                    icon: None,
                    category: "Emojis".to_string(),
                    score: 85,
                    actions: vec![Action {
                        label: "Copy Emoji".to_string(),
                        action_type: "copy-to-clipboard".to_string(),
                        value: emoji.to_string(),
                    }],
                }
            })
            .collect()
    }
}
