# Worm Edge Taxonomy and Evidence Schema

Date and Time: 2026-04-21 10:45 PM America/New_York

## Relation edge types

Initial relation edge types:

- `git_submodule`
- `git_remote_reference`
- `workspace_member`
- `dependency_repo_reference`
- `ci_pipeline_reference`
- `docs_repo_reference`
- `shared_contract_reference`
- `unknown_external_reference`

## Finding classes

Initial finding classes:

- `stale_submodule_pointer`
- `missing_referenced_repo`
- `broken_workspace_member_reference`
- `missing_shared_contract_path`
- `repo_link_drift`
- `ci_points_to_missing_artifact`
- `declared_relationship_target_absent`
- `ambiguous_target_identity`

## Evidence envelopes

### Worm edge

A Worm edge records a discovered relationship.

Required fields:
- `kind`
- `schemaVersion`
- `relationType`
- `sourceRepo`
- `sourceArtifact`
- `discoveryMethod`
- `target`
- `crawlScope`
- `confidence`
- `posture`
- `timestamp`

### Worm finding

A Worm finding records a bounded cross-repo issue surfaced from one or more edges.

Required fields:
- `kind`
- `schemaVersion`
- `findingClass`
- `sourceRepo`
- `sourceArtifact`
- `relatedEdgeIds`
- `target`
- `reasonCode`
- `crawlScope`
- `confidence`
- `posture`
- `timestamp`

## Confidence scale

Allowed confidence values:

- `high`
- `medium`
- `low`
- `unassessable`

## Posture values

Allowed posture values:

- `evidence_bound`
- `ambiguous`
- `unresolved`
- `blocked`

## Crawl scope values

Allowed crawl scopes:

- `local_repo`
- `cross_repo`
- `external_reference`

## Notes

- Worm must emit structured payloads only.
- Narrative text may exist in optional explanatory fields, but not as a substitute for typed contracts.
- Ambiguous target resolution must stay explicit.
- Downstream systems must be able to distinguish edge evidence from finding evidence.
