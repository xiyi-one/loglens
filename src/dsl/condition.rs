use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Condition {
    pub field: String,
    #[serde(rename = "op")]
    pub operator: Operator,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub value: Option<Value>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Operator {
    Contains,
    ContainsAll,
    ContainsAny,
    Regex,
    Equals,
    NotEquals,
    Exists,
    In,
    Gte,
    Lte,
    IsPrivateIp,
    IsPublicIp,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_condition_operator_from_json() {
        let condition: Condition = serde_json::from_str(
            r#"{ "field": "raw", "op": "contains_all", "value": ["error", "timeout"] }"#,
        )
        .expect("condition should parse");

        assert_eq!(condition.field, "raw");
        assert_eq!(condition.operator, Operator::ContainsAll);
        assert_eq!(
            condition.value,
            Some(serde_json::json!(["error", "timeout"]))
        );
    }
}
