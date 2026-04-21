# Slice 10 — Worm to Centipede Handoff Contract

Date and Time: 2026-04-22 12:20 AM America/New_York

## Slice boundary

This is the **WRM-08** slice from the Worm plan.

This slice locks the Worm to Centipede handoff envelope:
- handoff identity
- source lane identity
- evidence bundle references
- weighted-input posture
- proposal-ready summary surface
- lightweight validation

## Why this slice comes now

Worm is not the reconciler.
Centipede is the weighted reconciliation surface.

Before live integration, the handoff contract has to be explicit so Worm can feed Centipede without semantic drift or hidden assumptions.

## Included

- `doc/system/worm/08_centipede_handoff_contract.md`
- `doc/system/worm/schema/worm-centipede-handoff.schema.json`
- `doc/system/worm/examples/centipede_handoff_*.json`
- `scripts/validate_worm_centipede_handoff.py`

## Not included

- live Centipede ingestion code
- live weighting logic
- live approval workflows
