# Risks, Non-Goals, and ADR Set

## Major risks

### 1. Hidden orchestration drift

Risk: COR becomes a general scheduler.

Control: finite plan construction only; FA-Local owns execution routing.

### 2. Semantic drift

Risk: Gnats begin classifying or interpreting content.

Control: GNAT-01 syntax-only; NeuronForge handoff remains separate and optional.

### 3. Identity drift

Risk: repository is COR while code and documents still claim Cortex.

Control: identity ADR and separate migration slice.

### 4. Parallel nondeterminism

Risk: completion order changes output.

Control: deterministic shard IDs, stable sort, immutable plan, order-independent reconciliation tests.

### 5. Resource exhaustion

Risk: workers overwhelm local hardware.

Control: bounded concurrency, deadlines, lane size limits, memory-pressure degradation, serial fallback.

### 6. Surveillance expansion

Risk: watchers or diagnostics expose local content.

Control: default-denied watchers; hashes, counts, and reason codes rather than raw content.

### 7. Receipt inflation

Risk: detailed receipts become expensive or leak content.

Control: minimal evidence schema, artifact references, redaction, retention policy.

### 8. Premature shared-core extraction

Risk: wrong abstractions lock multiple apps together.

Control: extract only after COR plus a second application prove stable overlap.

## Non-goals for GNAT-01

- No LLM inference.
- No code modification.
- No manuscript rewriting.
- No watcher.
- No full repository semantic graph.
- No generic plugin marketplace.
- No cloud execution.
- No automatic PR creation.
- No Scrivener extraction admission.
- No replacement of existing serial CLIs.

## Proposed ADRs

### ADR — COR identity and Cortex lineage

Decision:

- COR is the internal/business-side deployment lineage copied from Cortex.
- Cortex is used for public-facing application deployments.
- Shared concepts may remain aligned, but identity and configuration must be explicit.

### ADR — Bounded Gnat worker authorization

Decision:

- COR may construct finite shard plans for deterministic syntax work.
- This does not authorize general workflow ownership.

### ADR — FA-Local execution ownership

Decision:

- FA-Local owns scheduling, lifecycle, cancellation, and resource enforcement.
- COR owns plan validity and receipt/result validation.

### ADR — Serial fallback

Decision:

- Existing serial execution remains a supported degradation path.
- The request contract states whether fallback is allowed.

### ADR — Gnat evidence and privacy

Decision:

- Every attempt emits bounded evidence.
- Raw content is excluded from operational receipts by default.

### ADR — Shared-core extraction threshold

Decision:

- No standalone `gnat-core` until a second app demonstrates stable commonality.
