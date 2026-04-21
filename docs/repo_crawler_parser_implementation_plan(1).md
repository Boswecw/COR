# Mechanical Repo Crawler + Parser Implementation Plan

Date: April 20, 2026
Time: America/New_York

## 1. Purpose

Build a deterministic, mechanical repository crawler and parser in Rust for internal business-system use.

This system is not an AI tool.
It is a governed intake, parse, extraction, and evidence engine that:

- walks repositories
- respects ignore and exclusion policy
- fingerprints files
- parses supported source files
- extracts provable structural facts
- stores results locally
- supports incremental re-scan and controlled watch mode
- exports deterministic evidence

## 2. System Goal

Produce a local subsystem that can answer, with evidence:

- what files exist
- what changed since the last run
- what language each file appears to be
- what symbols and declarations are present
- what imports/includes are present
- what parse errors exist
- what TODO/FIXME and policy markers exist
- what scan evidence was produced in a given run

## 3. Non-Goals

The first implementation will not:

- call LLMs
- generate summaries
- attempt semantic reasoning
- auto-edit files
- claim full call-graph truth
- claim type-resolution truth across languages
- act as a generic agent

## 4. Technology Decision

### Core Language
Rust

### Core Libraries
- ignore for traversal and ignore handling
- tree-sitter plus language grammars for parsing
- rusqlite for local indexed storage
- notify for watch mode, backed by reconciliation
- sha2 for fingerprinting
- clap for CLI
- serde / serde_json for deterministic export

## 5. High-Level Architecture

```text
CLI / Service Entry
    ↓
Repo Discovery
    ↓
Traversal Engine
    ↓
Filter / Policy Layer
    ↓
Fingerprint Engine
    ↓
Language Classifier
    ↓
Parser Registry
    ↓
Extractor Registry
    ↓
Local Evidence Store
    ↓
Query / Export / Watch Reconcile
```

## 6. Core Subsystems

### 6.1 Repo Discovery
Responsibilities:
- detect repository root
- detect git presence
- capture branch / head metadata when available
- normalize root path
- reject unsafe or unsupported root states

Outputs:
- repo identity record
- root path
- vcs metadata snapshot

### 6.2 Traversal Engine
Responsibilities:
- recursively walk the repo tree
- respect .gitignore and configured excludes
- skip hidden or generated paths when policy says so
- support full scan and targeted scan modes

Scan modes:
- full scan
- changed-only scan
- path-targeted scan
- staged-only scan

### 6.3 Filter / Policy Layer
Responsibilities:
- skip known noise
- enforce size limits
- reject binary files unless specifically enabled
- apply extension and directory blocklists
- allow explicit overrides

Default skips:
- .git/
- node_modules/
- dist/
- build/
- target/
- coverage/
- bundled assets
- generated lockfiles unless explicitly enabled

### 6.4 Fingerprint Engine
Responsibilities:
- read metadata
- compute stable content hash
- compare against prior run
- determine unchanged / changed / added / deleted

Primary fields:
- size_bytes
- mtime_ns
- sha256
- binary/text determination

### 6.5 Language Classifier
Responsibilities:
- classify by path, extension, filename, and optional content hint
- assign parser family
- fall back cleanly to unknown text

Initial support target:
- Rust
- TypeScript
- JavaScript
- TSX
- JSX
- JSON
- TOML
- YAML
- Markdown
- Python
- Shell

### 6.6 Parser Registry
Responsibilities:
- map language id to parser adapter
- initialize parser per supported language
- expose a stable parse contract
- return parse result plus parse diagnostics

Common parser contract:
- language_id
- parser_id
- parse_success
- syntax_errors_present
- root_tree
- diagnostics

### 6.7 Extractor Registry
Responsibilities:
- transform parser tree into deterministic facts
- emit file facts, symbol facts, and edge facts
- reject unsupported claims

Initial extraction targets:
- modules / namespaces
- functions
- methods
- structs / classes
- enums
- traits / interfaces
- constants / statics
- imports / includes
- TODO / FIXME markers
- comment density metrics
- parse error counts

### 6.8 Local Evidence Store
Responsibilities:
- persist repo state
- persist file state
- persist extracted facts
- preserve scan-run history
- support deterministic re-index

Default store:
SQLite

### 6.9 Query / Export Layer
Responsibilities:
- query symbols by name
- query files by path/language/status
- export scan runs and findings
- export JSON and JSONL evidence packages

### 6.10 Watch / Reconcile Layer
Responsibilities:
- receive file change notifications
- debounce noisy events
- mark dirty paths
- re-stat and re-hash before re-parse
- periodically reconcile against actual filesystem state

Important rule:
watch events are advisory; filesystem reconciliation is authoritative.

## 7. Storage Model

### 7.1 Required Tables

