# Cortex Contract - Service Status

## Purpose

This contract defines operator-visible runtime truth for Cortex.

## Wire-state rule

Phase 1 uses `ready` as the wire-state equivalent of healthy.

The allowed service states are:

- `ready`
- `degraded`
- `unavailable`
- `denied`
- `stale`
- `partial_success`

## Required fields

- `service_id`
- `service_class`
- `state`
- `operator_visible_message`
- `last_updated_at`

## Conditional fields

- `degraded_subtype` is required when `state` is `degraded`
- `denied_state` is required when `state` is `denied`

## Required posture

Service status must be specific enough that consuming applications can distinguish:

- extraction incompleteness
- integrity failure
- freshness uncertainty
- observation-contract absence
- dependency unavailability
- denial posture

## Visibility rule

If watcher scopes exist, their presence must be operator-visible through status or a bounded diagnostics surface.

## Privacy rule

Service status is allowed to report operational truth.
It is not allowed to become a raw-content diagnostic channel.
