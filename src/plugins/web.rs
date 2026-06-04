use crate::plugins::{Action, ResultItem, Plugin};

pub struct WebSearchPlugin;

impl WebSearchPlugin {
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

impl Plugin for WebSearchPlugin {
    fn id(&self) -> &str {
        "web"
    }

    fn name(&self) -> &str {
        "Web Search"
    }

    fn query(&self, query_text: &str, settings: &crate::settings::AppSettings) -> Vec<ResultItem> {
        let query_text = query_text.trim();
        if query_text.is_empty() {
            return Vec::new();
        }

        let encoded = url_encode(query_text);
        let custom_url = if settings.web_custom_url.is_empty() {
            "https://google.com/search?q=".to_string()
        } else {
            settings.web_custom_url.clone()
        };

        vec![
            ResultItem {
                id: "web-custom".to_string(),
                title: format!("Search Web for '{}'", query_text),
                subtitle: Some(format!("Search using configured engine URL")),
                icon: Some("google.svg".to_string()),
                category: "Web Search".to_string(),
                score: 50,
                actions: vec![Action {
                    label: "Search Web".to_string(),
                    action_type: "open-url".to_string(),
                    value: format!("{}{}", custom_url, encoded),
                }],
            },
            ResultItem {
                id: "web-github".to_string(),
                title: format!("Search GitHub for '{}'", query_text),
                subtitle: Some("Search repositories and code on GitHub".to_string()),
                icon: Some("github.svg".to_string()),
                category: "Web Search".to_string(),
                score: 45,
                actions: vec![Action {
                    label: "Search GitHub".to_string(),
                    action_type: "open-url".to_string(),
                    value: format!("https://github.com/search?q={}", encoded),
                }],
            },
            ResultItem {
                id: "web-duckduckgo".to_string(),
                title: format!("Search DuckDuckGo for '{}'", query_text),
                subtitle: Some("Search privately with DuckDuckGo".to_string()),
                icon: Some("duckduckgo.svg".to_string()),
                category: "Web Search".to_string(),
                score: 40,
                actions: vec![Action {
                    label: "Search DuckDuckGo".to_string(),
                    action_type: "open-url".to_string(),
                    value: format!("https://duckduckgo.com/?q={}", encoded),
                }],
            },
        ]
    }
}
