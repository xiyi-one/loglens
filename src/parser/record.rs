use std::collections::HashMap;

use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Record {
    pub raw: String,
    pub message: Option<String>,
    pub level: Option<String>,
    pub timestamp: Option<String>,
    pub fields: HashMap<String, String>,
}

impl Record {
    pub fn from_raw(raw: impl Into<String>) -> Self {
        Self {
            raw: raw.into(),
            message: None,
            level: None,
            timestamp: None,
            fields: HashMap::new(),
        }
    }

    pub fn field_value(&self, field: &str) -> Option<&str> {
        match field {
            "raw" => Some(&self.raw),
            "message" => self
                .message
                .as_deref()
                .or_else(|| self.fields.get(field).map(String::as_str)),
            "level" => self
                .level
                .as_deref()
                .or_else(|| self.fields.get(field).map(String::as_str)),
            "timestamp" => self
                .timestamp
                .as_deref()
                .or_else(|| self.fields.get(field).map(String::as_str)),
            _ => self.fields.get(field).map(String::as_str),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn raw_field_always_resolves_to_original_line() {
        let record = Record::from_raw("INFO service started");

        assert_eq!(record.field_value("raw"), Some("INFO service started"));
    }

    #[test]
    fn known_fields_prefer_structured_values() {
        let mut record = Record::from_raw("level=INFO message=ready");
        record.message = Some("ready".to_string());
        record
            .fields
            .insert("message".to_string(), "fallback".to_string());

        assert_eq!(record.field_value("message"), Some("ready"));
    }
}
