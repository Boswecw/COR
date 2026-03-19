# Cortex Source Lane Draft - Scrivener

Governance note only. This draft defines planning posture and does not authorize implementation by itself.

## Status

Draft for governance review only.

## Purpose

This document defines the proposed admission boundary for a bounded Cortex Scrivener lane.

Scrivener is not a routine document-format peer.
This draft therefore treats Scrivener as a special-track bounded local project-source lane.

It does not authorize implementation by itself.

## 1. Lane identity

Provisional lane name:

- `scrivener_project`

Admitted family:

- local `.scriv` project directories only

Scrivener must be treated as a bounded project-source lane, not as a generic folder import and not as live application behavior.

## 2. Authority model

The Scrivener lane must define explicit project authority before admission.

At minimum, implementation must identify and honor bounded project truth for:

- project container discovery
- authoritative project index truth
- binder or equivalent hierarchy truth
- admitted textual item identity
- admitted item ordering or manuscript-path truth when that order is explicitly encoded

Cortex must not invent authority from compile behavior, editor state, UI conventions, or user intent.

## 3. Proposed admitted input boundary

Admitted inputs:

- local `.scriv` project directories
- readable project structures whose project and binder truth can be validated honestly
- projects whose admitted textual path can be established without relying on application runtime behavior

Not admitted:

- live app integration or open-project state
- sync-layer or cloud-state assumptions
- generic directory trees merely resembling a project
- exported standalone documents presented as project truth
- project variants whose structure cannot be verified truthfully

## 4. Proposed ready extraction surface

Ready extraction may include only content that can be recovered deterministically from admitted project truth.

Candidate ready surface:

- bounded textual content from explicitly admitted manuscript or draft items only
- paragraphs from admitted textual items
- explicit section or item boundaries when honestly recoverable from binder or item truth
- limited item titles when explicitly encoded and admissible by contract
- bounded manuscript ordering only when project truth states it clearly

All extraction must remain syntax-only.

Cortex must not infer narrative role, chapter meaning, drafting intent, editorial status, or project workflow meaning.

## 5. Explicit exclusions

The Scrivener lane must explicitly exclude any behavior outside bounded syntax-only project-source recovery.

Excluded by default:

- compile or export semantics
- editor state or UI state meaning
- labels, status, keywords, or metadata as semantic truth
- comments, annotations, snapshots, or revision semantics
- research-folder or media interpretation
- project-management or workflow semantics
- live sync, collaboration, or application-host behavior
- semantic labels, summaries, or editorial inference

## 6. Denial conditions

Inputs should be denied when they are recognizable as Scrivener candidates but contain out-of-lane or disallowed structures that make honest ready extraction inappropriate under the lane contract.

Candidate denial cases to refine during planning:

- project features whose presence would require compile semantics or application behavior to proceed honestly
- manuscript structures that require unsupported editorial or workflow interpretation
- admitted textual paths contaminated by out-of-lane annotation, review, or rich non-text structures if the contract excludes them absolutely

Denial conditions must be explicit and test-backed.

## 7. Unavailable conditions

Inputs should be unavailable when truthful extraction cannot proceed because project or parsing truth cannot be trusted.

Candidate unavailable cases:

- unreadable or incomplete project container
- missing authoritative project index or binder truth
- malformed project metadata or item references
- binder or manuscript truth too broken to establish an admitted textual path honestly

Unavailable conditions must be distinguished cleanly from denied conditions.

## 8. `partial_success` posture

`partial_success` is not admitted by default for Scrivener v1.

It may be introduced only if contract truth later demonstrates a stable honest degraded posture that does not hide uncertainty across project items.

Absent that proof, Scrivener v1 should follow ready, denied, and unavailable discipline only.

## 9. Provenance model

Scrivener extraction provenance must remain explicit and bounded.

At minimum, provenance should truthfully identify:

- source lane = `scrivener_project`
- local project source
- admitted project authority path used for extraction
- any bounded item or hierarchy identifiers necessary to explain emitted sections

Cortex must not emit provenance claims it cannot justify from project truth.

## 10. Admission requirements

Before Scrivener implementation begins, the lane must have:

- a fixture-first candidate set using bounded local project examples
- explicit ugly-case fixtures
- contract tests for ready, denied, and unavailable separation
- invariant coverage showing no cross-lane drift
- a documented anti-drift posture for project-source semantics

## 11. Anti-drift reminders

Scrivener admission must not be allowed to pull Cortex toward:

- project-management ownership
- workflow or editorial state ownership
- compile or export orchestration
- live app behavior
- generic folder ingestion
- semantic manuscript interpretation

The lane exists only to admit bounded local Scrivener project truth into Cortex's syntax-only extraction and retrieval-preparation surface.

## 12. Governance recommendation

Recommendation: approve Scrivener as the next planning target only through an explicit special-track project-source path and use this draft as the starting boundary note for the implementation plan.
