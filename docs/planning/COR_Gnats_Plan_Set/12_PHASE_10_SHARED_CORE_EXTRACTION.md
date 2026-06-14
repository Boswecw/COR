# Phase 10 Shared-Core Extraction Record

## Scope

Phase 10 extracts only scheduling-neutral primitives into `gnat_core` while
leaving Cortex GNAT domain behavior in `cortex_runtime/gnats`.

This is a local package extraction, not a new universal engine.

## Extracted interfaces

- `canonical_hash`
- `stable_short_digest`
- `sha256_bytes_digest`
- `source_path_token`
- `bounded_concurrency`
- `effective_concurrency`
- `RunStateCounts`
- `run_state_from_counts`
- `cache_key_from_identity`
- `CancellationToken`
- `ReceiptEnvelope`

## Cortex compatibility wrappers

Cortex continues to expose its existing GNAT API and compatibility symbols.
The following generic behaviors now delegate to `gnat_core`:

- immutable plan/run/shard hashing support;
- redacted source path token generation;
- requested/max concurrency hard-cap calculation;
- FA-Local effective concurrency calculation;
- deterministic receipt summary hash calculation;
- run-state derivation from accepted/rejected counts;
- cache-key construction from exact cache identity.

## Domain exclusions

`gnat_core` must not import:

- `cortex_runtime`;
- source-lane modules;
- GNAT workers;
- FA-Local concrete dispatch;
- DF-Local concrete storage;
- NeuronForge candidate logic;
- AuthorForge manuscript rules.

`tests/runtime/test_gnat_core_shared.py` verifies this import boundary.

## Second application requirements

The second application candidate remains AuthorForge.

A future AuthorForge consumer would need only neutral GNAT mechanics from this
core:

- deterministic request/shard identity;
- bounded concurrency negotiation inputs;
- receipt-envelope interface shape;
- cancellation token interface;
- lifecycle state vocabulary;
- cache identity/key construction.

AuthorForge-specific manuscript meaning, scene boundaries, entity observations,
continuity evidence, style observations, UI decisions, and promotion rules must
remain outside `gnat_core`.

## Distribution status

`gnat_core` version `0.1.0` is not yet published as an external package. It is a
local package inside Cortex with compatibility tests. External packaging should
wait until a second application imports it and passes its own compatibility
suite.
