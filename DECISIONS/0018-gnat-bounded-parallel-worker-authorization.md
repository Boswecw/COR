# Decision 0018 - Gnat bounded worker authorization

This record authorizes a narrow Gnat proving slice.
It does not authorize general agents, semantic workers, watchers, or mutation.

## Status

Accepted

## Date

2026-06-07

## Context

Cortex already owns bounded syntax-level content intelligence.
The first Gnat slice is intended to prove that existing deterministic Markdown and plain-text extraction can be planned as finite shards, executed by short-lived workers, and reconciled through receipts.

Without an explicit authorization record, a parallel-worker implementation could be misread as permission for Cortex to become a general scheduler, agent host, semantic evaluator, or workflow owner.

## Decision

Cortex may define Gnat contracts and implement deterministic local Gnat workers only for bounded syntax extraction.

For GNAT-01, admitted workers are limited to:

- `markdown_syntax`;
- `plain_text_syntax`.

Every Gnat worker must:

- process exactly one shard;
- be read-only;
- avoid network access;
- avoid source mutation;
- avoid subprocess spawning;
- emit a schema-valid receipt on success or failure when technically possible;
- preserve syntax-before-semantics;
- produce bounded diagnostics without raw operational content preview.

## Non-authorizations

This decision does not authorize:

- LLM use;
- semantic classification;
- ranking;
- editorial inference;
- workflow routing;
- watcher activation;
- background queue ownership by Cortex;
- PDF, DOCX, RTF, ODT, EPUB, or Scrivener Gnat admission.

## Consequences

Cortex may add a serial compatibility path before parallel dispatch is available.
That path is a contract proof and fallback behavior, not a claim that Cortex owns integrated execution scheduling.
