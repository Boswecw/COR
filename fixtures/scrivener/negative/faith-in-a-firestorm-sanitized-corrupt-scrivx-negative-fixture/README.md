# faith-in-a-firestorm-sanitized-corrupt-scrivx-negative-fixture

## Classification

- class: `negative`
- intake status: accepted with restrictions
- source provenance: sanitized-derived copy of a real Scrivener project
- sanitization posture: sanitized derivative first, intentional corruption second

## Lineage

raw source project -> sanitized derivative -> intentionally corrupted sanitized copy

## Packet contents

- `README.md`
- `README-source-sanitized.md`
- `sanitization-report-source.md`
- `corruption-note.md`
- `faith-in-a-firestorm-sanitized-v1.scriv/`
- `faith-in-a-firestorm-sanitized-corrupt-scrivx-negative-fixture.zip`

## Purpose

This packet is for fail-closed negative evidence only. The authoritative `*.scrivx` in the sanitized copy was intentionally malformed while preserving the surrounding package shape.

## What changed

- the sanitized derivative was copied
- the copied `*.scrivx` was intentionally made malformed
- the rest of the package shape was left intact

## Direct local observations

- the extracted packet contains a package-shaped `faith-in-a-firestorm-sanitized-v1.scriv/` container with top-level `Faith in a Firestorm.scrivx`
- direct `.scrivx` inspection shows sanitized metadata including `Identifier="SANITIZED"`, `Device="SANITIZED"`, `Modified="SANITIZED"`, and `ModID="SANITIZED"`
- direct `.scrivx` inspection also shows the malformed fragment `<Binder><BROKEN`
- the packet-local `corruption-note.md` records removal of the closing `</ScrivenerProject>` tag
- the extracted fixture still preserves surrounding project auxiliaries such as `Files/Data/`, `Settings/`, `binder.backup`, `binder.autosave.zip`, `search.indexes.xml`, `recents.txt`, and `.xps`

## Intended use

- malformed-authority observation
- fail-closed denial evidence
- governance-only reference

## Not suitable for

- support proof
- parser readiness claims
- deterministic mapping proof
- compatibility claims

## Known limitations

- one sanitized-derived malformed-authority case does not establish broad irregular coverage
- this packet does not prove `.scrivx` sufficiency
- this packet does not prove deterministic mapping across item types
- this packet does not resolve manuscript inclusion or exclusion rules
- this packet does not provide a separately retained clean sanitized baseline for the same source

## Restrictions

Evidence only. Not proof of general Scrivener support.
