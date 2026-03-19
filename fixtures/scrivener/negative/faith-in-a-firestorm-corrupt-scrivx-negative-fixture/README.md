# faith-in-a-firestorm-corrupt-scrivx-negative-fixture

## Superseded status

This packet is no longer the canonical malformed-authority negative fixture.

It is retained only as superseded provenance history after replacement by:

- `../faith-in-a-firestorm-sanitized-corrupt-scrivx-negative-fixture/`

## Classification

- class: `negative`
- intake status: superseded / non-canonical provenance only
- source provenance: intentionally corrupted copy of a real Scrivener project archive
- sanitization posture: not treated as sanitized

## Packet contents

- `faith-in-a-firestorm-corrupt-scrivx-negative-fixture.zip`

This packet is intentionally archive-backed only.

The archive was not expanded again under `fixtures/` in order to avoid unnecessary duplication of a derived real-project copy.

## Direct local observations

- the retained archive contains a package-shaped `Faith in a Firestorm.scriv/` container
- the archive contains a top-level `Faith in a Firestorm.scrivx` file plus surrounding project auxiliaries such as `Files/Data/`, `Settings/`, `binder.backup`, `binder.autosave.zip`, `search.indexes.xml`, `recents.txt`, `.pdf`, and `.xps`
- direct inspection of the corrupted `.scrivx` shows a readable header and binder body at the start
- direct inspection of the corrupted `.scrivx` also shows an intentionally truncated tail ending with `<!-- intentionally truncated negative fixture -->` followed by `<ScrivenerProject BROKEN`

## Intended use

- provenance audit only

## Not suitable for

- support proof
- canonical negative evidence
- parser readiness claims
- mapping proof
- compatibility claims

## Intended negative condition

A Scrivener intake path should treat this package as:

- unreadable or malformed project authority
- not safely parseable
- denied, blocked, or unavailable rather than best-effort accepted

## Known limitations

- raw-derived provenance conflicts with the canonical sanitized-derivative fixture doctrine
- one malformed-authority case does not establish broad irregular coverage
- this packet does not prove `.scrivx` sufficiency
- this packet does not prove deterministic mapping across item types
- this packet does not resolve manuscript inclusion or exclusion rules
- this packet does not resolve compatibility spread

## Original corruption note

- The authoritative `*.scrivx` file was deliberately made malformed XML by truncating its content and appending a broken tag.
- No changes were made to the original uploaded archive.
- This artifact is for fail-closed structural testing only.

## SHA-256

`608341624b54c6aa82d7012d1e1d27925c8fad21ff17ffbf517d86c9e793dbbf`
