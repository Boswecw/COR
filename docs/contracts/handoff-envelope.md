# Cortex Contract - Handoff Envelope

## Purpose

This contract defines the bounded transfer-truth envelope Cortex may emit at the handoff boundary.

The handoff envelope exists to validate and communicate transfer truth.
It does not exist to coordinate downstream work.

## Allowed truth

The handoff envelope may carry only bounded transfer truth such as:

- `ready_for_transfer`
- `denied`
- `stale`
- `integrity_failed`
- `re_prep_required`

## Required posture

Every handoff envelope must make explicit:

- that Cortex is the source service
- which bounded destination surface is involved
- which artifact class is being transferred
- what the current transfer truth is
- what integrity context supports that truth
- what freshness posture applies
- what reverse signal, when present, describes bounded downstream disposition

## What it is not allowed to carry

The handoff envelope must not carry fields, shapes, or meanings that imply:

- retries
- workflow ids
- queue semantics
- executor selection
- dispatch planning
- orchestration state
- agent assignment
- downstream coordination ownership

Explicitly rejected examples include:

- `retry_count`
- `retry_policy`
- `workflow_id`
- `queue_name`
- `executor`
- `dispatch_plan`
- `orchestration_state`
- `agent_assignment`

## Reverse signaling meaning

Reverse signaling is optional, minimal, and bounded.

Forward transfer truth must be able to stand on its own without forcing a reverse-side protocol shape.
If reverse disposition exists, `reverse_signal` may be carried explicitly.

The allowed reverse-signal vocabulary is:

- `accepted`
- `rejected_reason_code`
- `re_prep_required`
- `stale`
- `integrity_failed`

Reverse signaling is not a conversation protocol.
It must not expand into retry negotiation, queue progression, dispatch planning, or generalized workflow status.

## Denial semantics

`denied` means the transfer cannot proceed because the handoff crosses a contract, destination, privacy, or structure boundary.

`denied` requires an explicit denial object with a bounded reason class.
It must not smuggle in workflow or queue failure language.

## Stale semantics

`stale` means a handoff artifact exists, but freshness can no longer be asserted.

`stale` must carry explicit freshness posture.
It must not silently degrade into transfer readiness.

## Integrity-failure semantics

`integrity_failed` means the envelope cannot be trusted for transfer because its integrity basis failed.

This state requires explicit integrity failure posture.
It must not be softened into a generic degraded or retryable workflow status.

## Validation expectations

Validation must prove that:

1. bounded handoff shapes pass
2. missing integrity context fails
3. invalid reverse signals fail
4. denied envelopes fail when denial taxonomy is invalid
5. orchestration-shaped fields are rejected
6. fixture discovery includes handoff artifacts automatically in `make validate`
