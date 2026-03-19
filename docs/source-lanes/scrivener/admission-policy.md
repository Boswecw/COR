# Scrivener Admission Policy Draft

## Status

Draft policy only.

This is not an admission decision.

## Policy posture

Because no local fixtures are present, this draft stays intentionally conservative.

The policy below defines what a future Scrivener lane should assume until structural proof says otherwise.

## Draft category table

| Category | Draft posture | Reason |
| --- | --- | --- |
| manuscript documents | admitted only under explicit bounded rule | likely closest fit to bounded syntax-only recovery, but still unproven locally |
| manuscript folders | admitted only as structural containers if explicit binder truth proves them non-content | ordering and hierarchy may matter, but container semantics are not yet proven |
| research documents | denied by default | high drift pressure toward broad convenience ingestion |
| notes | denied by default | strong editorial and workflow pressure without clear bounded value proof |
| synopsis-only nodes | denied by default | metadata-like and likely non-body content |
| templates | denied by default | project tooling rather than source truth |
| trash | denied by default | deletion state is not admissible content truth |
| snapshots or backups | denied by default | revision history and backup semantics are outside the lane |
| metadata-only entries | denied by default | no bounded extractable body proved |
| unknown node classes | unsupported pending proof | fail-closed posture required |

## Draft admission rules

Any future Scrivener lane should require all of the following before a node is admitted:

- explicit inclusion through authoritative project or binder truth
- deterministic mapping to a bounded local content body
- no dependence on compile or export behavior
- no dependence on editor state or workflow metadata
- no unresolved ambiguity about whether the node is manuscript or workspace material

## Draft denial rules

Nodes should be denied by default when they require:

- project-management semantics
- editorial semantics
- research convenience handling
- note or synopsis interpretation
- compile semantics
- application-state interpretation

## Open policy questions

These must be answered from fixtures before any admission ADR:

- how manuscript membership is encoded structurally
- whether manuscript folders ever carry their own body content
- whether notes and synopsis live as separate bounded artifacts or only as editor features
- whether any non-RTF or externalized content path exists in supported projects
