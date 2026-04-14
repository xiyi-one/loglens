use serde_json::json;

use crate::dsl::{Condition, Operator, Query};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Translation {
    pub query: Query,
    pub explanation: String,
}

pub fn translate_heuristic(_input: &str) -> Translation {
    let input = _input.trim();
    let normalized = input.to_lowercase();
    let mut query = Query::empty();
    let mut explanation = Vec::new();

    for (level, aliases) in [
        ("ERROR", ["error", "errors"]),
        ("WARN", ["warn", "warning"]),
        ("INFO", ["info", "information"]),
    ] {
        if aliases.iter().any(|alias| has_word(&normalized, alias)) {
            query.filters.must.push(Condition {
                field: "level".to_string(),
                operator: Operator::Equals,
                value: Some(json!(level)),
            });
            explanation.push(format!("matched level keyword `{}`", level.to_lowercase()));
        }
    }

    for phrase in ["login failed", "payment failed", "timeout", "timed out"] {
        if normalized.contains(phrase) {
            query.filters.must.push(Condition {
                field: "raw".to_string(),
                operator: Operator::Contains,
                value: Some(json!(phrase)),
            });
            explanation.push(format!("matched phrase `{phrase}`"));
        }
    }

    if mentions_private_ip(&normalized) {
        query.filters.must.push(Condition {
            field: "ip".to_string(),
            operator: Operator::IsPrivateIp,
            value: None,
        });
        explanation.push("matched private IP intent".to_string());
    } else if mentions_public_ip(&normalized) {
        query.filters.must.push(Condition {
            field: "ip".to_string(),
            operator: Operator::IsPublicIp,
            value: None,
        });
        explanation.push("matched public IP intent".to_string());
    }

    if query.filters.must.is_empty() && !input.is_empty() {
        query.filters.must.push(Condition {
            field: "raw".to_string(),
            operator: Operator::Contains,
            value: Some(json!(input)),
        });
        explanation.push("used broad raw-line contains query".to_string());
    }

    Translation {
        query,
        explanation: if explanation.is_empty() {
            "no heuristic rules applied".to_string()
        } else {
            explanation.join("; ")
        },
    }
}

fn has_word(input: &str, needle: &str) -> bool {
    input
        .split(|character: char| !character.is_ascii_alphanumeric())
        .any(|word| word == needle)
}

fn mentions_private_ip(input: &str) -> bool {
    input.contains("private ip")
        || input.contains("private ips")
        || input.contains("internal ip")
        || input.contains("internal ips")
}

fn mentions_public_ip(input: &str) -> bool {
    input.contains("public ip")
        || input.contains("public ips")
        || input.contains("external ip")
        || input.contains("external ips")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dsl::validate_query;

    #[test]
    fn returns_valid_empty_query_for_placeholder() {
        let translation = translate_heuristic("show errors");

        assert_eq!(translation.query.filters.must.len(), 1);
        assert!(!translation.explanation.is_empty());
        assert_eq!(validate_query(&translation.query), Ok(()));
    }

    #[test]
    fn translates_level_keywords() {
        let translation = translate_heuristic("show warn logs");

        assert_eq!(
            translation.query.filters.must,
            vec![Condition {
                field: "level".to_string(),
                operator: Operator::Equals,
                value: Some(json!("WARN")),
            }]
        );
        assert_eq!(validate_query(&translation.query), Ok(()));
    }

    #[test]
    fn translates_common_phrases() {
        let translation = translate_heuristic("payment failed and timed out");

        assert!(translation.query.filters.must.contains(&Condition {
            field: "raw".to_string(),
            operator: Operator::Contains,
            value: Some(json!("payment failed")),
        }));
        assert!(translation.query.filters.must.contains(&Condition {
            field: "raw".to_string(),
            operator: Operator::Contains,
            value: Some(json!("timed out")),
        }));
        assert_eq!(validate_query(&translation.query), Ok(()));
    }

    #[test]
    fn translates_public_ip_intent() {
        let translation = translate_heuristic("login failed from public ip");

        assert!(translation.query.filters.must.contains(&Condition {
            field: "ip".to_string(),
            operator: Operator::IsPublicIp,
            value: None,
        }));
        assert_eq!(validate_query(&translation.query), Ok(()));
    }

    #[test]
    fn translates_private_ip_intent() {
        let translation = translate_heuristic("show errors from private ips");

        assert!(translation.query.filters.must.contains(&Condition {
            field: "ip".to_string(),
            operator: Operator::IsPrivateIp,
            value: None,
        }));
        assert_eq!(validate_query(&translation.query), Ok(()));
    }

    #[test]
    fn falls_back_to_broad_raw_query() {
        let translation = translate_heuristic("cache warmup finished");

        assert_eq!(
            translation.query.filters.must,
            vec![Condition {
                field: "raw".to_string(),
                operator: Operator::Contains,
                value: Some(json!("cache warmup finished")),
            }]
        );
        assert_eq!(validate_query(&translation.query), Ok(()));
    }
}
