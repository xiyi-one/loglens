# ARCHITECTURE.md

## 1. System Overview

LogLens is composed of several core modules:

- CLI Layer
- Translator Layer (NL → DSL)
- DSL Layer (query representation)
- Execution Engine
- Parser Layer
- Output Layer
- Alias / Config Layer

The system follows a strict data flow and module boundary to ensure maintainability and correctness.

---

## 2. High-Level Data Flow

```text
User Input (CLI)
   ↓
CLI Layer
   ↓
Translator (optional, NL → DSL)
   ↓
DSL Validator
   ↓
Execution Engine
   ↓
Parser Layer
   ↓
File Scanner (streaming)
   ↓
Output Formatter
````

---

## 3. Module Responsibilities

### 3.1 CLI Module (`cli/`)

Responsibilities:

* parse command-line arguments (clap)
* decide query mode (NL / DSL / regex)
* load config
* call translator if needed
* call execution engine
* format output

Must NOT:

* implement parsing logic
* implement engine logic

---

### 3.2 DSL Module (`dsl/`)

Responsibilities:

* define query structure (Query, Filters, Condition, Options)
* define operator enum
* validate DSL correctness
* ensure type safety

Must NOT:

* access files
* parse logs
* execute queries

---

### 3.3 Translator Module (`translator/`)

Responsibilities:

* convert natural language into DSL
* support heuristic mode (no LLM)
* support provider-based mode (future)
* return structured DSL

Must:

* produce valid DSL
* not generate shell commands

---

### 3.4 Engine Module (`engine/`)

Responsibilities:

* stream log lines from files
* evaluate DSL conditions
* coordinate parser + evaluator
* produce matching results

Sub-components:

* scanner (file reading)
* evaluator (condition execution)

Must NOT:

* parse CLI
* call LLM

---

### 3.5 Parser Module (`parser/`)

Responsibilities:

* convert raw log line → structured record
* support multiple formats

Parsing order:

1. JSON parser
2. key=value parser
3. timestamp-based parser
4. fallback raw text

Output:

```rust
struct Record {
    raw: String,
    timestamp: Option<DateTime>,
    level: Option<String>,
    message: Option<String>,
    fields: HashMap<String, String>,
}
```

---

### 3.6 Output Module (`output/`)

Responsibilities:

* format results (text / json)
* pretty print
* highlight matches (future)

---

### 3.7 Alias Module (`alias/`)

Responsibilities:

* store saved queries
* load alias
* map alias → DSL

---

### 3.8 Config Module (`config/`)

Responsibilities:

* load config file
* manage paths
* manage defaults

---

## 4. Execution Pipeline (Detailed)

1. CLI receives input
2. Determine mode:

   * DSL → skip translator
   * NL → call translator
   * regex → wrap into DSL
3. Validate DSL
4. Initialize engine
5. For each file:

   * open stream
   * read line-by-line
6. For each line:

   * parse → Record
   * evaluate DSL conditions
7. If match:

   * send to output
8. Stop when limit reached (if any)

---

## 5. Streaming Model

The engine MUST use streaming processing:

* do NOT load entire file into memory
* process line-by-line
* support follow mode (tail -f)

---

## 6. Dependency Direction (VERY IMPORTANT)

Modules must follow this direction:

```text
CLI
 ↓
Translator
 ↓
DSL
 ↓
Engine
 ↓
Parser
```

Rules:

* parser MUST NOT call engine
* engine MUST NOT call CLI
* DSL MUST be independent
* translator MUST NOT depend on engine

---

## 7. Error Handling Strategy

Use structured errors:

* invalid DSL → validation error
* parse failure → fallback, not crash
* file error → report and continue (if multiple files)

Use:

* anyhow for top-level
* thiserror for internal errors

---

## 8. Concurrency Model (v1)

For MVP:

* single-threaded streaming is acceptable

Optional future:

* multi-file parallel scan
* worker pool

---

## 9. Extensibility Points

Future extensions should plug into:

* new parser types
* new DSL operators
* new output formats
* new translator providers

---

## 10. Strict Constraints

1. NEVER generate shell commands
2. NEVER execute grep/awk
3. ALWAYS go through DSL
4. ALWAYS preserve raw log line
5. ALWAYS validate DSL before execution

---

## 11. Minimal MVP Path

To get a working system:

1. DSL module
2. Engine (simple contains match)
3. Parser (basic fallback)
4. CLI
5. Heuristic translator

LLM is NOT required for MVP

---

## 12. What Good Architecture Looks Like

* modules are independent
* each module has clear responsibility
* no circular dependencies
* DSL is central
* engine is streaming
