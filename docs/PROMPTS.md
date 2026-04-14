Read AGENTS.md and all docs in /docs first.

Then scaffold a Rust project for LogLens with these modules:
- cli
- dsl
- engine
- parser
- output
- alias
- translator
- config

Constraints:
- Rust only
- Do not implement LLM integration yet
- Do not implement shell command generation
- Keep changes minimal and focused
- Add test skeletons for each core module

Deliverables:
- code scaffolding
- test scaffolding
- a brief summary of created files

---

Read AGENTS.md and docs/DSL.md first.

Task:
Implement the DSL module in Rust.

Requirements:
- define Query, Filters, Condition, Options structs
- define Operator enum
- use serde for JSON parsing
- implement validation logic
- add unit tests

Constraints:
- do not touch engine or parser
- keep module structure clean
- split code into multiple files under dsl/

Deliverables:
- DSL implementation
- validation logic
- unit tests

---- 
Read AGENTS.md, docs/DSL.md, and docs/ARCHITECTURE.md first.

Task:
Implement the execution engine in Rust.

Scope:
- implement a streaming file scanner
- implement a basic evaluator for DSL conditions

Requirements:
- read files line-by-line using BufRead
- do NOT load entire file into memory
- implement support for:
  - contains
  - contains_all
  - contains_any
  - equals
- implement must and must_not logic
- apply limit option
- do NOT implement parser yet (use raw line only)

Constraints:
- do not modify DSL module
- do not implement translator
- keep engine modular (scanner + evaluator)
- no concurrency yet
- no async

Deliverables:
- engine module
- evaluator module
- unit tests for evaluator
- integration test using sample log file

Important:
- evaluation should operate on raw line (string)
- parser will be added later


----

Read AGENTS.md, docs/DSL.md, and docs/ARCHITECTURE.md first.

Task:
Refine the execution evaluator.

Requirements:
- implement should semantics:
  - if should is empty, ignore it
  - if should is non-empty, at least one should condition must match
- keep evaluation raw-line based for now
- make the raw-only behavior explicit in code comments
- add unit tests for should behavior
- add defensive tests for invalid condition value shapes

Constraints:
- do not integrate parser fields yet
- do not modify translator
- do not change DSL structs unless absolutely necessary
- keep changes limited to engine/evaluator tests

Deliverables:
- updated evaluator
- additional unit tests
- short summary of semantics implemented

----

Read AGENTS.md, docs/DSL.md, and docs/ARCHITECTURE.md first.

Task:
Implement the parser module in Rust and integrate it with the execution engine.

Requirements:
- define a Record struct used by parser and engine
- implement parser pipeline in this order:
  1. JSON line parser
  2. key=value parser
  3. fallback raw text parser
- preserve the original raw line
- extract structured fields where possible
- support at least:
  - raw
  - message
  - level
  - timestamp (optional, string is acceptable for now)
  - fields map
- integrate engine so that field resolution works like this:
  - field=raw -> use raw line
  - known structured fields -> use parsed record field first
  - unknown fields -> check fields map
  - if field missing -> condition does not match

Constraints:
- do not modify DSL schema unless absolutely necessary
- keep parser modular
- do not implement heuristic translator yet
- do not add follow mode
- keep execution streaming
- keep changes focused

Deliverables:
- parser module implementation
- Record struct
- parser tests
- engine integration for field-aware evaluation
- summary of how field resolution works

----

Read AGENTS.md, docs/ARCHITECTURE.md, and docs/DSL.md first.

Task:
Complete the parser pipeline and wire it into executor.

Requirements:
- implement parser::parse_line(line: &str) -> Record
- parser order must be:
  1. JSON parser
  2. key=value parser
  3. fallback text parser
- preserve raw line in all cases
- fallback text parser should set message to the raw line
- JSON parser should populate known fields when present:
  - message
  - level
  - timestamp
  - and remaining string-like fields into fields map
- key=value parser should populate:
  - known fields if present
  - remaining pairs into fields map
- integrate executor so each scanned line is parsed into Record before evaluation
- keep streaming execution
- add parser unit tests
- add integration tests showing field-aware matching works for:
  - raw text logs
  - key=value logs
  - JSON logs

Constraints:
- do not modify DSL schema
- do not add translator logic
- do not add follow mode
- keep changes focused to parser and engine integration

Deliverables:
- parser pipeline implementation
- executor integration
- tests for field-aware execution
- summary of parser precedence and fallback behavior

----

Read AGENTS.md, docs/ARCHITECTURE.md, and docs/MVP_PLAN.md first.

Task:
Implement the CLI layer for LogLens.

Requirements:
- use clap for argument parsing
- support:
  - single file input
  - multiple files (args)
  - glob patterns
  - stdin ("-")
- support flags:
  - --dsl <json string or file>
  - --output text|json
  - --limit
  - --explain
- call execution engine with parsed DSL
- print results to stdout
- support basic text output:
  - print raw log line
- support JSON output:
  - print Record as JSON

Constraints:
- do not implement natural language translator yet
- do not implement follow mode yet
- do not modify DSL or engine logic
- keep CLI thin (no business logic)

Deliverables:
- cli module implementation
- integration tests for CLI usage
- example usage commands
- summary of CLI behavior

----

Read AGENTS.md, docs/PRODUCT.md, docs/ARCHITECTURE.md, docs/DSL.md, and docs/MVP_PLAN.md first.

Task:
Implement a heuristic natural-language translator for LogLens and wire it into the CLI.

Requirements:
- support natural-language query input when --dsl is not provided
- keep --dsl behavior unchanged
- add a translator function that converts simple natural-language queries into a valid DSL Query
- support these heuristic cases:
  - keywords like error, warn, info
  - common phrases like "login failed", "payment failed", "timeout", "timed out"
  - simple public/private IP intent
  - simple time phrases are NOT required yet
- generated queries must always go through existing DSL validation
- if translation is broad or ambiguous, prefer a safe broad query
- --explain should print the final validated DSL regardless of whether it came from --dsl or heuristic translation

CLI behavior:
- if --dsl is present, use it
- otherwise require a positional natural-language query
- remaining positional inputs are files / globs / "-"

Constraints:
- do not add LLM integration
- do not modify engine semantics
- keep translator modular under src/translator/
- add translator unit tests
- add CLI integration tests for NL query mode

Deliverables:
- heuristic translator implementation
- CLI integration for heuristic mode
- tests
- short summary of supported NL patterns


----

Read AGENTS.md, docs/ARCHITECTURE.md, and docs/MVP_PLAN.md first.

Task:
Implement follow mode (tail -f behavior) in LogLens.

Requirements:
- support -f / --follow flag in CLI
- when enabled:
  - continuously read new lines appended to the file
  - reuse existing parser and evaluator
  - stream matching results to stdout in real time
- support only file inputs (not stdin) in follow mode
- do not break existing non-follow behavior
- keep streaming model (no full file reload)

Implementation hints:
- use a loop with file seeking
- track file position
- sleep briefly when no new data

Constraints:
- do not introduce async yet
- do not modify DSL
- do not modify translator
- keep implementation simple

Deliverables:
- follow mode implementation
- CLI flag support
- integration test (simulate appended logs)
- summary of follow behavior

----