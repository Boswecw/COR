# Gnat Operator Run Status

`GnatOperatorRunStatus.v1` is the Cortex operator-facing status contract for one
GNAT run.

It exposes:

- run state and accessible state label;
- parallel, serial-fallback, cache-reuse, or mixed execution mode;
- shard counts and bounded failure reasons;
- concurrency used;
- cache hit/miss posture;
- DF-Local persistence state;
- cancel, rerun, and stale-cache controls with keyboard labels.

It does not expose raw source text, unrestricted source browsing, workflow IDs,
or invisible watcher state.

Operator-Local can render this contract without becoming syntax authority. Cortex
still determines run truth through receipt validation and reconciliation.
