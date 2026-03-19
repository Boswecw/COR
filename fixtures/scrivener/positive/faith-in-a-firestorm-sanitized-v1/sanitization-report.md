# Scrivener Sanitization Report — faith-in-a-firestorm-sanitized-v1

## Source

- raw source archive: `docs/Faith in a Firestorm.scriv-20260319T124826Z-1-001.zip`
- same-source canonical negative packet: `fixtures/scrivener/negative/faith-in-a-firestorm-sanitized-corrupt-scrivx-negative-fixture/`
- output archive: `faith-in-a-firestorm-sanitized-v1.zip`
- output project: `faith-in-a-firestorm-sanitized-v1.scriv`

## Intent

This output is a sanitized derivative fixture candidate for Cortex Scrivener Phase 0 reconnaissance.
It is intended as a clean same-source baseline for comparison with the canonical sanitized-corrupted negative packet, not as proof of general format support.

## Provenance note

- the canonical negative packet already documented the lineage `raw source project -> sanitized derivative -> intentionally corrupted sanitized copy`
- this clean baseline packet was materialized by copying that sanitized-derived project tree and reversing only the documented corruption in `Faith in a Firestorm.scrivx`
- the reversal was limited to removing the injected `<BROKEN` fragment after `<Binder>` and restoring the closing `</ScrivenerProject>` tag

## Sanitization posture

- package structure preserved
- sanitized `*.scrivx` metadata retained
- sanitized binder/display titles retained
- `Files/Data/<UUID>/...` structure preserved
- `content.rtf`, `synopsis.txt`, and `notes.rtf` remain placeholdered from the sanitized derivative
- selected history, UI, and index surfaces remain placeholdered from the sanitized derivative
- bundled PDF export remains removed from the sanitized derivative

## Caveats

- this packet is a best-effort sanitized derivative baseline, not a guarantee of perfect Scrivener round-trip behavior
- same-source pairing with the negative packet reduces provenance ambiguity, but it does not prove `.scrivx` sufficiency, deterministic mapping, or admissible manuscript-selection rules
- this derivative must be treated conservatively until broader compatibility and irregular coverage exist
