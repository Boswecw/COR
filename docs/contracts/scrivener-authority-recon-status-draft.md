# Cortex Draft Contract - Scrivener Authority Recon

Governance note only. This draft defines a conditional Stage 1 status contract for Scrivener authority reconnaissance. It does not admit the Scrivener lane and does not authorize runtime implementation by itself.

## Status

Conditional draft for blocked implementation posture only.

## Purpose

This document defines the maximum honest first runtime contract the current Scrivener evidence can support if governance later authorizes a bounded build.

The contract is status-first, not content-first.
It exists only to answer whether bounded project authority and candidate mapping surfaces can be observed honestly.

## 1. Input boundary

Admitted candidate inputs for this draft:

- local `.scriv` project directories only
- one package-shaped project container at a time
- one top-level authority candidate set at a time

Not admitted:

- exported standalone documents
- live application state
- sync or cloud behavior
- generic folder trees that merely resemble a project

## 2. Observation surface

Stage 1 may observe only:

- package-shaped `.scriv` presence
- candidate top-level `*.scrivx` files
- readable and well-formed XML status
- binder item identifiers
- directly encoded folder-role surfaces such as draft, research, trash, template, and bookmarks
- candidate `Files/Data/<UUID>/...` correspondences

Stage 1 must not emit manuscript text, research text, normalized project models, or workflow semantics.

## 3. Minimum status shape

The first runtime slice should expose a status object shaped at least like:

- lane state: `ready`, `denied`, or `unavailable`
- authority status summary
- package status summary
- mapping status summary
- observed role surfaces
- bounded fail-closed reason detail
- bounded provenance summary

The contract must not expose:

- manuscript-body output
- research-body output
- semantic labels
- editorial or workflow interpretation

## 4. `ready` posture

Within this draft, `ready` means the Stage 1 observation contract completed honestly.

It does not mean Scrivener manuscript extraction is ready.

`ready` is allowed only when:

- exactly one resolvable authority candidate is found
- the authority candidate is readable and well-formed
- package shape can be described honestly
- binder identifier surfaces can be observed
- mapping posture can be stated honestly, even if the result is `mapping_unresolved`

## 5. `denied` posture

`denied` is required when:

- the input is outside the Stage 1 source boundary
- the request attempts content extraction, workflow interpretation, or any other out-of-scope behavior
- governance or operator posture explicitly disables Scrivener observation work

Preferred shared reason classes:

- `unsupported_source_type`
- `ineligible_source`
- `operator_disabled`

## 6. `unavailable` posture

`unavailable` is required when truthful structural status cannot be trusted.

This includes:

- no resolvable `*.scrivx` candidate
- multiple conflicting `*.scrivx` candidates
- malformed or unreadable `*.scrivx`
- missing required structural surfaces
- package states outside currently observed evidence bounds

Malformed authority should be treated as unavailable by default in this draft because trust failed before any out-of-lane policy judgment could be made.

## 7. Mapping posture

Mapping posture must remain bounded and separate from extraction permission.

Allowed Stage 1 mapping summaries:

- `candidate_mapping_observed`
- `mapping_unresolved`
- `mapping_ambiguous`
- `mapping_unavailable`

`candidate_mapping_observed` does not authorize manuscript extraction.
`mapping_unresolved` does not by itself require `denied` if the observation result can still be reported honestly.

## 8. `partial_success` posture

`partial_success` is not introduced by this Stage 1 draft.

The draft should remain status-complete or fail closed.

## 9. Provenance posture

At minimum, provenance should identify:

- source lane candidate = `scrivener_project`
- observed project container path
- authority path used for observation
- whether the result came from readable authority, malformed authority, missing authority, or ambiguous authority

Provenance must remain structural only.

## 10. Non-goals

This draft must not drift into:

- manuscript extraction
- research extraction
- compile behavior
- project normalization
- workflow or editorial semantics
- repair heuristics disguised as success

## 11. Governance posture

This draft is build-ready but not self-authorizing.

If governance later authorizes a bounded first slice, implementation should begin from this contract and the Stage 1 slice plan in `docs/source-lanes/scrivener/authority-recon-slice-plan.md`.
