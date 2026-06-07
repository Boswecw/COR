# GNAT Semantic Handoff

Schema: `schemas/gnat-semantic-handoff.schema.json`

Phase 9 defines the optional Cortex-to-NeuronForge handoff for semantic
candidate generation from reconciled GNAT syntax artifacts.

This is a separate contract family from extraction, retrieval preparation, and
the generic handoff envelope. It exists only when a user or consuming app makes
an explicit request for NeuronForge candidate generation.

Required posture:

- Cortex remains the source service.
- NeuronForge Local is the only destination for this contract.
- The source artifact is a referenced retrieval package, not inline raw text.
- The request carries the requesting user or app identity.
- The requested NeuronForge candidate contract family is explicit.
- The model and resource disclosure is explicit before candidate generation.
- Any semantic output is labeled `non_canonical_candidate`.
- COR receipts are immutable and receipt mutation is not allowed.
- The handoff carries no workflow, queue, executor, or dispatch planning fields.

Allowed source states:

- `ready`
- `partial_success`

`denied` and `stale` retrieval packages cannot emit this semantic handoff.
Cortex must fail closed instead of asking NeuronForge to infer meaning from an
unusable or freshness-invalid artifact.

Authority boundary:

NeuronForge may generate reviewable candidate artifacts from the referenced
syntax artifact. It must not rewrite, correct, promote, or replace COR receipts
or Cortex retrieval-package truth.
