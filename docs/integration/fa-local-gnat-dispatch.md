# FA-Local Gnat Dispatch

GNAT-01 in this repository is the Cortex contract, serial proof, receipt reconciliation,
and FA-Local-gated bounded parallel worker proof for Cortex-owned Markdown/plain-text
syntax extraction.

It does not implement FA-Local worker lifecycle or integrated FA-Local dispatch.

The intended split is:

| Concern | Owner |
|---|---|
| Source eligibility | Cortex |
| Gnat request and plan validation | Cortex |
| Deterministic Markdown/plain-text extraction implementation | Cortex |
| Integrated routing and cancellation | FA-Local |
| Worker lifecycle and concurrency | FA-Local |
| Receipt validation and reconciliation | Cortex |

Future FA-Local integration should negotiate:

- supported contract versions;
- admitted worker types;
- concurrency clamp and hard cap;
- deadlines;
- cancellation;
- serial fallback policy.

When FA-Local is unavailable, Cortex service status must report that parallel Gnat execution is unavailable and may expose only the serial fallback state if the request permits it.

The Cortex parallel runner must be entered only after a ready FA-Local capability state
admits the run. This keeps worker execution deterministic and receipt-validated inside
Cortex while preserving FA-Local as the integrated routing, lifecycle, and cancellation owner.
