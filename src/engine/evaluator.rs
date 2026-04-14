use serde_json::Value;

use crate::dsl::{Condition, Operator, Query};
use crate::parser::Record;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Match {
    pub record: Record,
}

pub fn matches(query: &Query, record: Record) -> Option<Match> {
    evaluate_query(query, &record).then_some(Match { record })
}

pub fn evaluate_query(query: &Query, record: &Record) -> bool {
    query
        .filters
        .must
        .iter()
        .all(|condition| evaluate_condition(condition, record, query.options.case_sensitive))
        && query
            .filters
            .must_not
            .iter()
            .all(|condition| !evaluate_condition(condition, record, query.options.case_sensitive))
        && (query.filters.should.is_empty()
            || query.filters.should.iter().any(|condition| {
                evaluate_condition(condition, record, query.options.case_sensitive)
            }))
}

fn evaluate_condition(condition: &Condition, record: &Record, case_sensitive: bool) -> bool {
    let Some(field_value) = record.field_value(&condition.field) else {
        return false;
    };

    match condition.operator {
        Operator::Contains => condition
            .value
            .as_ref()
            .and_then(Value::as_str)
            .is_some_and(|term| contains(field_value, term, case_sensitive)),
        Operator::ContainsAll => condition
            .value
            .as_ref()
            .and_then(Value::as_array)
            .is_some_and(|terms| match string_terms(terms) {
                Some(terms) => terms
                    .into_iter()
                    .all(|term| contains(field_value, term, case_sensitive)),
                None => false,
            }),
        Operator::ContainsAny => condition
            .value
            .as_ref()
            .and_then(Value::as_array)
            .is_some_and(|terms| match string_terms(terms) {
                Some(terms) => terms
                    .into_iter()
                    .any(|term| contains(field_value, term, case_sensitive)),
                None => false,
            }),
        Operator::Equals => condition
            .value
            .as_ref()
            .is_some_and(|value| equals(field_value, value, case_sensitive)),
        _ => false,
    }
}

fn string_terms(terms: &[Value]) -> Option<Vec<&str>> {
    if terms.is_empty() || !terms.iter().all(Value::is_string) {
        return None;
    }

    Some(terms.iter().filter_map(Value::as_str).collect())
}

fn contains(field_value: &str, term: &str, case_sensitive: bool) -> bool {
    if case_sensitive {
        field_value.contains(term)
    } else {
        field_value.to_lowercase().contains(&term.to_lowercase())
    }
}

