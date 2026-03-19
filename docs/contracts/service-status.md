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
- `runtime_surface_summary`
- `operator_visible_message`
- `last_updated_at`

## Conditional fields

- `degraded_subtype` is required when `state` is `degraded`
- `denied_state` is required when `state` is `denied`

## Runtime surface summary

`runtime_surface_summary` is required.

It carries only bounded local runtime truth:

- `implemented_slices`
- `admitted_source_lanes`
- `bounded_runtime_only`

It is allowed to report only the runtime slices Cortex actually implements and the governed source lanes it actually admits.
It is not allowed to imply unbounded source support, future capability promises, or broader platform reach.

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

## Control boundary

Service status must remain informational only.

It must not include:

- next-action recommendations
- workflow or queue identifiers
- dispatch plans
- executor assignment
- orchestration state
