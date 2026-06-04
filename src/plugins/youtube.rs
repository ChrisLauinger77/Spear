use crate::plugins::{Action, ResultItem, Plugin};

pub struct YouTubeSearchPlugin;

impl YouTubeSearchPlugin {
    pub fn new() -> Self {
        Self
    }
}

fn url_encode(input: &str) -> String {
    let mut encoded = String::new();
    for b in input.bytes() {
        match b {
            b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                encoded.push(b as char);
            }
            b' ' => {
                encoded.push('+');
            }
            _ => {
                encoded.push_str(&format!("%{:02X}", b));
            }
        }
    }
    encoded
}

impl Plugin for YouTubeSearchPlugin {
    fn id(&self) -> &str {
        "youtube"
    }

    fn name(&self) -> &str {
        "YouTube Search"
    }

    fn query(&self, query_text: &str, settings: &crate::settings::AppSettings) -> Vec<ResultItem> {
        let query_text = query_text.trim();
        if query_text.is_empty() {
            return Vec::new();
        }

        let encoded = url_encode(query_text);
        let base_url = if settings.youtube_url.is_empty() {
            "https://youtube.com/results?search_query=".to_string()
        } else {
            settings.youtube_url.clone()
        };

        vec![ResultItem {
            id: "youtube-search".to_string(),
            title: format!("Search YouTube for '{}'", query_text),
            subtitle: Some("Search and watch videos on YouTube".to_string()),
            icon: Some("youtube.svg".to_string()),
            category: "YouTube Search".to_string(),
            score: 35,
            actions: vec![Action {
                label: "Search YouTube".to_string(),
                action_type: "open-url".to_string(),
                value: format!("{}{}", base_url, encoded),
            }],
        }]
    }
}
