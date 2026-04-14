use chrono::NaiveDateTime;
use regex::Regex;
use serde_json::Value;
use thiserror::Error;

use super::condition::{Condition, Operator};
use super::schema::{Query, TimeRange};

#[derive(Debug, Clone, Error, PartialEq, Eq)]
pub enum ValidationError {
    #[error("unsupported DSL version {version}")]
    UnsupportedVersion { version: u32 },
    #[error("time_range must include start or end")]
    EmptyTimeRange,
    #[error("time_range.{field} must be an ISO8601 datetime")]
    InvalidTimeRangeValue { field: &'static str, value: String },
    #[error("time_range.start must be before or equal to time_range.end")]
    InvalidTimeRangeOrder,
    #[error("level_in must not be empty")]
    EmptyLevelIn,
    #[error("condition field must not be empty")]
    EmptyField,
    #[error("{operator} requires a value")]
    MissingValue { operator: &'static str },
    #[error("{operator} does not accept a value")]
    UnexpectedValue { operator: &'static str },
    #[error("{operator} value must be {expected}")]
    InvalidValue {
        operator: &'static str,
        expected: &'static str,
    },
    #[error("regex value is invalid: {message}")]
    InvalidRegex { message: String },
}

pub fn validate_query(query: &Query) -> Result<(), ValidationError> {
    if query.version != 1 {
        return Err(ValidationError::UnsupportedVersion {
            version: query.version,
        });
    }

    if let Some(time_range) = &query.filters.time_range {
        validate_time_range(time_range)?;
    }

    if matches!(query.filters.level_in.as_ref(), Some(levels) if levels.is_empty()) {
        return Err(ValidationError::EmptyLevelIn);
    }

    for condition in query
        .filters
        .must
        .iter()
        .chain(query.filters.must_not.iter())
        .chain(query.filters.should.iter())
    {
        validate_condition(condition)?;
    }

    if matches!(query.options.limit, Some(0)) {
        return Err(ValidationError::InvalidValue {
            operator: "limit",
            expected: "a positive integer",
        });
    }

    Ok(())
}

fn validate_time_range(time_range: &TimeRange) -> Result<(), ValidationError> {
    if time_range.start.is_none() && time_range.end.is_none() {
        return Err(ValidationError::EmptyTimeRange);
    }

    let start = time_range
        .start
        .as_deref()
        .map(|value| parse_datetime("start", value))
        .transpose()?;
    let end = time_range
        .end
        .as_deref()
        .map(|value| parse_datetime("end", value))
        .transpose()?;

    if let (Some(start), Some(end)) = (start, end) {
        if start > end {
            return Err(ValidationError::InvalidTimeRangeOrder);
        }
    }

    Ok(())
}

fn parse_datetime(field: &'static str, value: &str) -> Result<NaiveDateTime, ValidationError> {
    NaiveDateTime::parse_from_str(value, "%Y-%m-%dT%H:%M:%S").map_err(|_| {
        ValidationError::InvalidTimeRangeValue {
            field,
            value: value.to_string(),
        }
    })
}

fn validate_condition(condition: &Condition) -> Result<(), ValidationError> {
    if condition.field.trim().is_empty() {
        return Err(ValidationError::EmptyField);
    }

    match condition.operator {
        Operator::Contains => require_string("contains", condition.value.as_ref()),
        Operator::ContainsAll => {
            require_non_empty_string_array("contains_all", condition.value.as_ref())
        }
        Operator::ContainsAny => {
            require_non_empty_string_array("contains_any", condition.value.as_ref())
        }
        Operator::Regex => validate_regex(condition.value.as_ref()),
        Operator::Equals => require_scalar("equals", condition.value.as_ref()),
        Operator::NotEquals => require_scalar("not_equals", condition.value.as_ref()),
        Operator::Exists => reject_value("exists", condition.value.as_ref()),
        Operator::In => require_non_empty_array("in", condition.value.as_ref()),
        Operator::Gte => require_string_or_number("gte", condition.value.as_ref()),
        Operator::Lte => require_string_or_number("lte", condition.value.as_ref()),
        Operator::IsPrivateIp => validate_ip_operator("is_private_ip", condition),
        Operator::IsPublicIp => validate_ip_operator("is_public_ip", condition),
    }
}

fn require_string(operator: &'static str, value: Option<&Value>) -> Result<(), ValidationError> {
    match value {
        Some(Value::String(_)) => Ok(()),
        Some(_) => Err(ValidationError::InvalidValue {
            operator,
            expected: "a string",
        }),
        None => Err(ValidationError::MissingValue { operator }),
    }
}

fn require_non_empty_string_array(
    operator: &'static str,
    value: Option<&Value>,
) -> Result<(), ValidationError> {
    match value {
        Some(Value::Array(values)) if !values.is_empty() && values.iter().all(Value::is_string) => {
            Ok(())
        }
        Some(_) => Err(ValidationError::InvalidValue {
            operator,
            expected: "a non-empty array of strings",
        }),
        None => Err(ValidationError::MissingValue { operator }),
    }
}

fn require_scalar(operator: &'static str, value: Option<&Value>) -> Result<(), ValidationError> {
    match value {
        Some(Value::String(_) | Value::Number(_) | Value::Bool(_)) => Ok(()),
        Some(_) => Err(ValidationError::InvalidValue {
            operator,
            expected: "a string, number, or boolean",
        }),
        None => Err(ValidationError::MissingValue { operator }),
    }
}

fn require_non_empty_array(
    operator: &'static str,
    value: Option<&Value>,
) -> Result<(), ValidationError> {
    match value {
        Some(Value::Array(values)) if !values.is_empty() => Ok(()),
        Some(_) => Err(ValidationError::InvalidValue {
            operator,
            expected: "a non-empty array",
        }),
        None => Err(ValidationError::MissingValue { operator }),
    }
}

fn require_string_or_number(
    operator: &'static str,
    value: Option<&Value>,
) -> Result<(), ValidationError> {
    match value {
        Some(Value::String(_) | Value::Number(_)) => Ok(()),
        Some(_) => Err(ValidationError::InvalidValue {
            operator,
            expected: "a string or number",
        }),
        None => Err(ValidationError::MissingValue { operator }),
    }
}

fn reject_value(operator: &'static str, value: Option<&Value>) -> Result<(), ValidationError> {
    match value {
        Some(_) => Err(ValidationError::UnexpectedValue { operator }),
        None => Ok(()),
    }
}

fn validate_regex(value: Option<&Value>) -> Result<(), ValidationError> {
    let pattern = match value {
        Some(Value::String(pattern)) => pattern,
        Some(_) => {
            return Err(ValidationError::InvalidValue {
                operator: "regex",
                expected: "a string",
            });
        }
        None => return Err(ValidationError::MissingValue { operator: "regex" }),
    };

    Regex::new(pattern)
        .map(|_| ())
        .map_err(|error| ValidationError::InvalidRegex {
            message: error.to_string(),
        })
}

fn validate_ip_operator(
    operator: &'static str,
    condition: &Condition,
) -> Result<(), ValidationError> {
    if condition.field != "ip" {
        return Err(ValidationError::InvalidValue {
            operator,
            expected: "field ip",
        });
    }

    reject_value(operator, condition.value.as_ref())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn accepts_empty_query_placeholder() {
        assert_eq!(validate_query(&Query::empty()), Ok(()));
    }

    #[test]
    fn validates_example_query() {
        let query = Query::from_json(
            r#"{
                "version": 1,
                "filters": {
                    "time_range": {
                        "start": "2026-04-13T10:00:00",
                        "end": "2026-04-13T11:00:00"
                    },
                    "level_in": ["ERROR", "WARN"],
                    "must": [
                        { "field": "message", "op": "contains", "value": "login failed" },
                        { "field": "status_code", "op": "gte", "value": 500 }
                    ],
                    "must_not": [
                        { "field": "ip", "op": "is_private_ip" }
                    ],
                    "should": [
                        { "field": "raw", "op": "contains_any", "value": ["auth", "login"] }
                    ]
                },
                "options": {
                    "limit": 100,
                    "follow": false,
                    "case_sensitive": false
                }
            }"#,
        )
        .expect("query should parse");

        assert_eq!(validate_query(&query), Ok(()));
    }

    #[test]
    fn rejects_unsupported_version() {
        let mut query = Query::empty();
        query.version = 2;

        assert_eq!(
            validate_query(&query),
            Err(ValidationError::UnsupportedVersion { version: 2 })
        );
    }

    #[test]
    fn rejects_invalid_time_range_order() {
        let query = Query::from_json(
            r#"{
                "version": 1,
                "filters": {
                    "time_range": {
                        "start": "2026-04-13T11:00:00",
                        "end": "2026-04-13T10:00:00"
                    }
                }
            }"#,
        )
        .expect("query should parse");

        assert_eq!(
            validate_query(&query),
            Err(ValidationError::InvalidTimeRangeOrder)
        );
    }

    #[test]
    fn rejects_empty_level_filter() {
        let mut query = Query::empty();
        query.filters.level_in = Some(Vec::new());

        assert_eq!(validate_query(&query), Err(ValidationError::EmptyLevelIn));
    }

    #[test]
    fn rejects_missing_value_for_contains() {
        let mut query = Query::empty();
        query.filters.must.push(Condition {
            field: "message".to_string(),
            operator: Operator::Contains,
            value: None,
        });

        assert_eq!(
            validate_query(&query),
            Err(ValidationError::MissingValue {
                operator: "contains"
            })
        );
    }

    #[test]
    fn rejects_empty_array_for_contains_any() {
        let mut query = Query::empty();
        query.filters.must.push(Condition {
            field: "raw".to_string(),
            operator: Operator::ContainsAny,
            value: Some(json!([])),
        });

        assert_eq!(
            validate_query(&query),
            Err(ValidationError::InvalidValue {
                operator: "contains_any",
                expected: "a non-empty array of strings",
            })
        );
    }

    #[test]
    fn rejects_value_for_exists() {
        let mut query = Query::empty();
        query.filters.must.push(Condition {
            field: "trace_id".to_string(),
            operator: Operator::Exists,
            value: Some(json!(true)),
        });

        assert_eq!(
            validate_query(&query),
            Err(ValidationError::UnexpectedValue { operator: "exists" })
        );
    }

    #[test]
    fn rejects_ip_operator_on_non_ip_field() {
        let mut query = Query::empty();
        query.filters.must.push(Condition {
            field: "message".to_string(),
            operator: Operator::IsPublicIp,
            value: None,
        });

        assert_eq!(
            validate_query(&query),
            Err(ValidationError::InvalidValue {
                operator: "is_public_ip",
                expected: "field ip",
            })
        );
    }

    #[test]
    fn rejects_invalid_regex() {
        let mut query = Query::empty();
        query.filters.must.push(Condition {
            field: "message".to_string(),
            operator: Operator::Regex,
            value: Some(json!("[")),
        });

        assert!(matches!(
            validate_query(&query),
            Err(ValidationError::InvalidRegex { .. })
        ));
    }

    #[test]
    fn rejects_zero_limit() {
        let mut query = Query::empty();
        query.options.limit = Some(0);

        assert_eq!(
            validate_query(&query),
            Err(ValidationError::InvalidValue {
                operator: "limit",
                expected: "a positive integer",
            })
        );
    }
}
