# PRODUCT.md

## Project Name

LogLens

## One-Sentence Positioning

LogLens is a local-first Rust CLI for developers to search local log files using natural language, without needing to remember complex grep, awk, or sed syntax.

## Core Value Proposition

LogLens converts a natural-language log search request into a safe internal query plan, executes that plan locally against one or more log files, and returns relevant results without uploading raw log content by default.

## Target Users

### Backend Developers
Developers who frequently inspect application logs during debugging and incident investigation.

### Full-Stack Developers
Developers who need to search auth, API, payment, and database-related logs in local or staging environments.

### DevOps / SRE-lite Users
Users who need lightweight local log inspection on a VM, server, or container host without deploying ELK, Loki, or Datadog.

## Primary User Problems

### 1. Users remember intent, not shell syntax
Users often know what they want to find, such as:
- login failures from public IPs
- database timeout errors between 10 and 11
- payment retry related logs

But they do not want to manually compose:
- grep pipelines
- awk boolean conditions
- multi-stage exclusion filters
- regex expressions from scratch

### 2. Users often do not know the exact keyword
A user may know the meaning of an event but not the exact phrase used in the logs.

### 3. Users want immediate local execution
Users do not want to upload logs to a remote service just to perform a one-time search.

## Product Goals

1. Let users search logs using natural language.
2. Keep raw log content local by default.
3. Provide a better user experience than plain grep wrappers.
4. Be fast enough for real developer workflows.
5. Be easy to install and use as a single binary.

## Product Principles

1. Local-first: raw logs remain local unless the user explicitly opts into remote translation behavior.
2. Safe by design: never generate or execute shell commands from natural-language queries.
3. Structured when possible: prefer parsing logs into normalized structured records.
4. Graceful fallback: if structure inference fails, fall back to reliable line-based matching.
5. Transparent execution: show the normalized query plan or explain output.
6. Lightweight by default: prioritize fast startup and bounded memory usage.

## Open Source v1 Scope

The open source version should support:
- local file search
- multi-file search
- stdin input
- follow mode
- natural language query translation
- raw DSL input
- regex fallback mode
- plain text output
- JSON output
- alias save/load
- local config
- parser support for plain text, key=value, JSON logs, and timestamp-prefixed logs

## Non-Goals for Open Source v1

The open source version will not attempt to be:
- a distributed log platform
- a server-side indexing system
- a replacement for ELK, Loki, or Datadog
- a hosted observability product
- a dashboard-heavy monitoring suite
- a multi-user collaboration system

## Product Shape

LogLens must not be implemented as:

natural language -> shell command -> shell execution

LogLens must instead be implemented as:

natural language -> internal DSL / query plan -> local execution engine

This is a core design decision for safety, portability, testability, and long-term extensibility.

## Core User Experience

### Basic Search
```bash
loglens app.log "find all login failures where the IP is not private"
```

### Time-Bounded Search

```bash
loglens app.log "between 10am and 11am today, show database connection timeout errors"
```

### Follow Mode

```bash
loglens -f app.log "only show payment failures"
```

### Multi-File Search

```bash
loglens logs/*.log "show 5xx errors related to checkout"
```

### Alias Save/Reuse

```bash
loglens app.log "failed login from public IP" --save-alias bad-login
loglens --alias bad-login app.log
```

## Success Criteria

The open source CLI is successful if developers can replace workflows like:

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

* easier than grep for intent-based search
* safer than ad hoc shell pipelines
* lightweight enough to use every day
* local-first and privacy-aware

````
