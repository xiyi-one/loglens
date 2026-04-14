use crate::dsl::{Condition, Operator, Query};
use crate::engine::Match;

use clap::ValueEnum;
use serde_json::Value;

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum OutputFormat {
    Text,
    Json,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderedLine {
    pub line: String,
}

pub fn render_match(matched: &Match) -> RenderedLine {
    RenderedLine {
        line: matched.record.raw.clone(),
    }
}

pub fn format_match(
    matched: &Match,
    query: &Query,
    output_format: OutputFormat,
) -> Result<String, serde_json::Error> {
    match output_format {
        OutputFormat::Text => Ok(format_text_match(matched, query)),
        OutputFormat::Json => serde_json::to_string(&matched.record),
    }
}

pub fn format_query_plan(query: &Query) -> String {
    let mut lines = vec![
        "Query plan".to_string(),
        format!("version: {}", query.version),
        format!(
            "options: limit={}, follow={}, case_sensitive={}",
            query
                .options
                .limit
                .map(|limit| limit.to_string())
                .unwrap_or_else(|| "none".to_string()),
            query.options.follow,
            query.options.case_sensitive
        ),
    ];

    if let Some(levels) = &query.filters.level_in {
        lines.push(format!("level_in: {}", levels.join(", ")));
    }

    if let Some(time_range) = &query.filters.time_range {
        lines.push(format!(
            "time_range: start={}, end={}",
            time_range.start.as_deref().unwrap_or("*"),
            time_range.end.as_deref().unwrap_or("*")
        ));
    }

    push_conditions(&mut lines, "must", &query.filters.must);
    push_conditions(&mut lines, "must_not", &query.filters.must_not);
    push_conditions(&mut lines, "should", &query.filters.should);

    lines.join("\n")
}

fn format_text_match(matched: &Match, query: &Query) -> String {
    let highlighted_raw = highlight_terms(
        &matched.record.raw,
        &highlight_terms_from_query(query),
        query.options.case_sensitive,
    );
    let mut parts = Vec::new();

    if let Some(level) = &matched.record.level {
        parts.push(format!("level={level}"));
    }

    if let Some(message) = &matched.record.message {
        if message != &matched.record.raw {
            let highlighted_message = highlight_terms(
                message,
                &highlight_terms_from_query(query),
                query.options.case_sensitive,
            );
            parts.push(format!("message={highlighted_message}"));
        }
    }

    if parts.is_empty() {
        highlighted_raw
    } else {
        format!("{} | {}", parts.join(" "), highlighted_raw)
    }
}

fn push_conditions(lines: &mut Vec<String>, name: &str, conditions: &[Condition]) {
    if conditions.is_empty() {
        lines.push(format!("{name}: none"));
        return;
    }

    lines.push(format!("{name}:"));
    for condition in conditions {
        lines.push(format!(
            "  - field={} op={} value={}",
            condition.field,
            operator_name(&condition.operator),
            condition_value(&condition.value)
        ));
    }
}

fn condition_value(value: &Option<Value>) -> String {
    value
        .as_ref()
        .map(Value::to_string)
        .unwrap_or_else(|| "none".to_string())
}

fn highlight_terms_from_query(query: &Query) -> Vec<String> {
    let mut terms = Vec::new();

    for condition in query.filters.must.iter().chain(query.filters.should.iter()) {
        match condition.operator {
            Operator::Contains | Operator::Equals => {
                if let Some(term) = condition.value.as_ref().and_then(Value::as_str) {
                    terms.push(term.to_string());
                }
            }
            Operator::ContainsAll | Operator::ContainsAny => {
                if let Some(values) = condition.value.as_ref().and_then(Value::as_array) {
                    terms.extend(
                        values
                            .iter()
                            .filter_map(Value::as_str)
                            .map(ToString::to_string),
                    );
                }
            }
            _ => {}
        }
    }

    terms
}

fn highlight_terms(input: &str, terms: &[String], case_sensitive: bool) -> String {
    let mut output = input.to_string();

    for term in terms {
        if term.is_empty() {
            continue;
        }

        output = highlight_term(&output, term, case_sensitive);
    }

    output
}

fn highlight_term(input: &str, term: &str, case_sensitive: bool) -> String {
    let search_input = if case_sensitive {
        input.to_string()
    } else {
        input.to_ascii_lowercase()
    };
    let search_term = if case_sensitive {
        term.to_string()
    } else {
        term.to_ascii_lowercase()
    };

    let mut output = String::new();
    let mut cursor = 0;

    while let Some(relative_start) = search_input[cursor..].find(&search_term) {
        let start = cursor + relative_start;
        let end = start + search_term.len();
        output.push_str(&input[cursor..start]);
        output.push_str("\x1b[1;33m");
        output.push_str(&input[start..end]);
        output.push_str("\x1b[0m");
        cursor = end;
    }

    output.push_str(&input[cursor..]);
    output
}

fn operator_name(operator: &Operator) -> &'static str {
    match operator {
        Operator::Contains => "contains",
        Operator::ContainsAll => "contains_all",
        Operator::ContainsAny => "contains_any",
        Operator::Regex => "regex",
        Operator::Equals => "equals",
        Operator::NotEquals => "not_equals",
        Operator::Exists => "exists",
        Operator::In => "in",
        Operator::Gte => "gte",
        Operator::Lte => "lte",
        Operator::IsPrivateIp => "is_private_ip",
        Operator::IsPublicIp => "is_public_ip",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dsl::{Condition, Operator, Query};
    use crate::parser::Record;
    use serde_json::json;

    #[test]
    fn renders_raw_line() {
        let matched = Match {
            record: Record::from_raw("ERROR request failed"),
        };

        assert_eq!(render_match(&matched).line, "ERROR request failed");
    }

    #[test]
    fn renders_json_record() {
        let matched = Match {
            record: Record::from_raw("ERROR request failed"),
        };

        let rendered = format_match(&matched, &Query::empty(), OutputFormat::Json)
            .expect("json should render");

        assert!(rendered.contains(r#""raw":"ERROR request failed""#));
    }

    #[test]
    fn text_output_includes_metadata_and_highlights_matches() {
        let mut record = Record::from_raw("ERROR login failed for user=alice");
        record.level = Some("ERROR".to_string());
        record.message = Some("login failed".to_string());
        let matched = Match { record };
        let mut query = Query::empty();
        query.filters.must.push(Condition {
            field: "raw".to_string(),
            operator: Operator::Contains,
            value: Some(json!("login failed")),
        });

        let rendered =
            format_match(&matched, &query, OutputFormat::Text).expect("text should render");

        assert_eq!(
            rendered,
            "level=ERROR message=\u{1b}[1;33mlogin failed\u{1b}[0m | ERROR \u{1b}[1;33mlogin failed\u{1b}[0m for user=alice"
        );
    }

    #[test]
    fn formats_human_readable_query_plan() {
        let mut query = Query::empty();
        query.filters.must.push(Condition {
            field: "raw".to_string(),
            operator: Operator::Contains,
            value: Some(json!("timeout")),
        });

        let plan = format_query_plan(&query);

        assert!(plan.contains("Query plan"));
        assert!(plan.contains("version: 1"));
        assert!(plan.contains("field=raw op=contains value=\"timeout\""));
    }
}
