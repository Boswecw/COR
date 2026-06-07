# Sanitization Report

## Intent

This packet is a sanitized-derived negative fixture for Scrivener Phase 0 evidence only.
It preserves readable authority while removing one expected text-body artifact.

## Observed lineage

- started from a sanitized derivative of the same source lineage already represented by `fixtures/scrivener/ambiguous/scriv-sanitized-fixture-v1/`
- after sanitization, one expected `content.rtf` artifact was intentionally removed in the copy used for this negative packet

## Sanitization and derivation changes

- replaced content-bearing files with placeholders
- sanitized binder titles in `*.scrivx`
- scrubbed selected device / modification metadata
- removed bundled PDF and autosave / backup binder artifacts
- removed `Files/Data/A5101A53-7D7B-425D-82F0-A2FDF9F156F5/content.rtf` only in the negative derivative represented by this packet
- retained the `Files/Data/A5101A53-7D7B-425D-82F0-A2FDF9F156F5/` directory with a `synopsis.txt` sidecar so the present-directory-with-missing-body condition survives version control

## Caveats

- this is negative evidence only and does not prove general support
- same-source linkage is based on direct local comparison of readable authority and UUID-bearing structure
- readable `.scrivx` does not by itself prove content completeness
