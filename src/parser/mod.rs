pub mod json;
pub mod kv;
pub mod record;
pub mod text;

pub use record::Record;

pub fn parse_line(line: &str) -> Record {
    json::parse_json_line(line)
        .or_else(|| kv::parse_kv_line(line))
        .unwrap_or_else(|| text::parse_text_line(line))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_line_uses_json_before_key_value() {
        let record = parse_line(r#"{"level":"ERROR","message":"json message"}"#);

        assert_eq!(record.level.as_deref(), Some("ERROR"));
        assert_eq!(record.message.as_deref(), Some("json message"));
    }

    #[test]
    fn parse_line_uses_key_value_before_text_fallback() {
        let record = parse_line("level=INFO msg=ready");

        assert_eq!(record.level.as_deref(), Some("INFO"));
        assert_eq!(record.message.as_deref(), Some("ready"));
    }

    #[test]
    fn parse_line_falls_back_to_text() {
        let record = parse_line("WARN plain text event");

        assert_eq!(record.raw, "WARN plain text event");
        assert_eq!(record.message.as_deref(), Some("WARN plain text event"));
    }
}
