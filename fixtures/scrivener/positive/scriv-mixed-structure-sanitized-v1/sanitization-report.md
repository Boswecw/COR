# Scrivener Sanitization Report — v2

## Source
- Source archive: `Symbiogenesis - Gunnach Protocol.scriv-20260319T122317Z-1-001.zip`
- Output archive: `scriv-sanitized-fixture-v2.zip`
- Output project: `scriv-mixed-structure-sanitized-v1.scriv`

## Intent
This output is a **sanitized derivative fixture candidate** for Cortex Scrivener Phase 0 reconnaissance.
It is intended for structural and mapping observation, not as proof of general format support.

## Changes applied
- Renamed the project and `.scrivx` file to neutral fixture names.
- Sanitized binder titles in the `.scrivx` project XML to neutral placeholders based on binder item type and sequence.
- Scrubbed the `Device` attribute in the `.scrivx` root metadata.
- Replaced all `content.rtf` bodies with placeholder text.
- Replaced all `notes.rtf` bodies with placeholder text.
- Replaced all `synopsis.txt` bodies with placeholder text.
- Removed history/recent/backup/search-index artifacts.
- Removed embedded assets (`.pdf`, `.png`) and project notes content.
- Preserved the package structure, UUID-bearing data directories, and style/metadata files not directly required to be removed.

## Counts
- `content.rtf` replaced: 68
- `notes.rtf` replaced: 7
- `synopsis.txt` replaced: 1
- files removed: 8
- `.styles` files preserved: 5

## Caveats
- This is a best-effort fixture sanitization, not a guarantee of perfect Scrivener round-trip behavior.
- Sanitization may reduce some authority or mapping clues that existed in original text-bearing or asset-bearing surfaces.
- This derivative should be classified conservatively until inspected in the repo context.
