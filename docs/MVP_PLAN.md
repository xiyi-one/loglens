# MVP_PLAN.md

## 1. Goal

The goal of the MVP is to build a working local CLI that can:

- accept a query (DSL or simple natural language)
- scan log files
- filter results using DSL
- output matching lines

The MVP must be:

- correct
- simple
- testable
- streaming-based

LLM integration is NOT required for MVP.

---

## 2. Development Strategy

Development must follow strict order:

1. DSL (data model + validation)
2. Engine (execution)
3. Parser (basic)
4. CLI
5. Heuristic Translator
6. Follow Mode
7. Alias System
8. LLM (optional, last)

DO NOT change this order.

---

## 3. Phase Breakdown

---

## Phase 1: DSL Module

### Goal

Define query schema and validation.

### Tasks

- define `Query`
- define `Filters`
- define `Condition`
- define `Operator` enum
- define `Options`
- implement validation logic
- implement error types

### Deliverables

- Rust structs with serde support
- validation functions
- unit tests:
  - valid DSL
  - invalid DSL
  - operator/value mismatch

### Done When

- JSON DSL can be parsed into structs
- invalid DSL is rejected correctly

---

## Phase 2: Execution Engine

### Goal

Execute DSL against log lines.

### Tasks

- implement file scanner (BufRead)
- implement evaluator:
  - contains
  - contains_all
  - contains_any
  - equals
  - regex
- implement must / must_not logic
- implement limit handling

### Deliverables

- streaming engine
- evaluator module
- integration tests with sample logs

### Done When

- DSL can filter log lines correctly
- large files can be processed without loading into memory

---

## Phase 3: Parser

### Goal

Convert raw log lines into structured records.

### Tasks

- implement fallback parser (raw only)
- implement key=value parser
- implement JSON parser
- extract:
  - timestamp (optional)
  - level (optional)
  - message
  - fields map

### Deliverables

- Record struct
- parser pipeline
- parser tests

### Done When

- engine can use structured fields when available
- fallback works when parsing fails

---

## Phase 4: CLI

### Goal

Expose functionality via command line.

### Tasks

- integrate clap
- support:
  - file input
  - glob input
  - stdin
- support flags:
  - --dsl
  - --output text|json
  - --limit
  - --explain

### Deliverables

- working CLI binary
- CLI tests

### Done When

- user can run:
```bash
  loglens app.log --dsl query.json
```

* results are printed correctly

---

## Phase 5: Heuristic Translator

### Goal

Basic natural language → DSL conversion without LLM.

### Tasks

* detect keywords:

  * error, warn, timeout, login, payment
* detect time expressions (simple)
* detect IP conditions (public/private)
* map to DSL

### Deliverables

* heuristic translator module
* test cases for common queries

### Done When

* simple queries like:

  ```bash
  loglens app.log "login failed"
  ```

  produce valid DSL

---

## Phase 6: Follow Mode

### Goal

Support real-time log streaming.

### Tasks

* implement tail -f behavior
* continuously read appended lines
* reuse engine evaluator

### Deliverables

* follow mode flag (-f)
* streaming output

### Done When

* new log lines are processed in real time

---

## Phase 7: Alias System

### Goal

Allow saving and reusing queries.

### Tasks

* store alias → DSL mapping
* load alias
* integrate with CLI

### Deliverables

* alias file (JSON)
* CLI support

### Done When

* user can:

  ```bash
  loglens --save-alias bad-login ...
  loglens --alias bad-login
  ```

---

## Phase 8: LLM Integration (Optional)

### Goal

Improve NL → DSL translation.

### Tasks

* provider abstraction
* prompt design
* validation of output

### Constraints

* do NOT send raw logs
* always validate DSL

### Done When

* translator produces correct DSL
* fallback works if model fails

---

## 4. Codex Task Strategy

Each phase must be executed using focused prompts.

### Prompt Template

Use this template:

```text
Read AGENTS.md and relevant docs first.

Task:
<single goal>

Constraints:
- Rust only
- keep scope small
- do not modify unrelated modules
- add tests

Deliverables:
- code
- tests
- summary of changes
```

---

## 5. First Tasks to Run in Codex

### Task 1 (already done)

Scaffold project structure.

---

### Task 2 (next)

Implement DSL module.

---

### Task 3

Implement evaluator (basic contains).

---

### Task 4

Add file scanning.

---

### Task 5

Add parser fallback.

---

## 6. MVP Definition

MVP is complete when this works:

```bash
loglens app.log "login failed"
```

Behavior:

* query is translated (heuristic)
* file is scanned
* results are printed

---

## 7. Anti-Goals

During MVP, DO NOT:

* optimize prematurely
* add concurrency too early
* implement complex query planning
* introduce nested boolean trees
* integrate LLM too early

---

## 8. Engineering Discipline

* small commits
* test before next phase
* validate DSL before execution
* never bypass DSL

---

## 9. Success Criteria

MVP is successful if:

* CLI works reliably
* DSL works correctly
* engine is streaming
* no shell execution exists
* code is modular and testable


