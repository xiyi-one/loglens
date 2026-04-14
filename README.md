# LogLens Open Source CLI Spec

## 1. Project Overview

### 1.1 Name

LogLens

### 1.2 Positioning

A local-first command-line tool for developers to search log files using natural language, without requiring users to remember complex grep, awk, or sed syntax.

### 1.3 One-Sentence Value Proposition

Translate natural language log search requests into a safe local query plan, execute it locally against log files, and return relevant results without uploading log content.

### 1.4 Open Source Version Goal

The open source version should be a practical, developer-friendly CLI that works well for:

* single-machine local logs
* one or more plain text log files
* ad hoc debugging
* real-time tail/follow filtering
* privacy-sensitive local usage

The open source version is not intended to replace ELK, Loki, or Datadog. It is intended to replace common one-off grep/awk workflows for local debugging and small-scale operational troubleshooting.

---

## 2. Product Goals

### 2.1 Primary Goals

1. Let users search logs using natural language.
2. Keep all raw log content local by default.
3. Make the result more useful than a plain grep wrapper.
4. Be fast enough for real-world developer workflows.
5. Be easy to install as a single binary.

### 2.2 Non-Goals for Open Source v1

1. No distributed log storage.
2. No server-side indexing service.
3. No mandatory cloud backend.
4. No complex dashboards.
5. No full observability platform ambitions.
6. No heavy schema management system.

---

## 3. User Personas

### 3.1 Backend Developer

Needs to find error patterns in local app logs quickly.

### 3.2 Full-Stack Developer

Needs to investigate auth failures, API errors, and DB issues in dev or staging logs.

### 3.3 DevOps / SRE-lite User

Needs a lightweight tool to inspect logs on a VM, Docker host, or container output files without setting up ELK.

---

## 4. Core User Problems

### 4.1 Problem A: The user remembers intent, not syntax

The user remembers:

* "find login failures from public IPs"
* "show DB timeout errors between 10 and 11"
* "find requests that look like payment retries"

The user does not want to manually write:

* grep pipelines
* awk boolean conditions
* regex with multiple exclusions

### 4.2 Problem B: The user often does not know the exact keyword

The user may know the meaning of the event, but not the exact phrase used in the logs.

### 4.3 Problem C: The user wants immediate local execution

They do not want to upload logs to a remote system just to perform a one-time search.

---

## 5. Product Principles

1. Local-first: raw logs stay local unless the user explicitly opts into a remote model flow.
2. Safe by design: avoid shell command generation and shell execution.
3. Structured when possible: prefer parsing logs into structured records when feasible.
4. Graceful fallback: when structure inference fails, use robust line-based matching.
5. Transparent execution: always show the generated query plan before or alongside results.
6. Fast enough for daily use: prioritize low startup overhead and streaming-friendly execution.

---

## 6. High-Level Product Shape

LogLens should not be implemented as:

* natural language -> shell command -> execute shell command

LogLens should instead be implemented as:

* natural language -> internal query DSL / query plan -> local execution engine

This distinction is important because an internal query layer is safer, more portable, easier to test, and easier to extend.

---

## 7. Core User Experience

### 7.1 Basic Search

```bash
loglens app.log "find all login failures where the IP is not private"
```

Possible output:

```text
Query Plan:
  time: any
  must contain: ["login", "fail"]
  must_not_ip_ranges: ["10.0.0.0/8", "172.16.0.0/12", "192.168.0.0/16"]
  inferred fields: [timestamp, level, message, ip]

Matches: 23

2026-04-13 10:23:15 ERROR login failed user=john@ex.com ip=45.33.22.11
2026-04-13 10:24:02 ERROR login failed user=jane@ex.com ip=98.76.54.32
```

### 7.2 Time-Bounded Search

```bash
loglens app.log "between 10am and 11am today, show database connection timeout errors"
```

### 7.3 Follow Mode

```bash
loglens -f app.log "only show payment failures"
```

### 7.4 Multi-File Search

```bash
loglens logs/*.log "show 5xx errors related to checkout"
```

### 7.5 Save Alias

```bash
loglens app.log "failed login from public IP" --save-alias bad-login
loglens --alias bad-login app.log
```

---

## 8. Functional Requirements

