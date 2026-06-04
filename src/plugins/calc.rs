use crate::plugins::{Action, ResultItem, Plugin};
use crate::utils::evaluate_math;

pub struct CalculatorPlugin;

impl CalculatorPlugin {
    pub fn new() -> Self {
        Self
    }
}

impl Plugin for CalculatorPlugin {
    fn id(&self) -> &str {
        "calc"
    }

    fn name(&self) -> &str {
        "Calculator"
    }

    fn query(&self, query_text: &str, _settings: &crate::settings::AppSettings) -> Vec<ResultItem> {
        let query_text = query_text.trim();
        if query_text.is_empty() {
            return Vec::new();
        }

        // Quick heuristic: does the query contain digits or start with math constants/functions?
        let has_digits = query_text.chars().any(|c| c.is_ascii_digit());
        let starts_with_math_func = ["sin", "cos", "tan", "sqrt", "abs", "pi", "e"]
            .iter()
            .any(|&f| query_text.to_lowercase().starts_with(f));

        if !has_digits && !starts_with_math_func {
            return Vec::new();
        }

        // Evaluate expression
        if let Some(res) = evaluate_math(query_text) {
            // Check for NaN or Inf
            let formatted = if res.is_nan() || res.is_infinite() {
                res.to_string()
            } else {
                // Format float with up to 6 decimal places, removing trailing zeros
                let s = format!("{:.6}", res);
                let trimmed = s.trim_end_matches('0').trim_end_matches('.');
                trimmed.to_string()
            };

            vec![ResultItem {
                id: format!("calc-{}", formatted),
                title: formatted.clone(),
                subtitle: Some(format!("Result for: {}", query_text)),
                icon: Some("calculator.svg".to_string()),
                category: "Calculator".to_string(),
                score: 110, // Higher score so calculator results sit at the top
                actions: vec![Action {
                    label: "Copy to Clipboard".to_string(),
                    action_type: "copy-to-clipboard".to_string(),
                    value: formatted,
                }],
            }]
        } else {
            Vec::new()
        }
    }
}
