# DSL.md

## 1. Purpose

The LogLens DSL is the internal query representation used by the system.

It exists to ensure that:

- natural language queries are translated into a structured format
- execution is deterministic
- validation is possible before running a query
- shell command generation is never required

The DSL is the single source of truth between translator and execution engine.

---

## 2. Design Goals

The DSL must be:

- structured
- deterministic
- easy to validate
- safe to execute
- independent from shell syntax
- extensible for future versions

The DSL is not intended to be user-friendly by default.
It is primarily an internal machine-readable representation.

---

## 3. Root Structure

A query must have this top-level shape:

```json
{
  "version": 1,
  "filters": {},
  "options": {}
}
````

---

## 4. Root Fields

### 4.1 `version`

Type:

* integer

Required:

* yes

Meaning:

* DSL schema version

Allowed values for v1:

* `1`

---

### 4.2 `filters`

Type:

* object

Required:

* yes

Meaning:

* filtering logic used by the execution engine

---

### 4.3 `options`

Type:

* object

Required:

* no

Meaning:

* execution options such as limit, case sensitivity, follow mode

If omitted, defaults should be applied by the validator or execution layer.

---

## 5. Filters Object

The `filters` object contains the actual query conditions.

Supported fields in v1:

* `time_range`
* `level_in`
* `must`
* `must_not`
* `should`

Example:

```json
{
  "time_range": {
    "start": "2026-04-13T10:00:00",
    "end": "2026-04-13T11:00:00"
  },
  "level_in": ["ERROR", "WARN"],
  "must": [
    { "field": "message", "op": "contains", "value": "login failed" }
  ],
  "must_not": [
    { "field": "ip", "op": "is_private_ip", "value": true }
  ],
  "should": [
    { "field": "raw", "op": "contains", "value": "auth" }
  ]
}
```

---

## 6. Time Range

### 6.1 Structure

```json
"time_range": {
  "start": "ISO8601 datetime",
  "end": "ISO8601 datetime"
}
```

### 6.2 Rules

* `start` is optional
* `end` is optional
* at least one of `start` or `end` must be present
* datetimes must be parseable
* if both are present, `start <= end`

### 6.3 Semantics

* if `start` exists, include records with timestamp >= start
* if `end` exists, include records with timestamp <= end
* if a record has no timestamp, it does not match a time-range filter

---

## 7. Level Filter

### 7.1 Structure

```json
"level_in": ["ERROR", "WARN"]
```

### 7.2 Rules

* must be a non-empty array of strings
* comparison should be normalized case-insensitively in v1
* suggested normalized values:

  * TRACE
  * DEBUG
  * INFO
  * WARN
  * ERROR
  * FATAL

### 7.3 Semantics

A record matches `level_in` if its parsed level is one of the provided values.

If the record has no parsed level, it does not match `level_in`.

---

## 8. Condition Arrays

The DSL supports three condition arrays:

* `must`
* `must_not`
* `should`

Each array contains zero or more condition objects.

### 8.1 `must`

Semantics:

* all conditions must match
* logical AND

### 8.2 `must_not`

Semantics:

* none of the conditions may match
* logical NOT

### 8.3 `should`

Semantics in v1:

* optional OR-style preference
* if `should` exists and is non-empty, at least one `should` condition must match

This keeps v1 behavior simple and deterministic.

---

## 9. Condition Object

A condition object has this shape:

```json
{
  "field": "message",
  "op": "contains",
  "value": "login failed"
}
```

### 9.1 Fields

#### `field`

Type:

* string

Required:

* yes

Meaning:

* which field to evaluate

Supported field values in v1:

* `raw`
* `message`
* `level`
* `timestamp`
* `ip`
* `user`
* `request_id`
* `trace_id`
* `path`
* `method`
* `status_code`

Additional rule:

* unknown fields may still be allowed for parsed key/value or JSON logs
* the engine should first try structured fields, then fallback where appropriate

---

#### `op`

Type:

* string

Required:

* yes

Meaning:

* operator name

Supported operators in v1:

* `contains`
* `contains_all`
* `contains_any`
* `regex`
* `equals`
* `not_equals`
* `exists`
* `in`
* `gte`
* `lte`
* `is_private_ip`
* `is_public_ip`

---

#### `value`

Type:

* depends on operator

Required:

* yes for all operators except `exists`, `is_private_ip`, `is_public_ip`

Meaning:

* comparison value used by the operator

---

## 10. Operator Semantics

### 10.1 `contains`

Example:

```json
{ "field": "message", "op": "contains", "value": "timeout" }
```

Rules:

* `value` must be a string

Semantics:

* true if the target field contains the substring

---

### 10.2 `contains_all`

Example:

```json
{ "field": "raw", "op": "contains_all", "value": ["login", "failed"] }
```

Rules:

* `value` must be a non-empty array of strings

Semantics:

* true if all terms are present

---

### 10.3 `contains_any`

Example:

```json
{ "field": "raw", "op": "contains_any", "value": ["timeout", "timed out"] }
```

Rules:

* `value` must be a non-empty array of strings

Semantics:

* true if at least one term is present

---

### 10.4 `regex`

Example:

```json
{ "field": "message", "op": "regex", "value": "connection.*timeout" }
```

Rules:

* `value` must be a valid regex string

Semantics:

* true if regex matches the target field

---

### 10.5 `equals`

Example:

```json
{ "field": "status_code", "op": "equals", "value": "500" }
```

Rules:

* `value` must be a string, number, or boolean

Semantics:

* exact equality after normalization where applicable

---

### 10.6 `not_equals`

Example:

```json
{ "field": "method", "op": "not_equals", "value": "GET" }
```

Rules:

* same value constraints as `equals`

Semantics:

* inverse of equals

---

### 10.7 `exists`

Example:

```json
{ "field": "trace_id", "op": "exists" }
```

Rules:

* no `value` required

Semantics:

* true if the field exists and is not empty

---

### 10.8 `in`

Example:

```json
{ "field": "status_code", "op": "in", "value": ["500", "502", "503"] }
```

Rules:

* `value` must be a non-empty array

Semantics:

* true if the field equals one of the listed values

---

### 10.9 `gte`

Example:

```json
{ "field": "status_code", "op": "gte", "value": 500 }
```

Rules:

* `value` must be string or number
* numeric comparison should be attempted first when applicable
* timestamp comparison may also use this operator for future compatibility, but v1 should primarily use `time_range`

Semantics:

* field >= value

---

### 10.10 `lte`

Example:

```json
{ "field": "status_code", "op": "lte", "value": 599 }
```

Rules:

* same general constraints as `gte`

Semantics:

* field <= value

---

### 10.11 `is_private_ip`

Example:

```json
{ "field": "ip", "op": "is_private_ip" }
```

Rules:

* only valid when field is `ip`
* no `value` required

Semantics:

* true if the parsed IP is private

---

### 10.12 `is_public_ip`

Example:

```json
{ "field": "ip", "op": "is_public_ip" }
```

Rules:

* only valid when field is `ip`
* no `value` required

Semantics:

* true if the parsed IP is public

---

## 11. Options Object

The `options` object controls execution behavior.

Supported fields in v1:

* `limit`
* `follow`
* `case_sensitive`

Example:

```json
{
  "limit": 100,
  "follow": false,
  "case_sensitive": false
}
```

---

## 12. Option Fields

### 12.1 `limit`

Type:

* integer

Rules:

* optional
* if present, must be > 0

Semantics:

* maximum number of results returned

---

### 12.2 `follow`

Type:

* boolean

Rules:

* optional
* default = false

Semantics:

* enables follow/tail mode

---

### 12.3 `case_sensitive`

Type:

* boolean

Rules:

* optional
* default = false

Semantics:

* controls string comparison behavior where applicable

---

## 13. Validation Rules

A valid v1 DSL query must satisfy all of the following:

1. `version` must exist and equal `1`
2. `filters` must exist
3. `time_range.start <= time_range.end` if both exist
4. condition arrays must contain only valid condition objects
5. operators must be known
6. operator/value combinations must be valid
7. `exists`, `is_private_ip`, and `is_public_ip` must not require `value`
8. `is_private_ip` and `is_public_ip` are only valid for field `ip`
9. `limit` must be positive if present
10. arrays used by `contains_all`, `contains_any`, and `in` must be non-empty

Invalid DSL must fail validation before execution.

---

## 14. Execution Rules

The execution engine must evaluate the DSL in this order:

1. validate query
2. apply `time_range`
3. apply `level_in`
4. apply `must`
5. apply `must_not`
6. apply `should`

A record matches if:

* it satisfies all required filters
* it is not rejected by any `must_not`
* if `should` is present and non-empty, at least one `should` condition matches

---

## 15. Field Resolution Rules

When evaluating a condition:

1. try the parsed structured field first
2. if the target is `raw`, always use the raw line
3. if the field is unavailable, treat the condition as not matched
4. do not silently invent fields

For v1, the engine may also map well-known fields from parser output:

* `message`
* `level`
* `timestamp`
* entries inside `fields`

---

## 16. Defaults

If `options` is omitted, use:

```json
{
  "limit": 100,
  "follow": false,
  "case_sensitive": false
}
```

If `must`, `must_not`, or `should` are omitted, treat them as empty arrays.

---

## 17. Example Queries

### 17.1 Failed login from public IP

```json
{
  "version": 1,
  "filters": {
    "must": [
      { "field": "message", "op": "contains", "value": "login failed" },
      { "field": "ip", "op": "is_public_ip" }
    ]
  },
  "options": {
    "limit": 100,
    "follow": false,
    "case_sensitive": false
  }
}
```

---

### 17.2 Database timeout between two times

```json
{
  "version": 1,
  "filters": {
    "time_range": {
      "start": "2026-04-13T10:00:00",
      "end": "2026-04-13T11:00:00"
    },
    "must": [
      { "field": "message", "op": "contains_any", "value": ["timeout", "timed out"] },
      { "field": "raw", "op": "contains", "value": "database" }
    ]
  },
  "options": {
    "limit": 50,
    "follow": false,
    "case_sensitive": false
  }
}
```

---

### 17.3 Only 5xx checkout errors

```json
{
  "version": 1,
  "filters": {
    "must": [
      { "field": "path", "op": "contains", "value": "checkout" },
      { "field": "status_code", "op": "gte", "value": 500 },
      { "field": "status_code", "op": "lte", "value": 599 }
    ]
  },
  "options": {
    "limit": 20,
    "follow": false,
    "case_sensitive": false
  }
}
```

---

## 18. Rust Implementation Guidance

The Rust implementation should likely define:

* `Query`
* `Filters`
* `TimeRange`
* `Condition`
* `Operator`
* `Options`

Suggested implementation requirements:

* use `serde` for serialization/deserialization
* use `thiserror` for validation errors
* separate schema definition from validation logic
* keep operator-specific validation explicit

---

## 19. What Must Not Change in v1

For v1, do not introduce:

* nested boolean trees
* arbitrary scripting
* shell command equivalents
* implicit execution shortcuts
* fuzzy scoring in the DSL itself

Keep v1 simple and deterministic.

---

## 20. Summary

The DSL is the core contract of LogLens.

Translator → DSL → Validator → Engine

If this contract is stable, the rest of the system can evolve safely.
