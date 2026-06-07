# the-heart-of-the-storm-sanitized-missing-content-negative-fixture

## Classification

- class: `negative`
- intake status: accepted with restrictions
- source provenance: sanitized-derived negative copy of the same source lineage already represented by `fixtures/scrivener/ambiguous/scriv-sanitized-fixture-v1/`
- platform evidence: `.scrivx` `Creator="SCRWIN-3.1.6.0"`
- project metadata evidence: `.scrivx` `Version="2.0"`
- authority posture: readable `.scrivx`, intentionally missing expected `content.rtf`

## Packet contents

- `README.md`
- `sanitization-report.md`
- `corruption-note.md`
- `the-heart-of-the-storm-sanitized-v1.scriv/`
- `the-heart-of-the-storm-sanitized-missing-content-negative-fixture.zip`

## Intended use

- missing-content fail-closed observation
- readable-authority but unavailable-content observation
- same-source clean-vs-missing-content comparison against `fixtures/scrivener/ambiguous/scriv-sanitized-fixture-v1/`
- governance-only reference

## Direct local observations

- `.scrivx` remains readable and shows `Creator="SCRWIN-3.1.6.0"` plus `Version="2.0"`
- direct local comparison with `scriv-sanitized-fixture-v1` shows matching root binder structure and the same `BinderItem UUID="A5101A53-7D7B-425D-82F0-A2FDF9F156F5"` under the draft subtree
- `corruption-note.md` records removal of `Files/Data/A5101A53-7D7B-425D-82F0-A2FDF9F156F5/content.rtf` while leaving readable authority intact
- direct archive inspection shows no `Files/Data/A5101A53-7D7B-425D-82F0-A2FDF9F156F5/content.rtf` entry in this packet
- the `Files/Data/A5101A53-7D7B-425D-82F0-A2FDF9F156F5/` directory itself is retained with a `synopsis.txt` sidecar, so the degraded condition is a present data directory with a missing `content.rtf` body rather than an absent directory
- the corresponding clean sibling packet retains `Files/Data/A5101A53-7D7B-425D-82F0-A2FDF9F156F5/content.rtf`
- readable authority plus package shape therefore do not guarantee complete text-body availability

## Not suitable for

- support proof
- parser readiness claims
- `.scrivx` sufficiency proof
- deterministic mapping proof
- compatibility claims

## Known limitations

- one missing-content case does not establish broad irregular coverage
- same-source linkage is observational evidence, not admission proof
- this packet does not prove safe unavailable-vs-denied status semantics
- this packet does not resolve manuscript inclusion or exclusion rules
- this packet does not resolve compatibility spread

Evidence only. Not proof of general Scrivener support.
