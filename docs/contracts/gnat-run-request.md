# Gnat Run Request

Schema: `schemas/gnat-run-request.schema.json`

`GnatRunRequest.v1` is the app or operator request for a bounded Cortex Gnat run.
GNAT-01 initially admitted `syntax_extract` over Markdown and plain-text source
references. Phase 7 extends the admitted GNAT source lanes to bounded text-layer
PDF when the host PDF tools are available.

The contract uses scoped source tokens and caller authority references.
It does not admit raw path persistence, workflow IDs, executor fields, semantic operations, or raw-content previews.

Parallel routing still requires FA-Local.
The Cortex serial path may run only when `serial_fallback_allowed` is true.
