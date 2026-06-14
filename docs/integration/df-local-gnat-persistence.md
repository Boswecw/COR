# DF-Local Gnat Persistence

DF-Local owns durable storage for Cortex GNAT plans, immutable worker receipts,
run summaries, and exact-version cache records.

Cortex owns:

- GNAT contract validation;
- source eligibility and source fingerprints;
- syntax extraction;
- receipt reconciliation;
- cache-hit receipt rehydration for a new run.

DF-Local owns:

- durable plan records;
- immutable receipt records;
- summary records;
- cache record storage;
- retention and invalidation storage actions.

Persistence and cache failures must be reported through the GNAT persistence
status and must not fabricate extraction success or failure. Cortex run state is
still determined by validated receipts and deterministic reconciliation.
