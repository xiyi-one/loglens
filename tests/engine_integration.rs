use loglens::dsl::{Condition, Operator, Query};
use loglens::engine::execute_file;
use serde_json::json;

#[test]
fn scans_file_line_by_line_and_applies_limit() {
    let mut query = Query::empty();
    query.options.limit = Some(2);
    query.filters.must.push(Condition {
        field: "raw".to_string(),
        operator: Operator::ContainsAny,
        value: Some(json!(["ERROR", "WARN"])),
    });
    query.filters.must_not.push(Condition {
        field: "raw".to_string(),
        operator: Operator::Contains,
        value: Some(json!("ignore")),
    });

    let matches = execute_file("tests/fixtures/sample.log", &query).expect("scan should succeed");

    let raw_lines: Vec<_> = matches
        .into_iter()
        .map(|matched| matched.record.raw)
        .collect();

    assert_eq!(
        raw_lines,
        vec![
            "ERROR payment failed for request_id=abc",
            "WARN cache timeout for request_id=def",
        ]
    );
}

#[test]
fn evaluates_structured_fields_from_parsed_records() {
    let mut query = Query::empty();
    query.filters.must.push(Condition {
        field: "level".to_string(),
        operator: Operator::Equals,
        value: Some(json!("ERROR")),
    });
    query.filters.must.push(Condition {
        field: "request_id".to_string(),
        operator: Operator::Equals,
        value: Some(json!("json-1")),
    });

    let matches =
        execute_file("tests/fixtures/structured.log", &query).expect("scan should succeed");

    let raw_lines: Vec<_> = matches
        .into_iter()
        .map(|matched| matched.record.raw)
        .collect();

    assert_eq!(
        raw_lines,
        vec![r#"{"level":"ERROR","message":"login failed","request_id":"json-1"}"#]
    );
}

#[test]
fn field_matching_works_for_raw_text_logs() {
    let mut query = Query::empty();
    query.filters.must.push(Condition {
        field: "message".to_string(),
        operator: Operator::Contains,
        value: Some(json!("plain text")),
    });

    let matches =
        execute_file("tests/fixtures/structured.log", &query).expect("scan should succeed");

    let raw_lines: Vec<_> = matches
        .into_iter()
        .map(|matched| matched.record.raw)
        .collect();

    assert_eq!(raw_lines, vec!["INFO plain text line"]);
}

#[test]
fn field_matching_works_for_key_value_logs() {
    let mut query = Query::empty();
    query.filters.must.push(Condition {
        field: "request_id".to_string(),
        operator: Operator::Equals,
        value: Some(json!("kv-1")),
    });
    query.filters.must.push(Condition {
        field: "message".to_string(),
        operator: Operator::Equals,
        value: Some(json!("payment_failed")),
    });

    let matches =
        execute_file("tests/fixtures/structured.log", &query).expect("scan should succeed");

    let raw_lines: Vec<_> = matches
        .into_iter()
        .map(|matched| matched.record.raw)
        .collect();

    assert_eq!(
        raw_lines,
        vec!["level=ERROR msg=payment_failed request_id=kv-1"]
    );
}

#[test]
fn field_matching_works_for_json_logs() {
    let mut query = Query::empty();
    query.filters.must.push(Condition {
        field: "request_id".to_string(),
        operator: Operator::Equals,
        value: Some(json!("json-1")),
    });
    query.filters.must.push(Condition {
        field: "message".to_string(),
        operator: Operator::Contains,
        value: Some(json!("login")),
    });

    let matches =
        execute_file("tests/fixtures/structured.log", &query).expect("scan should succeed");

    let raw_lines: Vec<_> = matches
        .into_iter()
        .map(|matched| matched.record.raw)
        .collect();

    assert_eq!(
        raw_lines,
        vec![r#"{"level":"ERROR","message":"login failed","request_id":"json-1"}"#]
    );
}
