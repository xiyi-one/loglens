use serde::{Deserialize, Serialize};

use super::condition::Condition;

const DSL_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Query {
    pub version: u32,
    pub filters: Filters,
    #[serde(default)]
    pub options: Options,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Filters {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub time_range: Option<TimeRange>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub level_in: Option<Vec<String>>,
    #[serde(default)]
    pub must: Vec<Condition>,
    #[serde(default)]
    pub must_not: Vec<Condition>,
    #[serde(default)]
    pub should: Vec<Condition>,
}

impl Filters {
    pub fn is_empty(&self) -> bool {
        self.time_range.is_none()
            && self.level_in.as_ref().is_none_or(Vec::is_empty)
            && self.must.is_empty()
            && self.must_not.is_empty()
            && self.should.is_empty()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TimeRange {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub start: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub end: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Options {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub limit: Option<usize>,
    #[serde(default)]
    pub follow: bool,
    #[serde(default)]
    pub case_sensitive: bool,
}

impl Query {
    pub fn empty() -> Self {
        Self {
            version: DSL_VERSION,
            filters: Filters::default(),
            options: Options::default(),
        }
    }

    pub fn from_json(input: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(input)
    }
}

impl Default for Options {
    fn default() -> Self {
        Self {
            limit: Some(100),
            follow: false,
            case_sensitive: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creates_empty_query() {
        let query = Query::empty();

        assert_eq!(query.version, 1);
        assert!(query.filters.must.is_empty());
        assert_eq!(query.options, Options::default());
    }

    #[test]
    fn parses_json_with_default_options() {
        let query = Query::from_json(
            r#"{
                "version": 1,
                "filters": {
                    "must": [
                        { "field": "message", "op": "contains", "value": "timeout" }
                    ]
                }
            }"#,
        )
        .expect("query should parse");

        assert_eq!(query.version, 1);
        assert_eq!(query.options, Options::default());
        assert_eq!(query.filters.must.len(), 1);
    }
}
