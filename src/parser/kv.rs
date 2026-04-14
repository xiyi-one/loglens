use super::Record;

pub fn parse_kv_line(line: &str) -> Option<Record> {
    let pairs: Vec<_> = line
        .split_whitespace()
        .filter_map(|part| part.split_once('='))
        .filter(|(key, value)| !key.is_empty() && !value.is_empty())
        .collect();

    if pairs.is_empty() {
        return None;
    }

    let mut record = Record::from_raw(line);

    for (key, value) in pairs {
        match key {
            "message" | "msg" => record.message = Some(value.to_string()),
            "level" => record.level = Some(value.to_string()),
            "timestamp" | "time" | "ts" => record.timestamp = Some(value.to_string()),
            _ => {
                record.fields.insert(key.to_string(), value.to_string());
            }
        }
    }

    Some(record)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kv_placeholder_preserves_raw_line() {
        let record = parse_kv_line("level=info msg=ready").expect("kv should parse");

        assert_eq!(record.raw, "level=info msg=ready");
        assert_eq!(record.level.as_deref(), Some("info"));
        assert_eq!(record.message.as_deref(), Some("ready"));
        assert!(record.fields.is_empty());
    }

    #[test]
    fn extracts_custom_fields() {
        let record =
            parse_kv_line("level=ERROR request_id=abc status_code=500").expect("kv should parse");

        assert_eq!(record.level.as_deref(), Some("ERROR"));
        assert!(!record.fields.contains_key("level"));
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
    fn returns_none_when_no_key_value_pairs_exist() {
        assert!(parse_kv_line("INFO service ready").is_none());
    }
}
