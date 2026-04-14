use serde_json::Value;

use super::Record;

pub fn parse_json_line(line: &str) -> Option<Record> {
    let Value::Object(object) = serde_json::from_str::<Value>(line).ok()? else {
        return None;
    };

    let mut record = Record::from_raw(line);

    for (key, value) in object {
        let Some(value) = json_value_to_field(value) else {
            continue;
        };

        match key.as_str() {
            "message" | "msg" => record.message = Some(value.clone()),
            "level" => record.level = Some(value.clone()),
            "timestamp" | "time" | "ts" => record.timestamp = Some(value.clone()),
            _ => {
                record.fields.insert(key, value);
            }
        }
    }

    Some(record)
}

fn json_value_to_field(value: Value) -> Option<String> {
    match value {
        Value::String(value) => Some(value),
        Value::Number(value) => Some(value.to_string()),
        Value::Bool(value) => Some(value.to_string()),
        Value::Null | Value::Array(_) | Value::Object(_) => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn json_placeholder_preserves_raw_line() {
        let record = parse_json_line(r#"{"level":"info"}"#).expect("json should parse");

        assert_eq!(record.raw, r#"{"level":"info"}"#);
        assert_eq!(record.level.as_deref(), Some("info"));
        assert!(record.fields.is_empty());
    }

    #[test]
    fn extracts_known_and_custom_json_fields() {
        let record = parse_json_line(
            r#"{"timestamp":"2026-04-13T10:00:00","level":"ERROR","message":"login failed","request_id":"abc","status_code":500}"#,
        )
        .expect("json should parse");

        assert_eq!(record.timestamp.as_deref(), Some("2026-04-13T10:00:00"));
        assert_eq!(record.level.as_deref(), Some("ERROR"));
        assert_eq!(record.message.as_deref(), Some("login failed"));
        assert!(!record.fields.contains_key("timestamp"));
        assert!(!record.fields.contains_key("level"));
        assert!(!record.fields.contains_key("message"));
        assert_eq!(
            record.fields.get("request_id").map(String::as_str),
            Some("abc")
        );
        assert_eq!(
            record.fields.get("status_code").map(String::as_str),
            Some("500")
        );
    }

    #[test]
    fn returns_none_for_non_json_line() {
        assert!(parse_json_line("level=INFO message=ready").is_none());
    }

    #[test]
    fn skips_nested_json_values_in_fields_map() {
        let record =
            parse_json_line(r#"{"level":"INFO","nested":{"id":"abc"},"tags":["a"]}"#).unwrap();

        assert_eq!(record.level.as_deref(), Some("INFO"));
        assert!(!record.fields.contains_key("nested"));
        assert!(!record.fields.contains_key("tags"));
    }
}
