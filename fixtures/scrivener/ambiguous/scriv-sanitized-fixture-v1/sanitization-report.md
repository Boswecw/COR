# Scrivener Sanitization Report

## Output
- Sanitized fixture archive: `scriv-sanitized-fixture-v1.zip`

## What I changed
- Renamed the top-level project folder to `scriv-sanitized-fixture-v1.scriv`
- Renamed the project file to `scriv-sanitized-fixture-v1.scrivx`
- Rewrote all `content.rtf` files to a short sanitized placeholder
- Rewrote all `notes.rtf` files to a short sanitized placeholder
- Rewrote all `synopsis.txt` files to a short sanitized placeholder
- Scrubbed the `.scrivx` project XML by replacing all binder titles with neutral placeholders and scrubbing the `Device` attribute
- Performed simple string scrubbing in remaining text/XML/INI files for known project/user strings

## Removed files
- `scriv-sanitized-fixture-v1.scriv/Files/binder.autosave.zip`
- `scriv-sanitized-fixture-v1.scriv/Files/binder.backup`
- `scriv-sanitized-fixture-v1.scriv/Icons/apple-touch-icon.png`
- `scriv-sanitized-fixture-v1.scriv/The Heart of the Storm test.pdf`
- `scriv-sanitized-fixture-v1.scriv/Files/search.indexes.xml`
- `scriv-sanitized-fixture-v1.scriv/Settings/recents.txt`
- `scriv-sanitized-fixture-v1.scriv/Files/writing.history.xml`
- `scriv-sanitized-fixture-v1.scriv/Files/Data/C846EA33-A727-44C4-9254-F5428C56C63F/content.png`
- `scriv-sanitized-fixture-v1.scriv/Files/Data/62FF97C4-994E-4778-B24F-1B07FB81601D/content.png`
- `scriv-sanitized-fixture-v1.scriv/Files/Data/41904D5A-4B87-411E-A636-BC8E10DD3948/content.jpg`
- `scriv-sanitized-fixture-v1.scriv/Files/Data/AA7F21EF-A4C4-485A-8044-6A7856D23E5B/content.pdf`
- `scriv-sanitized-fixture-v1.scriv/Files/Data/8011470C-3DAB-46BE-A166-FCF536FE86F6/content.pdf`
- `scriv-sanitized-fixture-v1.scriv/Files/Data/95B2CC32-0F79-4CBC-8BCD-8DE86385226A/content.png`

## Counts
- Files copied: 152
- RTF files rewritten: 128
- Synopsis files rewritten: 2
- XML/TXT/INI files rewritten: 9
- Files removed: 13

## Caveats
- This is a **best-effort fixture sanitization**, not a formal guarantee that the project will open cleanly in Scrivener.
- Structural layout, UUID-bearing paths, and the `.scrivx` binder tree were preserved as much as possible for Cortex Phase 0 reconnaissance.
- Embedded PDFs and images were removed to reduce private-content risk and keep the fixture focused on structural proof.
- Backup/history/recents/search-index artifacts were removed because they are not needed for the canonical fixture.

## Recommended repo classification
- Start as `ambiguous` or `candidate-positive` until you inspect it locally and confirm it still preserves the authority and mapping evidence you need.
