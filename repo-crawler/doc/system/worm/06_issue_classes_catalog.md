# Worm Issue Classes Catalog

Date and Time: 2026-04-21 11:50 PM America/New_York

## Purpose

This document defines the bounded issue catalog Worm is allowed to emit.

## Rules

- findings must use declared issue classes only
- findings must use declared reason codes only
- severity must remain machine-readable
- narrative text is optional and not authoritative
- ambiguous cases stay ambiguous and do not silently escalate

## Allowed issue classes

- `stale_submodule_pointer`
- `missing_referenced_repo`
- `broken_workspace_member_reference`
- `missing_shared_contract_path`
- `repo_link_drift`
- `ci_points_to_missing_artifact`
- `declared_relationship_target_absent`
- `ambiguous_target_identity`

## Reason code intent

Reason codes explain why a finding was emitted in a compact, auditable way.
They should be stable enough for Centipede and downstream governance surfaces to reconcile over time.
