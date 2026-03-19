# Reverse Handoff Signaling

## Purpose

This document constrains how downstream systems may signal bounded handoff outcomes back to Cortex.

## Phase 1 enum

The minimal bounded reverse-signaling vocabulary is:

- `accepted`
- `rejected_reason_code`
- `re_prep_required`
- `stale`
- `integrity_failed`

## Rule

Reverse signaling exists to report bounded handoff outcome, not to create downstream workflow control.
It is optional and appears only when a bounded downstream disposition exists.

## Required posture

Any reverse signal must remain:

- minimal
- explicit
- reason-coded when rejected
- attributable to a source and target surface
- free of hidden retry or workflow semantics

Forward transfer truth must remain valid without a reverse signal.

Schema alignment:

- `schemas/handoff-envelope.schema.json`
- `docs/contracts/handoff-envelope.md`

Disallowed examples include:

- `retry_count`
- `retry_policy`
- `workflow_id`
- `queue_name`
- `executor`
- `dispatch_plan`
- `orchestration_state`
- `agent_assignment`

## Anti-drift rule

If a proposed reverse signal starts to imply retries, sequencing, approvals, branching logic, or durable workflow ownership, it should be rejected or moved outside Cortex.