#### repos
- repo_id
- root_path
- vcs_type
- head_ref
- head_commit
- last_scan_ts

#### files
- file_id
- repo_id
- rel_path
- abs_path
- size_bytes
- mtime_ns
- sha256
- lang
- parser_id
- is_binary
- parse_status
- last_indexed_ts

#### symbols
- symbol_id
- file_id
- kind
- name
- qualname
- start_byte
- end_byte
- start_line
- end_line
- visibility
- signature
- doc_excerpt

#### edges
- edge_id
- repo_id
- src_symbol_id
- dst_symbol_id
- edge_kind
- src_file_id
- dst_file_id
- payload_json

#### scan_runs
- scan_id
- repo_id
- started_ts
- finished_ts
- status
- files_seen
- files_changed
- files_parsed
- errors_json

#### parse_diagnostics
- diagnostic_id
- file_id
- severity
- code
- message
- start_line
- end_line
- payload_json

## 8. Determinism Rules

The crawler must follow these rules:

1. The same repo state should produce the same stored facts.
2. Unsupported claims must not be emitted.
3. Parse failure must be recorded explicitly, not silently ignored.
4. Watch mode must not claim final truth without reconciliation.
5. Deleted files must invalidate prior facts.
6. Export order should be stable and sortable.
7. Configuration must be explicit and versioned.

## 9. CLI Surface

```text
repo-crawler init
repo-crawler scan .
repo-crawler scan . --changed-only
repo-crawler scan . --paths src-tauri/src src/lib
repo-crawler watch .
repo-crawler query symbols "Registry"
repo-crawler query files --lang rust --status parse_error
repo-crawler export scan --scan-id <id> --format json
repo-crawler doctor
```

## 10. Config Surface

Example:

```toml
[repo]
root = "."
follow_symlinks = false

[crawl]
respect_gitignore = true
include_hidden = false
max_file_size_bytes = 1048576
exclude_dirs = ["node_modules", "dist", "build", "target", "coverage"]
exclude_extensions = ["png", "jpg", "jpeg", "gif", "pdf", "zip", "tar", "gz"]

[parse]
enabled_languages = ["rust", "ts", "tsx", "js", "jsx", "py", "json", "toml", "yaml", "md", "sh"]
max_parser_workers = 8

[watch]
enabled = false
debounce_ms = 500
poll_reconcile_seconds = 30

[store]
sqlite_path = ".repo-crawler/index.db"

[export]
default_format = "json"
include_diagnostics = true
```

## 11. Phased Implementation Program

## Phase 0 — Decision Lock

Goal:
Lock the system shape before coding spreads.

Deliverables:
- system scope note
- non-goals
- CLI command list
- SQLite schema draft
- config schema draft
- supported-language list
- evidence/export contract

Acceptance gate:
- no unresolved ambiguity on whether the tool is mechanical-only
- initial schema and CLI approved

## Phase 1 — Repo Discovery + Traversal Foundation

Goal:
Create a reliable repo walker.

Build:
- root detection
- ignore-aware walker
- exclusion policy
- traversal summaries
- full-scan command

Outputs:
- files seen
- files skipped
- skip reasons

Acceptance gate:
- respects .gitignore
- respects configured excludes
- produces stable file list for same repo state

## Phase 2 — Fingerprint + Local Store

Goal:
Add durable file indexing.

Build:
- SQLite bootstrap
- file metadata capture
- hash computation
- scan_runs table
- added/changed/unchanged detection

Outputs:
- persistent file table
- repeatable changed-file detection

Acceptance gate:
- second scan of unchanged repo yields zero changed files
- modified file updates cleanly
- deleted file is detected

## Phase 3 — Parser Registry Foundation

Goal:
Introduce parser dispatch without extraction complexity.

Build:
- parser trait
- parser registry
- Tree-sitter integration
- language adapters for Rust, TS/JS, Python, JSON/TOML/YAML fallback handling
- parse diagnostics recording

Outputs:
- per-file parse status
- diagnostics for parse failures

Acceptance gate:
- supported files parse into syntax trees
- unsupported files are marked unsupported, not treated as parse failures
- malformed files record diagnostics predictably

## Phase 4 — Deterministic Extraction v1

Goal:
Extract mechanical facts only.

Build:
- symbol extractor
- import extractor
- TODO/FIXME extractor
- file metrics extractor
- symbol/edge persistence

Outputs:
- symbol table
- edge table
- file-level metrics

Acceptance gate:
- extraction output for fixture repos is stable
- no inferred facts beyond parser-visible structure
- deleted or changed files invalidate stale symbol records

## Phase 5 — Incremental Re-indexing

Goal:
Avoid full re-parse for unchanged content.

Build:
- changed-only scan mode
- hash-based invalidation
- path-targeted scan mode
- optional git-diff targeting

Outputs:
- fast partial scans
- deterministic invalidation behavior

