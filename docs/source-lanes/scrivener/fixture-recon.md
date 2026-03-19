# Scrivener Fixture Recon

## Status

Blocked pending compatibility fixture acquisition plus remaining authority and mapping proof.

## Scope

This note records the current fixture posture for Scrivener Phase 0.

It is evidence only.
It does not authorize implementation.

## Workspace search result

Date:

- `2026-03-19`

Search scope:

- `/home/charlie/Forge/ecosystem`

Search result:

- one extracted sanitized derivative fixture exists at `fixtures/scrivener/ambiguous/scriv-sanitized-fixture-v1/`
- the extracted fixture contains `scriv-sanitized-fixture-v1.scriv/` with top-level `scriv-sanitized-fixture-v1.scrivx`
- direct local inspection shows `Files/Data/<uuid>/content.rtf`, `notes.rtf`, `synopsis.txt`, `notes.styles`, and `content.styles` artifacts inside the extracted fixture
- the supplied archive is retained beside the extracted fixture as `fixtures/scrivener/ambiguous/scriv-sanitized-fixture-v1/scriv-sanitized-fixture-v1.zip`
- one extracted sanitized derivative fixture now exists at `fixtures/scrivener/positive/scriv-mixed-structure-sanitized-v1/`
- the extracted positive fixture contains `scriv-mixed-structure-sanitized-v1.scriv/` with top-level `scriv-mixed-structure-sanitized-v1.scrivx`
- direct local inspection of the second fixture shows `DraftFolder`, `ResearchFolder`, `TrashFolder`, `TemplateFolderUUID`, and `BookmarksFolderUUID` surfaces in `.scrivx`
- direct local inspection of the second fixture shows multiple `BinderItem UUID` values repeated under `Files/Data/<UUID>/...`, including `0A7EDD9F-9DE0-4CC9-9AC1-EE0E3769B6A8`, `323E42B8-881A-4A72-938C-1D683D78D8DF`, `B23591A4-8EC3-4E1F-B242-ECCA0207561C`, and `4908D479-188E-4232-AA7D-38FD12692DC5`
- direct local inspection of the second fixture shows text, notes, synopsis, and style sidecars, but no `.pdf`, `.png`, or `.jpg` payload files in the retained archive
- the supplied archive is retained beside the second fixture as `fixtures/scrivener/positive/scriv-mixed-structure-sanitized-v1/scriv-sanitized-fixture-v2.zip`
- one extracted sanitized-derived negative fixture now exists at `fixtures/scrivener/negative/faith-in-a-firestorm-sanitized-corrupt-scrivx-negative-fixture/`
- the packet includes `README-source-sanitized.md`, `sanitization-report-source.md`, and `corruption-note.md`, which document the lineage `raw source project -> sanitized derivative -> intentionally corrupted sanitized copy`
- direct local inspection of the sanitized-derived negative shows `Faith in a Firestorm.scrivx` with sanitized metadata fields such as `Identifier="SANITIZED"`, `Device="SANITIZED"`, `Modified="SANITIZED"`, and `ModID="SANITIZED"`
- direct local `.scrivx` inspection also shows the malformed fragment `<Binder><BROKEN`, and the packet-local corruption note records removal of the closing `</ScrivenerProject>` tag
- direct local fixture inspection shows a package-shaped `faith-in-a-firestorm-sanitized-v1.scriv/` container with `Files/Data/`, `Settings/`, `binder.backup`, `binder.autosave.zip`, `search.indexes.xml`, `recents.txt`, and `.xps`
- the retained negative archive is stored as `fixtures/scrivener/negative/faith-in-a-firestorm-sanitized-corrupt-scrivx-negative-fixture/faith-in-a-firestorm-sanitized-corrupt-scrivx-negative-fixture.zip`
- one extracted clean sanitized sibling fixture now exists at `fixtures/scrivener/positive/faith-in-a-firestorm-sanitized-v1/`
- the packet is the clean same-source baseline for the canonical sanitized-corrupted negative fixture and is derived from the same raw source lineage
- direct local inspection of the clean baseline shows a readable sanitized `.scrivx` with `DraftFolder`, `ResearchFolder`, `TrashFolder`, `TemplateFolderUUID`, and `BookmarksFolderUUID`
- direct local inspection of the clean baseline shows multiple `BinderItem UUID` values repeated under `Files/Data/<UUID>/...`, including `323E42B8-881A-4A72-938C-1D683D78D8DF`, `B23591A4-8EC3-4E1F-B242-ECCA0207561C`, `209B8706-36B7-4E9F-BDC7-5B1E918EACD0`, and `972FDF2E-6367-49D9-9EB5-8197175E526A`
- the retained clean baseline archive is stored as `fixtures/scrivener/positive/faith-in-a-firestorm-sanitized-v1/faith-in-a-firestorm-sanitized-v1.zip`
- one earlier raw-derived negative packet remains at `fixtures/scrivener/negative/faith-in-a-firestorm-corrupt-scrivx-negative-fixture/`, but it is now retained only as superseded provenance history and not as the canonical malformed-authority packet

