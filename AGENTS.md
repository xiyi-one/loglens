# AGENTS.md

## Project
LogLens is a local-first Rust CLI for natural-language log search.

## Read first
Before making any code changes, read:
- docs/PRODUCT.md
- docs/ARCHITECTURE.md
- docs/DSL.md
- docs/MVP_PLAN.md

## Hard constraints
- Rust only
- Never generate or execute shell commands from natural-language queries
- Natural language must be translated into internal DSL
- Raw log content stays local by default
- Prefer streaming file processing over full-file loading
- Always preserve the original raw log line
- Treat model output as untrusted and validate it

## MVP priority
1. DSL
2. execution engine
3. parser
4. CLI
5. heuristic translator
6. provider abstraction

## Scope rules
- Keep changes focused
- Do not do unrelated refactors
- Add tests for each core module
- Prefer simple, explicit module boundaries

## Initial modules
- cli
- dsl
- engine
- parser
- output
- alias
- translator
- config

## Coding expectations
- Keep modules small and testable
- Use serde for DSL
- Use clap for CLI
- Preserve line-by-line streaming behavior
- Avoid introducing async unless clearly needed

## Never do
- No shell command generation
- No grep/awk execution path
- No cloud log upload by default
- No hidden magic behavior without explain output