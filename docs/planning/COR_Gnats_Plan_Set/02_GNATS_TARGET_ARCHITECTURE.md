# COR Gnats Target Architecture

## Definition

A **Gnat** is a small, deterministic local worker that processes exactly one bounded shard and emits a schema-valid receipt.

A Gnat is not:

- an autonomous agent;
- an LLM session;
- a semantic judge;
- a long-running service;
- a workflow owner;
- a mutation authority.

## Local-stack topology

```text
Consuming application / Operator-Local
                  |
                  v
                COR
  validates request and constructs bounded batch plan
                  |
                  v
              FA-Local
 policy gate, scheduling, cancellation, retry, limits
                  |
        +---------+---------+
        |         |         |
      Gnat 1    Gnat 2    Gnat N
 deterministic local extraction workers
        |         |         |
        +---------+---------+
                  |
                  v
                COR
 validates receipts, reconciles structural output,
 emits extraction/retrieval package or partial result
          |                 |
          v                 v
       DF-Local         NeuronForge
 receipts/cache       optional semantics only
          |
          v
     Operator-Local
```

## Ownership matrix

| Concern | Owner |
|---|---|
| Source eligibility | COR |
| Source-lane admission | COR |
| Shard construction rules | COR |
| Execution routing | FA-Local |
| Worker process/thread lifecycle | FA-Local |
| Syntax extraction | COR Gnat implementation |
| Semantic interpretation | NeuronForge |
| Durable receipts/cache substrate | DF-Local |
| Operator controls and presentation | Operator-Local |
| App meaning and canonical truth | Consuming application |

## COR components

### 1. Batch planner

Produces a finite immutable `GnatRunPlan` from one validated request.

Responsibilities:

- enumerate eligible source items;
- enforce path and media-type boundaries;
- calculate source fingerprints;
- choose admitted worker type;
- cap total shards;
- assign deterministic shard IDs;
- declare required output contract.

It does not execute workers.

### 2. Worker adapter registry

Maps an admitted lane and task type to a deterministic callable.

Initial entries:

```text
text/markdown + syntax_extract -> MarkdownTextGnat
text/plain    + syntax_extract -> PlainTextGnat
application/pdf + syntax_extract -> PdfTextGnat
```

Later entries can reuse existing DOCX, RTF, ODT, and EPUB lane adapters after the concurrency model is proven.

### 3. Receipt validator

Validates every worker result against schema and verifies:

- run and shard IDs;
- source fingerprint;
- parser identity and version;
- declared lane;
- completion state;
- timing and resource observations;
- extraction payload reference or inline bounded payload;
- error reason code when unsuccessful.

### 4. Reconciler

Combines valid receipts only. It never invents missing output.

Possible run states:

```text
ready
partial_success
degraded
denied
cancelled
failed
stale
```

### 5. Status reporter extension

Adds Gnat capability truth without claiming active execution when FA-Local is unavailable.

Suggested fields:

```json
{
  "gnat_summary": {
    "profile": "bounded_parallel_extraction",
    "admitted_worker_types": ["markdown_syntax", "plain_text_syntax", "pdf_text_syntax"],
    "max_concurrency": 4,
    "fa_local_required": true,
    "fa_local_state": "unknown|ready|degraded|unavailable"
  }
}
```

## Worker rules

Every worker must:

1. receive an immutable shard envelope;
2. verify that the source fingerprint still matches;
3. execute one admitted operation;
4. avoid network access;
5. avoid mutation;
6. avoid spawning unapproved subprocesses;
7. produce a receipt on success or failure;
8. stop on cancellation or deadline;
9. expose bounded diagnostics only;
10. never enqueue another worker.

## Concurrency policy

Initial default:

```text
configured_max = 4
actual_workers = min(configured_max, max(1, logical_cpu_count - 2))
```

Additional limits:

- hard cap of 8 workers in the first generation;
- one shard per worker at a time;
- maximum input size remains governed by source-lane limits;
- per-shard deadline;
- run-level deadline;
- memory-pressure degradation;
- deterministic serial fallback.

## Deterministic fallback

Parallel execution must never be the only path.

```text
FA-Local ready        -> bounded parallel path
FA-Local unavailable  -> serial COR path, if contract permits
resource pressure     -> reduced concurrency or serial path
receipt conflict      -> fail closed / partial result
```

## No semantic drift

NeuronForge may consume reconciled syntax artifacts later. It must not be inserted into GNAT-01. This prevents a simple performance feature from turning into semantic ownership.
