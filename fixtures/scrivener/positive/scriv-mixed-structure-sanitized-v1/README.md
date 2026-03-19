# scriv-mixed-structure-sanitized-v1

## Classification

- class: `positive`
- intake status: accepted with restrictions
- source provenance: sanitized derivative of a real `.scriv` project
- platform evidence: `.scrivx` `Creator="SCRWIN-3.1.6.0"`
- project metadata evidence: `.scrivx` `Version="2.0"`

## Packet contents

- `scriv-mixed-structure-sanitized-v1.scriv/`
- `scriv-sanitized-fixture-v2.zip`
- `sanitization-report.md`

## Intended use

- authority observation
- binder-to-content mapping observation
- manuscript/research boundary observation
- governance-only reference

## Direct local observations

- `.scrivx` contains `DraftFolder`, `ResearchFolder`, and `TrashFolder` binder surfaces
- `.scrivx` contains `TemplateFolderUUID` and `BookmarksFolderUUID`
- multiple `BinderItem UUID` values also appear under `Files/Data/<UUID>/...`, including `0A7EDD9F-9DE0-4CC9-9AC1-EE0E3769B6A8`, `323E42B8-881A-4A72-938C-1D683D78D8DF`, `B23591A4-8EC3-4E1F-B242-ECCA0207561C`, and `4908D479-188E-4232-AA7D-38FD12692DC5`
- retained payloads are text-side surfaces only; no `.pdf`, `.png`, or `.jpg` payload files were observed in the retained archive

## Not suitable for

- lane admission proof
- negative-case support
- compatibility claims
- general support claims

## Known unresolveds

- sanitization may obscure asset-side or mapping details
- `.scrivx` sufficiency remains unresolved
- deterministic mapping across all item types remains unproven
- no negative or irregular coverage exists yet
- no compat or version-drift coverage exists yet

This packet is evidence, not authorization.
