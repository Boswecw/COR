# Cortex Local Doctrine

## Purpose

This document defines the non-negotiable operating posture for Cortex.

## Core rules

### 1. Service-only visibility

Cortex may appear only through consuming application surfaces such as HUD indicators, diagnostics panels, bounded controls, and contract-scoped operator views.

It must not become a standalone destination product or parallel workspace.

### 2. Syntax before semantics

Cortex extracts structure.
It does not decide meaning.

Allowed examples:

- headings
- sections
- tables
- metadata fields
- provenance markers
- chunk boundaries

Disallowed examples:

- summarization
- classification
- thematic labeling
- ranking doctrine
- business conclusions

### 3. Retrieval infrastructure, not retrieval authority

Cortex may shape retrieval-oriented artifacts needed for its function.
It may not decide what downstream systems should believe, prefer, or accept as truth.

### 4. Fail closed over convenience

If eligibility, integrity, completeness, freshness, or handoff confidence are insufficient, Cortex must narrow, degrade, deny, or mark stale rather than pretending readiness.

### 5. Privacy-preserving operational truth

Operational visibility is required.
Content surveillance is forbidden by default.

Diagnostics should prefer:

- counts
- hashes
- bounded metadata
- redacted summaries
- reason codes

Diagnostics must not default to:

- raw content previews
- free-form browsing
- convenience content inspection

### 6. App sovereignty remains intact

Consuming applications own business meaning, workflow policy, user experience, and canonical truth.
Cortex may assist those systems.
It may not absorb them.

### 7. Default-denied observation

No watcher may exist unless a consuming application contract explicitly enables it, scopes it, and exposes it through operator-visible status.

### 8. Explicit invalidation over assumed freshness

No governed artifact may be treated as indefinitely fresh by omission.
Artifacts must carry TTL, invalidation triggers, source-change markers, explicit stale state, or a bounded combination of those signals.

### 9. No hidden orchestration or ETL drift

Cortex must not become:

- a general local execution host
- a hidden workflow coordinator
- a generic transform-and-sink pipeline
- a dumping ground for file, memory, or search features that belong elsewhere

## Governing review question

If a proposal makes Cortex look more like a semantic engine, workflow router, surveillance surface, or ETL platform, the default answer is no unless constitutional necessity is demonstrated.
