# 3. Contract Surface

## Phase 1 contract set

The current contract surface covers:

- intake request
- extraction result
- retrieval package
- handoff envelope
- service status
- embedded diagnostics

## Intake request

The intake contract requires explicit source identity, source class, artifact request type, normalization mode, and observation posture.

Observation defaults to denied.
If a watcher is requested, it must be contract-scoped, operator-visible, removable, and bounded by source class.

## Extraction result

Extraction results are syntax-only by contract.

They must expose:

- provenance
- completeness posture
- freshness posture when relevant
- refusal posture when requests cross into semantics or other denied boundaries

## Retrieval package

Retrieval packages are:

- non-canonical
- non-semantic by default
- freshness-bound

They require explicit retrieval profile, freshness, invalidation, and completeness fields.

## Service status

Service status exposes truthful operator-visible state with the narrow Phase 1 vocabulary:

- `ready`
- `degraded`
- `unavailable`
- `denied`
- `stale`
- `partial_success`

## Handoff envelope

The handoff envelope is a bounded transfer-truth surface only.

It may express:

- `ready_for_transfer`
- `denied`
- `stale`
- `integrity_failed`
- `re_prep_required`

`reverse_signal` is optional.
When present, it must stay within the bounded reverse-signaling enum and must not become a workflow protocol.

It must reject orchestration-shaped fields such as:

- `retry_count`
- `workflow_id`
- `queue_name`
- `dispatch_plan`
- `agent_assignment`

## Supporting references

This section is grounded in:

- `docs/contracts/intake-request.md`
- `docs/contracts/extraction-result.md`
- `docs/contracts/retrieval-package.md`
- `docs/contracts/handoff-envelope.md`
- `docs/contracts/service-status.md`
- `docs/contracts/embedded-diagnostics.md`
