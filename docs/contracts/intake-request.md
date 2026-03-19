# Cortex Contract - Intake Request

## Purpose

This contract governs bounded admission of eligible local content into Cortex.

## Required fields

- `request_id`
- `requester_id`
- `source_type`
- `sources`
- `normalization_mode`
- `requested_artifact`
- `observation_policy`
- `submitted_at`

## Required posture

The contract must make explicit:

- what source class is being admitted
- who requested the intake
- whether the request is for extraction or retrieval-package preparation
- whether any watcher is being created
- whether the watcher is contract-scoped and operator-visible

## Source rules

Phase 1 source types are intentionally narrow:

- `file_path`
- `directory_scope`

Each source must identify:

- a stable source id
- source class
- local path

## Observation rules

Observation is default-denied.

If `observation_policy.mode` is `contract_scoped`, the request must also assert:

- an observation scope reference
- operator-visible status
- bounded path scope
- removability without service corruption
- source-class bounding

## Refusal guidance

Intake should be denied when the request attempts:

- unsupported source types
- invisible watcher creation
- broad uncontrolled observation
- out-of-scope artifact requests
- contract-invalid admission posture

## Resulting contract relationship

An accepted intake request does not itself imply extraction success, retrieval readiness, or semantic authority.
It only authorizes bounded Cortex intake work under the declared request posture.
