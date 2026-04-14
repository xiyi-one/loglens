# LogLens

> 🔍 Local-first CLI for searching logs with natural language.

LogLens lets you search local log files using natural language instead of memorizing complex `grep`, `awk`, or `sed` commands.

---

## ✨ Features

- 🧠 Natural language log search
- 🧩 Internal DSL-based query engine (no shell execution)
- ⚡ Streaming execution (no full file loading)
- 🔍 Structured parsing:
  - JSON logs
  - key=value logs
  - plain text logs
- 📡 Follow mode (`tail -f` replacement)
- 🧾 Human-readable query plan (`--explain`)
- 📤 Output formats:
  - text (with highlight)
  - JSON (machine-readable)

---

## 🚀 Quick Start

### Build from source

```bash
git clone https://github.com/xiyi-one/loglens.git
cd loglens
cargo build --release
````

Binary will be at:

```bash
./target/release/loglens
```

---

## 📖 Usage

### Basic search

```bash
loglens app.log "login failed"
```

---

### Multiple files / glob

```bash
loglens logs/*.log "timeout"
```

---

### DSL mode

```bash
loglens app.log --dsl query.json
```

or:

```bash
loglens app.log --dsl '{"version":1,"filters":{"must":[{"field":"raw","op":"contains","value":"error"}]}}'
```

---

### Follow mode (like `tail -f`)

```bash
loglens --follow app.log "payment failed"
```

---

### Pipe input (stdin)

```bash
cat app.log | loglens - "timeout"
```

---

### JSON output

```bash
loglens app.log "error" --output json
```

---

### Explain query

```bash
loglens app.log "login failed" --explain
```

Example output:

```text
Query plan
version: 1
options: limit=100, follow=false, case_sensitive=false
must:
  - field=raw op=contains value="login failed"
must_not: none
should: none
```

---

## 🧠 How It Works

LogLens does NOT execute shell commands.

Instead:

```text
Natural language
   ↓
Heuristic translator
   ↓
Internal DSL
   ↓
Execution engine
   ↓
Streaming results
```

### Key Principles

* ❌ No `grep` / `awk` / shell pipelines
* 🔒 Logs stay local (no upload)
* ⚙️ Deterministic execution via DSL
* 📦 Single binary CLI

---

## 📦 Supported Log Formats

### JSON logs

```json
{"level":"ERROR","message":"db timeout"}
```

---

### key=value logs

```text
level=ERROR message="db timeout"
```

---

### Plain text logs

```text
ERROR db timeout
```

---

## 🎯 Natural Language Support (v0.1)

Currently supports:

* Level keywords:

  * `error`, `warn`, `info`
* Common patterns:

  * `login failed`
  * `payment failed`
  * `timeout`, `timed out`
* IP intent:

  * `public ip`
  * `private ip`

Fallback behavior:

```text
"some query"
→ raw contains "some query"
```

---

## ⚠️ Limitations (v0.1)

* No alias system yet
* No log rotation handling
* No concurrency optimization
* No LLM integration (heuristic only)
* Limited time expression support
* IP detection depends on parser coverage

---

## 🛣️ Roadmap

Planned improvements:

* [ ] Alias system
* [ ] Better natural language parsing
* [ ] LLM provider support (optional)
* [ ] Log rotation handling
* [ ] Parallel file scanning
* [ ] More DSL operators
* [ ] Advanced filtering (nested conditions)

---

## 🧪 Example

```bash
loglens app.log "login failed from public ip"
```

Output:

```text
[ERROR] login failed for user john ip=1.2.3.4
```

---

## 🧰 Why LogLens?

Traditional workflow:

```bash
grep "login failed" app.log | grep -v "192.168." | grep -v "10."
```

With LogLens:

```bash
loglens app.log "login failed from public ip"
```

---

## 📄 License

MIT License