### 8.1 Input Sources

The open source version should support:

* single file
* multiple files
* glob patterns
* stdin
* follow mode for append-only files

Examples:

```bash
cat app.log | loglens - "db timeout"
loglens app.log
loglens logs/*.log
loglens -f app.log
```

### 8.2 Query Input Modes

Support at least these query modes:

1. Natural language
2. Raw DSL
3. Regex fallback mode

Examples:

```bash
loglens app.log "find failed login from public IP"
loglens app.log --dsl '{"must":[{"field":"message","match":"login failed"}]}'
loglens app.log --regex "timeout|connection reset"
```

### 8.3 Output Requirements

Support:

* plain text human-readable output
* json output for scripts
* optional colorized terminal output

Examples:

```bash
loglens app.log "db timeout" --output text
loglens app.log "db timeout" --output json
```

### 8.4 Explainability

The tool should display one of:

* generated query DSL
* normalized execution plan
* confidence / fallback notes when parsing was ambiguous

### 8.5 Alias Support

Support saving frequently used query intents under local aliases. Aliases should be stored in a local config directory.

### 8.6 Parser Support

The open source version should support these log shapes:

1. plain text unstructured logs
2. key=value logs
3. JSON logs
4. common timestamp-prefixed logs

### 8.7 Common Semantic Operators

The engine should support:

* contains text
* contains all terms
* contains any terms
* excludes text
* regex match
* level equals / in
* timestamp range
* field equals
* field exists
* IP is private / public
* limit

---

## 9. Non-Functional Requirements

### 9.1 Privacy

Raw log lines must not be uploaded by default. If an LLM is used, only the user query and small parser hints may be sent unless the user explicitly opts in to broader context sharing.

### 9.2 Portability

The CLI should work on:

* macOS
* Linux
* Windows if feasible, but macOS/Linux should be prioritized for v1

### 9.3 Performance

Target expectations for v1:

* startup is near-instant
* scans moderately large local files efficiently
* follow mode does not block or lag noticeably
* memory usage is bounded and streaming-friendly

### 9.4 Reliability

The tool should degrade gracefully when:

* the log format cannot be inferred
* timestamps are missing
* fields cannot be extracted
* the model returns an incomplete plan

---

## 10. Recommended Architecture

### 10.1 Language Choice

Recommended choices:

* Go: faster CLI iteration, easy distribution, good concurrency model
* Rust: better control, performance, stronger type safety

For the open source CLI v1, Go is a strong default if speed of shipping matters more than absolute engine optimization.

### 10.2 Core Modules

#### Module A: CLI Layer

Responsibilities:

* argument parsing
* config loading
* mode selection
* formatting output

#### Module B: Query Translator

Responsibilities:

* convert natural language into internal DSL
* validate generated query plan
* normalize time expressions
* normalize IP/public/private predicates

#### Module C: Parser / Record Extractor

Responsibilities:

* infer log format
* parse timestamp
* parse level
* parse message body
* parse common key=value fields
* parse JSON logs

#### Module D: Execution Engine

Responsibilities:

* stream file lines
* convert lines to records when possible
* evaluate DSL predicates
* produce matching results

#### Module E: Follow Engine

Responsibilities:

* tail -f behavior
* incremental processing
* resume matching on appended lines

#### Module F: Alias / Local Memory

Responsibilities:

* save query aliases
* store user-confirmed translations
* optionally cache normalized query plans

---

## 11. Proposed Internal Query DSL

### 11.1 Design Goals

The DSL should be:

* machine-friendly
* deterministic
* easy to validate
* independent from shell syntax
* extensible for future UI/server versions

### 11.2 Example DSL

```json
{
  "version": 1,
  "filters": {
    "time_range": {
      "start": "2026-04-13T10:00:00",
      "end": "2026-04-13T11:00:00"
    },
    "level_in": ["ERROR", "WARN"],
    "must": [
      {"field": "message", "op": "contains", "value": "login failed"}
    ],
    "must_not": [
      {"field": "ip", "op": "is_private_ip", "value": true}
    ]
  },
  "options": {
    "limit": 100,
    "follow": false,
    "case_sensitive": false
  }
}
```

### 11.3 Supported Operators for v1

