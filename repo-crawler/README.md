# Slice 06 — Worm Discovery Adapter Contract

Date and Time: 2026-04-21 11:20 PM America/New_York

## Slice boundary

This is the **WRM-04** slice from the Worm plan.

This slice locks the first discovery adapter contracts for Worm:
- `gitmodules_parse`
- `package_manifest_parse`

It defines:
- adapter emission envelope
- adapter names
- required metadata
- example emissions
- lightweight validation

## Why this slice comes now

Before live traversal code is added, Worm needs stable adapter contracts so that:
- each discovery source emits structured edges
- downstream systems can distinguish source adapter provenance
- adapters remain bounded and testable

## Included

- `doc/system/worm/04_discovery_adapters.md`
- `doc/system/worm/schema/worm-adapter-emission.schema.json`
- `doc/system/worm/examples/adapter_emit_*.json`
- `scripts/validate_worm_discovery_adapters.py`

## Not included

- live adapter implementation code
- file parsing logic
- target normalization code
- Centipede handoff code
