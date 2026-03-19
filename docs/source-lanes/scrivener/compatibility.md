# Scrivener Compatibility And Risk

## Status

Preliminary risk capture only.

## Current evidence

| Risk area | Current evidence | Status |
| --- | --- | --- |
| platform packaging difference | official manuals indicate Windows shows the project as a folder while macOS hides the same file-level structure in a bundle | partially understood |
| partial copy risk | official manuals indicate the entire `.scriv` folder must be copied, not the `.scrivx` file alone | supported |
| version drift | no local fixtures available across versions | unresolved |
| unsupported project states | no local fixtures available | unresolved |
| incomplete project copies | official manual guidance implies incomplete copies are invalid, but failure posture is not yet fixture-proven | partially understood |
| external file references | no local fixtures available | unresolved |
| corrupted project structures | no local fixtures available | unresolved |
| content storage variability | no local fixtures available | unresolved |

Official references:

- https://www.literatureandlatte.com/docs/Scrivener_Manual-Win.pdf
- https://www.literatureandlatte.com/docs/Scrivener_Manual-Mac.pdf

## Known bounded risks

1. Cross-platform appearance differences can tempt implementation toward platform-specific assumptions.
2. Partial project copies are likely to produce misleading structural truth if not fail-closed immediately.
3. Version drift cannot be characterized honestly without real fixtures from more than one project generation.

## Unknowns that block implementation

- whether older and newer projects differ materially in internal authority layout
- whether Windows-created and macOS-created projects differ in ways that affect bounded admission
- whether cloud-synced or partially materialized projects introduce distinctive failure states
- whether content may live outside the default local project container

## Compatibility judgment

Compatibility risk remains too under-evidenced for implementation authorization.