* contains
* contains_all
* contains_any
* regex
* equals
* not_equals
* exists
* in
* gte
* lte
* is_private_ip
* is_public_ip

### 11.4 Execution Semantics

The engine evaluates predicates against:

1. structured fields if available
2. fallback text fields if structure is unavailable

---

## 12. Log Parsing Strategy

### 12.1 Parsing Modes

LogLens should try parsers in this order:

1. JSON line parser
2. key=value parser
3. timestamp-prefixed plain log parser
4. unstructured text fallback

### 12.2 Normalized Record Shape

Every log line should be normalized into a record like:

```json
{
  "raw": "2026-04-13 10:23:15 ERROR login failed user=john ip=45.33.22.11",
  "timestamp": "2026-04-13T10:23:15",
  "level": "ERROR",
  "message": "login failed",
  "fields": {
    "user": "john",
    "ip": "45.33.22.11"
  }
}
```

### 12.3 Field Inference

The parser may infer well-known fields such as:

* timestamp
* level
* message
* ip
* user
* request_id
* trace_id
* path
* method
* status_code

### 12.4 Parser Constraints

The parser should never silently mutate raw values in a destructive way. The raw line must always remain available in the record.

---

## 13. Natural Language Translation Strategy

### 13.1 LLM Role

The LLM should only produce a structured query plan. It should not generate shell commands. It should not receive raw log content by default.

### 13.2 Translator Input

Input may include:

* the user query
* current local time
* selected files / path hints
* parser capabilities
* known field names, if inferred from a small sample or schema hints

### 13.3 Translator Output

The translator should produce:

* a valid DSL candidate
* optional assumptions
* optional confidence notes

### 13.4 Validation Layer

All LLM output must pass local validation before execution. Invalid fields, bad operators, or malformed time ranges must be rejected or repaired locally.

### 13.5 Fallback Mode

If translation confidence is low, LogLens should either:

* ask the user to confirm a simplified plan
* or run a best-effort broad search and explain the fallback

---

## 14. Safety Model

### 14.1 Security Principles

1. Never execute generated shell commands.
2. Never allow arbitrary code generation as part of query translation.
3. Restrict execution to the internal DSL engine.
4. Keep file access scoped to user-provided paths.

### 14.2 Privacy Principles

By default, do not send log data to any hosted service. Only send the natural language query and optional metadata needed for translation.

### 14.3 Model Provider Abstraction

The open source version should support a provider abstraction such as:

* OpenAI-compatible API
* local model adapter
* mock translator for offline tests

---

## 15. CLI Interface Proposal

### 15.1 Base Command

```bash
loglens [FILES...] [QUERY]
```

### 15.2 Important Flags

```text
-f, --follow           Follow file updates like tail -f
--dsl                  Pass raw query DSL
--regex                Pass raw regex query
--output               text | json
--limit                Max result count
--save-alias           Save current query under alias name
--alias                Run a saved alias
--explain              Show normalized query plan
--no-llm               Disable LLM translation, use heuristic parsing only
--provider             LLM provider name
--config               Config file path
```

### 15.3 Example Commands

```bash
loglens app.log "show login failures"
loglens app.log "today between 10 and 11 show db timeouts" --explain
loglens -f app.log "payment failed"
loglens logs/*.log "5xx checkout errors" --limit 20
loglens --alias bad-login app.log
loglens app.log --dsl @query.json
```

---

## 16. Local Config Layout

Suggested config locations:

* Linux/macOS: XDG-style config directory
* Windows: user config directory

Example:

```text
~/.config/loglens/
  config.toml
  aliases.json
  cache.json
```

### 16.1 Suggested Config Keys

```toml
[llm]
provider = "openai-compatible"
base_url = ""
api_key_env = "LOGLENS_API_KEY"
model = ""

auto_explain = true
default_output = "text"
default_limit = 100
```

---

## 17. MVP Scope

### 17.1 Must-Have for Open Source v1

1. single file and multi-file search
2. natural language to DSL translation
3. local execution engine
4. text + json output
5. explain mode
6. alias save/load
7. follow mode
8. support for plain logs, key=value logs, and JSON logs
9. basic time-range understanding
10. private/public IP predicate support

### 17.2 Should-Have for v1.1

