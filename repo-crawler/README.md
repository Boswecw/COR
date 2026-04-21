# Slice 08 — Worm Issue Classes Catalog

Date and Time: 2026-04-21 11:50 PM America/New_York

## Slice boundary

This is the **WRM-06** slice from the Worm plan.

This slice locks Worm's bounded issue catalog:
- issue classes
- reason code registry
- severity posture
- example findings
- lightweight validation

## Why this slice comes now

Worm should not emit loose narrative complaints.
It should emit typed cross-repo issue classes with explicit reason codes and controlled posture.

## Included

- `doc/system/worm/06_issue_classes_catalog.md`
- `doc/system/worm/schema/worm-reason-code-catalog.schema.json`
- `doc/system/worm/examples/issue_catalog_*.json`
- `doc/system/worm/examples/finding_catalog_*.json`
- `scripts/validate_worm_issue_catalog.py`

## Not included

- live issue detection logic
- live graph walking logic
- Centipede reconciliation code