Acceptance gate:
- unchanged files do not re-parse
- changed files fully replace old facts
- path-targeted scan affects only requested scope

## Phase 6 — Query + Export Surface

Goal:
Make the crawler useful to operators and other internal tools.

Build:
- query symbols
- query files by language/status
- export scan report
- export symbol graph JSON/JSONL
- doctor command

Outputs:
- operator-readable and machine-readable evidence

Acceptance gate:
- exports are stable and sortable
- query results match database content
- doctor surfaces config/store/parser issues clearly

## Phase 7 — Watch + Reconcile

Goal:
Support near-real-time updates without sacrificing correctness.

Build:
- notify-backed watch mode
- debounce queue
- dirty path reconciliation
- periodic polling fallback/reconcile

Outputs:
- near-live index freshness
- controlled dirty-path reprocessing

Acceptance gate:
- create/update/delete events converge correctly after reconciliation
- missed or noisy events do not leave index in a silently wrong state

## Phase 8 — Hardening + Fixtures + Operational Controls

Goal:
Turn the crawler from “working” into “trustworthy.”

Build:
- fixture repos for regression tests
- malformed source fixtures
- generated/noise directory fixtures
- DB migration control
- export validation tests
- benchmark harness
- failure taxonomy

Outputs:
- repeatable proof set
- known error classes
- operational readiness notes

Acceptance gate:
- all fixture suites pass
- benchmarks stay inside accepted envelopes
- migration and export contracts are versioned

## 12. Testing Strategy

### 12.1 Unit Tests
- path filter rules
- file classifier
- hash logic
- language dispatch
- individual extractors

### 12.2 Integration Tests
- full scan against fixture repo
- changed-file detection
- deletion invalidation
- parse diagnostics persistence
- export correctness

### 12.3 Regression Fixtures
Prepare small fixture repos for:
- Rust-only repo
- TS/JS mixed repo
- polyglot repo
- malformed syntax repo
- noisy generated-files repo
- large nested-path repo

### 12.4 Watch Tests
- file create
- file modify
- file delete
- rapid sequence change burst
- missed-event recovery through poll reconcile

## 13. Error Taxonomy

All failures must be categorized:
- repo_discovery_error
- traversal_error
- permission_error
- file_read_error
- hash_error
- classify_error
- unsupported_language
- parse_error
- extract_error
- store_error
- export_error
- watch_error
- reconcile_error

## 14. Operational Rules

1. Never silently drop parse failures.
2. Never leave stale symbol rows after file replacement or deletion.
3. Never trust watch mode without reconciliation.
4. Never emit inferred relationships unless explicitly supported by the extractor contract.
5. Always retain scan-run evidence.
6. Keep config versioned.
7. Keep schema migrations explicit.

## 15. Suggested Initial Folder Layout

```text
repo-crawler/
  Cargo.toml
  src/
    main.rs
    app.rs
    cli.rs
    config/
      mod.rs
    repo/
      mod.rs
      discover.rs
      vcs.rs
    crawl/
      mod.rs
      walker.rs
      filter.rs
      fingerprint.rs
      scheduler.rs
    parse/
      mod.rs
      registry.rs
      common.rs
      rust.rs
      typescript.rs
      javascript.rs
      python.rs
      json.rs
      toml.rs
      yaml.rs
      markdown.rs
      shell.rs
    extract/
      mod.rs
      symbols.rs
      imports.rs
      todos.rs
      metrics.rs
    store/
      mod.rs
      sqlite.rs
      schema.rs
      migrations/
    watch/
      mod.rs
      fswatch.rs
      reconcile.rs
    model/
      mod.rs
      repo.rs
      file.rs
      symbol.rs
      edge.rs
      diagnostic.rs
      scan_run.rs
  tests/
    fixtures/
```

## 16. Recommended First Deliverable

The first real deliverable should be:

### Slice 01 — Crawl + Fingerprint + SQLite

Includes:
- init command
- scan command
- repo discovery
- ignore-aware traversal
- filter policy
- file hash + metadata capture
- SQLite persistence
- scan summary output

Why this first:
- proves the foundation
- gives immediate utility
- creates the base required for parser and extractor layers
- avoids mixing parsing complexity into basic crawler correctness

## 17. Recommended Build Order

1. schema + config lock
2. crawl foundation
3. fingerprint/store
4. parser registry
5. extraction v1
6. incremental re-index
7. query/export
8. watch/reconcile
9. hardening

## 18. Final Recommendation

Build this as a Rust mechanical subsystem with strict boundaries:

- crawler produces file truth
- parser produces syntax truth
- extractor produces structural fact truth
- store preserves evidence truth
- watch mode only accelerates freshness; it does not replace reconciliation

That gives you a strong internal business-system crawler that stays deterministic, inspectable, and expandable without becoming an AI-shaped black box.