1. better parser inference
2. more robust timestamp normalization
3. result highlighting
4. ranking / relevance sorting for fuzzy queries
5. field discovery command

### 17.3 Not Needed Yet

1. web UI
2. central server
3. cluster agent
4. long-term indexing service
5. multi-user collaboration

---

## 18. Suggested Development Phases

### Phase 0: Query Engine First

Build and test the internal DSL and execution engine before integrating any LLM.

Deliverables:

* DSL schema
* predicate evaluator
* file scanner
* parser abstraction
* explain output

### Phase 1: Heuristic Translation

Support basic natural language patterns without a remote model. Examples:

* time range detection
* common error intent keywords
* IP private/public detection
* level words like error, warn, info

### Phase 2: LLM Translation Layer

Add provider-based natural language translation to improve coverage.

### Phase 3: Follow Mode + Alias UX

Improve day-to-day usage with streaming filter mode and saved queries.

### Phase 4: Ecosystem Integrations

Optional later additions:

* VS Code extension
* Docker / kubectl helper wrappers
* shell completions

---

## 19. Recommended Repository Structure

```text
loglens/
  cmd/
    loglens/
  internal/
    cli/
    config/
    query/
    translator/
    parser/
    engine/
    follow/
    alias/
    output/
    provider/
  docs/
  examples/
  testdata/
  scripts/
  README.md
```

If using Rust, the equivalent module structure should follow the same conceptual boundaries.

---

## 20. Testing Strategy

### 20.1 Unit Tests

Test:

* DSL validation
* predicate evaluation
* parser behavior
* timestamp range handling
* IP classification

### 20.2 Golden Tests

Use sample logs and expected result snapshots.

### 20.3 Translator Contract Tests

Ensure the translator only produces valid DSL outputs.

### 20.4 Performance Tests

Use representative log fixtures of increasing size.

### 20.5 Safety Tests

Ensure no shell execution paths exist in the codebase.

---

## 21. Example User Stories

### Story 1

As a backend developer, I want to search for failed logins from public IPs using plain English so I do not need to remember grep syntax.

### Story 2

As a developer debugging a service, I want to follow logs in real time with a semantic filter so I only see relevant new events.

### Story 3

As a privacy-conscious user, I want my log content to remain local while still using an LLM to interpret my query.

### Story 4

As a frequent user, I want to save common searches as aliases so I can rerun them instantly.

---

## 22. Open Source Positioning

### 22.1 What the open source version should emphasize

* local-first
* privacy-aware
* developer UX
* simple installation
* transparent query plans

### 22.2 What should be left for future commercial versions, if any

* team-shared alias libraries
* enterprise policy controls
* remote agent mode
* indexed long-term storage
* IDE/cloud integrations beyond the basics

---

## 23. Design Decisions Summary

1. Do not generate shell commands.
2. Use an internal query DSL.
3. Treat the LLM as a translator, not an executor.
4. Keep raw logs local by default.
5. Prioritize execution engine quality over model cleverness.
6. Support structured parsing where possible, line-based fallback where necessary.
7. Make explainability a first-class feature.

---

## 24. What Success Looks Like

The open source CLI succeeds if developers can replace workflows like:

* grep + grep -v + awk
* tail -f piped into regex filters
* one-off shell history archaeology

with a workflow like:

```bash
loglens app.log "show login failures from public IPs"
loglens -f app.log "only payment timeout errors"
loglens logs/*.log "checkout 5xx errors in the last hour"
```

The user should feel that LogLens is:

* faster than searching shell history
* safer than writing ad hoc commands
* easier than grep for complex intent
* lightweight enough to use every day

---

## 25. Implementation Notes for AI Coding Assistants

When generating code for this project, follow these constraints:

1. Never implement shell-command generation or shell-command execution for user queries.
2. Always translate natural language into an internal validated DSL.
3. Keep parser and execution engine fully local.
4. Make modules small and testable.
5. Prefer streaming file processing over full-file loading.
6. Always preserve the original raw line.
7. Treat LLM output as untrusted input that must be validated.
8. Make follow mode incremental and memory-bounded.
9. Keep provider integration behind an interface.
10. Ensure the CLI works without an LLM in heuristic mode.

---

## 26. Next Document Suggestions