Implication:

- there are now four active local Scrivener evidence packets available for read-only inspection plus one superseded raw-derived audit packet
- single-fixture overfitting risk is reduced, but not replaced by admission-grade proof
- one source lineage now has an explicit same-source clean baseline plus malformed-authority negative pair, which narrows provenance ambiguity around the fail-closed authority claim
- the second fixture adds direct mixed-structure observation across draft, research, trash, template, and bookmark surfaces
- candidate binder-item UUID to `Files/Data/<UUID>/...` linkage is now directly observable across multiple examples
- one sanitized-derived malformed-authority negative case now strengthens fail-closed evidence with cleaner provenance discipline
- package resemblance alone is now directly shown to be insufficient when authoritative `.scrivx` XML is unreadable
- the repo still lacks compatibility coverage, broader irregular breadth, deterministic mapping proof, and proof that sanitization or restricted derivation did not remove needed admission evidence

## Current observed utility and limits

Observed utility:

- suitable for authority observation at the container level
- suitable for limited content-storage observation
- suitable for manuscript/research boundary observation in the second fixture
- suitable for candidate binder-to-content mapping observation in the second fixture
- suitable for malformed-authority fail-closed observation in the sanitized-derived negative fixture
- suitable for same-source clean-vs-corrupted authority comparison in the `faith-in-a-firestorm` pair
- suitable for provenance-discipline observation because the canonical negative packet is now derived from a sanitized derivative rather than directly from the raw source project
- suitable as governance-only reference material

Observed limits:

- the current active evidence packet mixes three sanitized derivatives with one sanitized-derived negative; the earlier raw-derived negative packet is retained only as superseded audit history
- no compatibility or newer-version fixture exists yet
- manuscript vs research structural distinction is now observable, but safe admission rules are not yet proven from it
- binder hierarchy, inclusion rules, and deterministic body linkage are not yet proven as contract truth
- research-side asset observation in the second fixture is structural only because binary payloads were removed
- negative coverage is still narrow and currently limited to one corrupted-authority case
- the same-source clean baseline reduces provenance ambiguity, but the canonical negative packet is still restricted evidence only and the earlier raw-derived negative packet must not be used to strengthen support claims

## Target fixture set

| Target fixture | Status | Blocking condition | Notes |
| --- | --- | --- | --- |
| sanitized derivative fixture in canonical intake packet | present but restricted | three restricted sanitized fixtures are still not enough for admission and sanitization limits still apply | useful for direct local inspection and provenance retention |
| same-source clean baseline paired with malformed-authority sibling | present but restricted | same-source pairing narrows provenance ambiguity only; it does not establish support or gate clearance | useful for clean-vs-corrupted authority comparison |
| mixed manuscript + research project | present but restricted | sanitized derivative and asset removal limit what can be inferred | useful for manuscript/research boundary observation |
| malformed-authority negative project | present but restricted | one sanitized-derived corrupted-`.scrivx` case does not establish full irregular breadth | useful for fail-closed authority denial evidence |
| simple manuscript-only project | still useful | current packet still lacks a clean baseline manuscript-only comparison | would help isolate authority from mixed-content pressure |
| manuscript + notes or synopsis-heavy project | missing | no sanitized source project present | required to test metadata-only and note-bearing nodes |
| nested binder and reordered items | missing | no sanitized source project present | required to prove hierarchy and ordering truth |
| irregular or missing-content project | missing | no sanitized source project present | required to define unavailable vs denied posture |
| newer-version project | missing | no sanitized source project present | required to test version drift |

## Canonical fixture path

When sanitized fixtures exist, store them under:

- `fixtures/scrivener/`

Current observed fixtures:

- `fixtures/scrivener/ambiguous/scriv-sanitized-fixture-v1/`
- `fixtures/scrivener/negative/faith-in-a-firestorm-sanitized-corrupt-scrivx-negative-fixture/`
- `fixtures/scrivener/positive/faith-in-a-firestorm-sanitized-v1/`
- `fixtures/scrivener/positive/scriv-mixed-structure-sanitized-v1/`

Retained superseded packet:

- `fixtures/scrivener/negative/faith-in-a-firestorm-corrupt-scrivx-negative-fixture/`

Supporting governance files expected there:

- `fixtures/scrivener/README.md`
- `fixtures/scrivener/fixture-intake-checklist.md`
- `fixtures/scrivener/fixtures-index.md`
- `fixtures/scrivener/provenance-policy.md`

Optional later test-facing wrapper only if needed:

- `tests/runtime/fixtures/scrivener/README.md`

That optional wrapper should point back to `fixtures/scrivener/` rather than becoming a second authority surface.

## Current gate

Fixture reconnaissance is not complete.

Implementation must remain blocked until at least one compatibility-oriented fixture and broader irregular coverage exist locally for read-only inspection, with the current active four-packet evidence surface used to narrow remaining authority, admission-boundary, and deterministic mapping ambiguity.

The retained raw-derived negative packet is audit history only and must not be used as canonical support evidence.
