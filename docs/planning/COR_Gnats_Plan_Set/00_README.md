# COR Gnats Plan Set

## Purpose

This plan set adds a bounded local parallel-worker model, **Gnats**, to the present COR codebase without violating COR's existing constitutional limits.

COR already provides governed local intake, syntax-only extraction, retrieval-package emission, service-status truth, admitted source lanes for Markdown, text, PDF, DOCX, RTF, ODT, and EPUB, plus a bounded Scrivener authority-recon track. Gnats therefore begin as a parallel execution profile for existing deterministic work, not as a new semantic or orchestration authority.

## Architectural ruling

```text
COR owns bounded syntax-level content intelligence.
FA-Local owns policy-gated execution routing.
Gnats are short-lived deterministic workers governed through FA-Local.
NeuronForge owns semantic interpretation and candidate generation.
DF-Local owns durable local records and cache substrate.
Operator-Local owns user/operator presentation and controls.
```

## Documents

1. `01_CURRENT_STATE_AUDIT.md` — present COR capabilities and constraints.
2. `02_GNATS_TARGET_ARCHITECTURE.md` — target components and boundaries.
3. `03_CONTRACT_AND_SCHEMA_PLAN.md` — proposed contracts and schema sequence.
4. `04_IMPLEMENTATION_PHASES.md` — staged build order.
5. `05_FILE_LEVEL_CHANGE_MAP.md` — expected modifications against current structure.
6. `06_TEST_VALIDATION_AND_BENCHMARK_PLAN.md` — correctness and performance gates.
7. `07_FA_LOCAL_NEURONFORGE_DF_LOCAL_INTEGRATION.md` — local-stack integration.
8. `08_EXTRACTION_TO_SHARED_GNAT_CORE.md` — later reuse by AuthorForge and other apps.
9. `09_RISKS_NON_GOALS_AND_ADRS.md` — drift controls and decisions.
10. `10_CODEX_IMPLEMENTATION_PROMPTS.md` — implementation-ready prompts.
11. `11_MASTER_CHECKLIST.md` — execution checklist and completion definition.
12. `12_PHASE_10_SHARED_CORE_EXTRACTION.md` — local `gnat_core` extraction record.

## Recommended first proving slice

**GNAT-01 — Parallel Syntax-Only Extraction for Existing Admitted Text Lanes**

Scope:

- `.md` and `.txt` only
- one request split into bounded file shards
- in-process worker pool behind a COR-owned adapter
- read-only workers
- existing extraction output remains authoritative
- one receipt per shard plus one run summary
- no NeuronForge dependency
- no watcher
- no automatic mutation

This gives a measurable speed proof while preserving the current service doctrine.