After this spec, the next recommended documents are:

1. `ARCHITECTURE.md` for module interactions and data flow
2. `DSL.md` for the full query schema and semantics
3. `MVP_PLAN.md` for milestone-based implementation
4. `PROMPTS.md` for translation prompt design
5. `TEST_PLAN.md` for fixtures and validation strategy

---

# 27. ARCHITECTURE.md (Detailed)

## 27.1 System Overview

```text
User CLI Input
   ↓
CLI Layer
   ↓
Query Translator (NL → DSL)
   ↓
Query Validator
   ↓
Execution Engine
   ↓
Parser Layer
   ↓
File Scanner / Stream Reader
   ↓
Output Formatter
```

## 27.2 Data Flow

1. User inputs query
2. CLI parses flags + input
3. If NL mode → send to Translator
4. Translator returns DSL
5. Validator checks DSL
6. Execution Engine streams logs
7. Parser extracts structured fields
8. Engine evaluates predicates
9. Matches passed to Output module

## 27.3 Module Contracts

### Translator Interface

```go
Translate(input string, context Context) (DSL, error)
```

### Engine Interface

```go
Execute(query DSL, sources []File) (<-chan Result, error)
```

### Parser Interface

```go
Parse(line string) Record
```

---

# 28. DSL.md (Complete Spec)

## 28.1 Root Structure

```json
{
  "version": 1,
  "filters": {},
  "options": {}
}
```

## 28.2 Filters Schema

### Time Range

```json
"time_range": {
  "start": "ISO8601",
  "end": "ISO8601"
}
```

### Level

```json
"level_in": ["ERROR", "WARN"]
```

### Conditions

```json
"must": [Condition]
"must_not": [Condition]
"should": [Condition]
```

### Condition Object

```json
{
  "field": "message | ip | user | *",
  "op": "contains | regex | equals | in | exists | is_private_ip",
  "value": "string | array | bool"
}
```

## 28.3 Options

```json
{
  "limit": 100,
  "follow": false,
  "case_sensitive": false
}
```

## 28.4 Execution Rules

1. must = AND
2. should = OR (optional)
3. must_not = exclusion
4. if structured field missing → fallback to raw

---

# 29. MVP_PLAN.md

## Phase 1: Core Engine (No AI)

### Goals

* DSL execution
* file scanning
* parser

### Tasks

*

---

## Phase 2: CLI

### Tasks

*

---

## Phase 3: Heuristic NL Parser

### Tasks

*

---

## Phase 4: LLM Integration

### Tasks

*

---

## Phase 5: Follow Mode

### Tasks

*

---

## Phase 6: Alias System

### Tasks

*

---

# 30. PROMPTS.md

## 30.1 System Prompt Template

""" You are a translator that converts natural language log queries into structured JSON DSL.

Rules:

1. Output ONLY valid JSON
2. Do not generate shell commands
3. Use provided schema strictly
4. If unsure, generate broad query """

## 30.2 Example Few-shot

Input: "find login failures"

Output:

```json
{
  "filters": {
    "must": [
      {"field": "message", "op": "contains", "value": "login failed"}
    ]
  }
}
```

---

# 31. TEST_PLAN.md

## 31.1 Unit Tests

* DSL validation
* predicate evaluation
* parser accuracy

## 31.2 Integration Tests

* end-to-end CLI queries

## 31.3 Golden Tests

* fixed input logs → expected output

## 31.4 Performance Tests

* large file scan benchmark

## 31.5 Safety Tests

* ensure no shell execution

---

# 32. FINAL IMPLEMENTATION CHECKLIST

## Engine

*

## CLI

*

## AI

*

## UX

*

---

# 33. FIRST MVP TARGET

Minimal usable command:

```bash
loglens app.log "login failed"
```

Expected behavior:

* parses query
* scans file
* returns results

If this works fast and reliably → MVP success

---

# 34. ENGINEERING PRIORITY ORDER

1. Execution Engine
2. Parser
3. DSL
4. CLI UX
5. LLM Layer

Never reverse this order.

---

# 35. END STATE

When completed, LogLens should behave like:

* ripgrep + jq + tail
* but with natural language interface
* and structured understanding

This is the baseline for a production-grade open source CLI.

   