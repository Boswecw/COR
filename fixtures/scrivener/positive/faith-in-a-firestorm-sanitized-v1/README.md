# faith-in-a-firestorm-sanitized-v1

## Classification

- class: `positive`
- intake status: accepted with restrictions
- source provenance: sanitized derivative of the same raw source lineage used for `faith-in-a-firestorm-sanitized-corrupt-scrivx-negative-fixture`
- platform evidence: `.scrivx` `Creator="SCRWIN-3.1.5.1"`
- project metadata evidence: `.scrivx` `Version="2.0"`

## Packet contents

- `faith-in-a-firestorm-sanitized-v1.scriv/`
- `faith-in-a-firestorm-sanitized-v1.zip`
- `sanitization-report.md`

## Intended use

- authority observation
- same-source clean baseline comparison against the canonical sanitized-corrupted negative packet
- manuscript/research boundary observation
- candidate binder-to-content mapping observation
- governance-only reference

## Direct local observations

- `.scrivx` is readable and retains sanitized metadata fields such as `Identifier="SANITIZED"`, `Device="SANITIZED"`, `Modified="SANITIZED"`, and `ModID="SANITIZED"`
- `.scrivx` contains `DraftFolder`, `ResearchFolder`, and `TrashFolder` binder surfaces
- `.scrivx` contains `TemplateFolderUUID` and `BookmarksFolderUUID`
- multiple `BinderItem UUID` values are also present under `Files/Data/<UUID>/...`, including `323E42B8-881A-4A72-938C-1D683D78D8DF`, `B23591A4-8EC3-4E1F-B242-ECCA0207561C`, `209B8706-36B7-4E9F-BDC7-5B1E918EACD0`, and `972FDF2E-6367-49D9-9EB5-8197175E526A`
- the packet is the clean sibling baseline for `../faith-in-a-firestorm-sanitized-corrupt-scrivx-negative-fixture/`

## Not suitable for

- lane admission proof
- parser readiness claims
- deterministic mapping proof
- compatibility claims
- general support claims

## Known unresolveds

- sanitization may obscure authority or mapping details that existed in the raw source archive
- `.scrivx` sufficiency remains unresolved
- deterministic mapping across all item types remains unproven
- manuscript inclusion and non-manuscript exclusion rules remain unresolved
- this same-source baseline improves comparison discipline but does not clear the implementation gate

This packet is evidence, not authorization.