fn equals(field_value: &str, value: &Value, case_sensitive: bool) -> bool {
    match value {
        Value::String(expected) if case_sensitive => field_value == expected,
        Value::String(expected) => field_value.eq_ignore_ascii_case(expected),
        Value::Number(expected) => field_value == expected.to_string(),
        Value::Bool(expected) => field_value == expected.to_string(),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn condition(operator: Operator, value: Value) -> Condition {
        Condition {
            field: "raw".to_string(),
            operator,
            value: Some(value),
        }
    }

    fn field_condition(field: &str, operator: Operator, value: Value) -> Condition {
        Condition {
            field: field.to_string(),
            operator,
            value: Some(value),
        }
    }

    #[test]
    fn preserves_record_in_match() {
        let record = Record::from_raw("INFO service started");
        let matched = matches(&Query::empty(), record.clone()).expect("record should match");

        assert_eq!(matched.record, record);
    }

    #[test]
    fn empty_query_matches_unconditionally() {
        assert!(evaluate_query(
            &Query::empty(),
            &Record::from_raw("any raw log line")
        ));
        assert!(evaluate_query(&Query::empty(), &Record::from_raw("")));
    }

    #[test]
    fn matches_contains_condition() {
        let mut query = Query::empty();
        query
            .filters
            .must
            .push(condition(Operator::Contains, json!("service started")));

        assert!(evaluate_query(
            &query,
            &Record::from_raw("INFO service started")
        ));
        assert!(!evaluate_query(
            &query,
            &Record::from_raw("INFO service stopped")
        ));
    }

    #[test]
    fn matches_contains_all_condition() {
        let mut query = Query::empty();
        query
            .filters
            .must
            .push(condition(Operator::ContainsAll, json!(["INFO", "started"])));

        assert!(evaluate_query(
            &query,
            &Record::from_raw("INFO service started")
        ));
        assert!(!evaluate_query(
            &query,
            &Record::from_raw("INFO service stopped")
        ));
    }

    #[test]
    fn matches_contains_any_condition() {
        let mut query = Query::empty();
        query
            .filters
            .must
            .push(condition(Operator::ContainsAny, json!(["ERROR", "WARN"])));

        assert!(evaluate_query(
            &query,
            &Record::from_raw("WARN disk space low")
        ));
        assert!(!evaluate_query(
            &query,
            &Record::from_raw("INFO service ready")
        ));
    }

    #[test]
    fn matches_equals_condition() {
        let mut query = Query::empty();
        query
            .filters
            .must
            .push(condition(Operator::Equals, json!("healthcheck ok")));

        assert!(evaluate_query(&query, &Record::from_raw("healthcheck ok")));
        assert!(!evaluate_query(
            &query,
            &Record::from_raw("healthcheck failed")
        ));
    }

    #[test]
    fn applies_must_not_condition() {
        let mut query = Query::empty();
        query
            .filters
            .must
            .push(condition(Operator::Contains, json!("payment")));
        query
            .filters
            .must_not
            .push(condition(Operator::Contains, json!("debug")));

        assert!(evaluate_query(
            &query,
            &Record::from_raw("ERROR payment failed")
        ));
        assert!(!evaluate_query(
            &query,
            &Record::from_raw("DEBUG payment retry")
        ));
    }

    #[test]
    fn non_empty_should_requires_one_match() {
        let mut query = Query::empty();
        query
            .filters
            .must
            .push(condition(Operator::Contains, json!("payment")));
        query
            .filters
            .should
            .push(condition(Operator::Contains, json!("checkout")));
        query
            .filters
            .should
            .push(condition(Operator::Contains, json!("invoice")));

        assert!(evaluate_query(
            &query,
            &Record::from_raw("ERROR checkout payment failed")
        ));
        assert!(!evaluate_query(
            &query,
            &Record::from_raw("ERROR refund payment failed")
        ));
    }

    #[test]
    fn empty_should_does_not_filter_matches() {
        let mut query = Query::empty();
        query
            .filters
            .must
            .push(condition(Operator::Contains, json!("payment")));

        assert!(evaluate_query(
            &query,
            &Record::from_raw("ERROR payment failed")
        ));
    }

    #[test]
    fn contains_all_rejects_malformed_value_defensively() {
        let mut empty_array = Query::empty();
        empty_array
            .filters
            .must
            .push(condition(Operator::ContainsAll, json!([])));

        let mut mixed_array = Query::empty();
        mixed_array
            .filters
            .must
            .push(condition(Operator::ContainsAll, json!(["ERROR", 500])));

        let mut non_array = Query::empty();
        non_array
            .filters
            .must
            .push(condition(Operator::ContainsAll, json!("ERROR")));

        assert!(!evaluate_query(
            &empty_array,
            &Record::from_raw("ERROR payment failed")
        ));
        assert!(!evaluate_query(
            &mixed_array,
            &Record::from_raw("ERROR 500 payment failed")
        ));
        assert!(!evaluate_query(
            &non_array,
            &Record::from_raw("ERROR payment failed")
        ));
    }

    #[test]
    fn contains_any_rejects_malformed_value_defensively() {
        let mut empty_array = Query::empty();
        empty_array
            .filters
            .must
            .push(condition(Operator::ContainsAny, json!([])));

        let mut mixed_array = Query::empty();
        mixed_array
            .filters
            .must
            .push(condition(Operator::ContainsAny, json!(["ERROR", 500])));

        let mut non_array = Query::empty();
        non_array
            .filters
            .must
            .push(condition(Operator::ContainsAny, json!("ERROR")));

        assert!(!evaluate_query(
            &empty_array,
            &Record::from_raw("ERROR payment failed")
        ));
        assert!(!evaluate_query(
            &mixed_array,
            &Record::from_raw("ERROR 500 payment failed")
        ));
        assert!(!evaluate_query(
            &non_array,
            &Record::from_raw("ERROR payment failed")
        ));
    }

    #[test]
    fn defaults_to_case_insensitive_matching() {
        let mut query = Query::empty();
        query
            .filters
            .must
            .push(condition(Operator::Contains, json!("error")));

        assert!(evaluate_query(
            &query,
            &Record::from_raw("ERROR payment failed")
        ));
    }

    #[test]
    fn supports_case_sensitive_matching() {
        let mut query = Query::empty();
        query.options.case_sensitive = true;
        query
            .filters
            .must
            .push(condition(Operator::Contains, json!("error")));

        assert!(!evaluate_query(
            &query,
            &Record::from_raw("ERROR payment failed")
        ));
    }

    #[test]
    fn resolves_known_structured_field_before_fields_map() {
        let mut record = Record::from_raw("message=fallback");
        record.message = Some("structured message".to_string());
        record
            .fields
            .insert("message".to_string(), "fallback".to_string());

        let mut query = Query::empty();
        query.filters.must.push(field_condition(
            "message",
            Operator::Equals,
            json!("structured message"),
        ));

        assert!(evaluate_query(&query, &record));
    }

    #[test]
    fn resolves_unknown_field_from_fields_map() {
        let mut record = Record::from_raw("request_id=abc");
        record
            .fields
            .insert("request_id".to_string(), "abc".to_string());

        let mut query = Query::empty();
        query.filters.must.push(field_condition(
            "request_id",
            Operator::Equals,
            json!("abc"),
        ));

        assert!(evaluate_query(&query, &record));
    }

    #[test]
    fn missing_field_does_not_match() {
        let record = Record::from_raw("INFO service started");

        let mut query = Query::empty();
        query.filters.must.push(field_condition(
            "request_id",
            Operator::Equals,
            json!("abc"),
        ));

        assert!(!evaluate_query(&query, &record));
    }
}
