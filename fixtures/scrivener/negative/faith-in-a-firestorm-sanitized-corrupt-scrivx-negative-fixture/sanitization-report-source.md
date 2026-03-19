# Sanitization Report — faith-in-a-firestorm-sanitized-v1

## Source

Derived from the uploaded archive `Faith in a Firestorm.scriv-20260319T124826Z-1-001.zip`.

## Actions performed

- extracted the `.scriv` package from the source archive
- retained package structure and `*.scrivx` authority file
- replaced all `content.rtf` files with a fixed placeholder
- replaced all `synopsis.txt` files with a fixed placeholder
- replaced all `notes.rtf` files with a fixed placeholder
- sanitized `*.scrivx` binder/display titles and selected top-level metadata
- replaced selected history/UI/index surfaces with placeholders
- removed the bundled PDF export
- preserved UUID-based `Files/Data/<UUID>/...` directory structure

## Counts

- content.rtf replaced: 220
- synopsis.txt replaced: 191
- notes.rtf replaced: 6

## Limits

This sanitization preserves some structural metadata for evidence purposes. It should be treated as a bounded derivative fixture only.
