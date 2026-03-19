# Cortex Degradation Model

## Purpose

This document defines the truthful reduced-state vocabulary for Cortex.

## State definitions

### Healthy

All required components are available and the requested output satisfies its contract.

### Degraded

The request can still be served, but fidelity, completeness, breadth, or performance is reduced in a way that matters.

### Unavailable

Cortex cannot satisfy the requested contract.

### Denied

The request is rejected because it crosses an eligibility, integrity, privacy, contract, or authority boundary.

### Stale

An artifact exists, but freshness or validity can no longer be asserted.

### Partial success

Some work succeeded, but the result is incomplete and must be marked as such.

## Reason taxonomy guidance

Reason codes should be specific enough to distinguish at least:

- eligibility failure
- integrity failure
- extraction incomplete
- freshness unknown
- invalidation triggered
- contract scope denied
- privacy boundary denied
- downstream handoff re-prep required
- dependency unavailable

## Rules

- No reduced state may be hidden behind a generic "ok" surface.
- `stale` is not the same as `healthy`.
- `partial_success` is not the same as success.
- `denied` must carry a boundary-aware reason.
- Operator-visible state must be derived from the same contract posture that downstream consumers rely on.
