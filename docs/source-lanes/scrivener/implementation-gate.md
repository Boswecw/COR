# Scrivener Implementation Gate

## Verdict

Blocked pending compatibility coverage and remaining authority proof.

## Additional blocking condition

Blocked pending unresolved authority and deterministic mapping ambiguity.

## Why implementation is blocked

| Gate area | Current result | Blocking reason |
| --- | --- | --- |
| fixture packet | partial | four active evidence packets now exist in the canonical intake surface across ambiguous, positive, and negative classes, plus one retained superseded raw-derived audit packet, but coverage is still narrow and there is no compat packet |
| structural authority | partial | three fixtures show `.scriv` plus `.scrivx` co-presence, the same-source clean baseline and the positive mixed-structure fixture expose draft/research/trash/template/bookmark surfaces, and the sanitized-derived negative packet shows malformed `.scrivx` must fail closed, but sole-authority and sufficiency claims remain unresolved |
| node admission policy | partial | mixed draft and research structure is now observable, but no safe admission rule exists yet for admissible subtree selection or non-manuscript exclusion |
| content mapping proof | partial | the same-source clean baseline and the second positive fixture show multiple `BinderItem UUID` values mirrored under `Files/Data/<UUID>/...`, but deterministic mapping across all item types is unproven |
| negative or irregular coverage | partial | one sanitized-derived intentionally corrupted-authority negative packet now exists with a clean same-source baseline, but irregular breadth is still narrow and missing-content or partial-package cases are still absent |
| compatibility coverage | failed | no version-drift or compatibility-oriented fixture exists |
| bounded RTF dependency decision | unresolved | the positive and ambiguous fixtures show heavy text-side `content.rtf` storage with notes, synopsis, and style sidecars, but dependency posture cannot be fixed from the current evidence packet alone |

## What may happen next

The next allowed actions are:

1. acquire a compatibility-oriented or newer-version fixture
2. acquire broader irregular coverage such as missing-content or partial-package failure cases
3. continue read-only comparison across the current active four-packet evidence surface
4. re-run the implementation gate

## What may not happen next

Until the gate is cleared, do not:

- add parser modules
- add runtime extraction code
- add schemas
- add CLI entrypoints
- add a provisional Scrivener lane by convenience

## Current recommendation

Do not draft a Scrivener admission ADR yet.

The repo first needs fixture-backed proof of:

- structural authority sufficiency
- safe binder inclusion and exclusion rules
- deterministic content mapping across item types
- version and irregular-project failure posture
- sanitization and restricted-derivation limits that do not collapse the evidence

Canonical fixture location for that work:

- `fixtures/scrivener/`

Current evidence artifacts:

- `fixtures/scrivener/ambiguous/scriv-sanitized-fixture-v1/`
- `fixtures/scrivener/negative/faith-in-a-firestorm-sanitized-corrupt-scrivx-negative-fixture/`
- `fixtures/scrivener/positive/faith-in-a-firestorm-sanitized-v1/`
- `fixtures/scrivener/positive/scriv-mixed-structure-sanitized-v1/`

Retained superseded audit packet:

- `fixtures/scrivener/negative/faith-in-a-firestorm-corrupt-scrivx-negative-fixture/`

The `faith-in-a-firestorm` clean baseline and canonical negative packet now form an explicit same-source pair.

The canonical negative packet strengthens fail-closed malformed-authority evidence while preserving sanitized-derivative provenance.

The earlier raw-derived negative packet is retained only as superseded provenance history.

It does not change the blocked verdict.
