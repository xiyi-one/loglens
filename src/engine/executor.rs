use std::io;
use std::path::Path;

use crate::dsl::Query;
use crate::dsl::validate_query;
use crate::parser::Record;

use super::evaluator::{Match, matches};
use super::scanner::scan_file_lines;

#[derive(Debug, thiserror::Error)]
pub enum EngineError {
    #[error("invalid DSL query: {0}")]
    InvalidQuery(#[from] crate::dsl::ValidationError),
    #[error("file scan failed: {0}")]
    Io(#[from] io::Error),
}

pub fn execute_record(query: &Query, record: Record) -> Option<Match> {
    matches(query, record)
}

pub fn execute_file(path: impl AsRef<Path>, query: &Query) -> Result<Vec<Match>, EngineError> {
    validate_query(query)?;

    let limit = query.options.limit.unwrap_or(usize::MAX);
    let mut matches = Vec::new();

    for record in scan_file_lines(path)? {
        if let Some(matched) = execute_record(query, record?) {
            matches.push(matched);

            if matches.len() >= limit {
                break;
            }
        }
    }

    Ok(matches)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dsl::{Condition, Operator};
    use serde_json::json;

    #[test]
    fn executes_single_record_placeholder() {
        let record = Record::from_raw("INFO ready");
        let matched = execute_record(&Query::empty(), record.clone()).expect("record should match");

        assert_eq!(matched.record, record);
    }

    #[test]
    fn execute_record_filters_non_matches() {
        let mut query = Query::empty();
        query.filters.must.push(Condition {
            field: "raw".to_string(),
            operator: Operator::Contains,
            value: Some(json!("ERROR")),
        });

        let record = Record::from_raw("INFO ready");

        assert!(execute_record(&query, record).is_none());
    }
}
