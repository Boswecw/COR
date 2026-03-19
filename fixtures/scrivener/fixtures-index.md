# Scrivener Fixtures Index

| Fixture id | Path | Class | Platform | Version | Sanitization status | Key authority value | Key mapping value | Major unresolveds | Intended use | Status |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| `scriv-sanitized-fixture-v1` | `fixtures/scrivener/ambiguous/scriv-sanitized-fixture-v1/` | ambiguous | unknown | unknown | sanitized derivative with extracted project and retained source archive | extracted fixture shows one top-level `.scriv` container with a sibling `.scrivx` control file | extracted fixture shows `Files/Data/<uuid>/content.rtf`, `notes.rtf`, `synopsis.txt`, and `content.styles` artifacts | ambiguous packet; manuscript vs research boundary not yet proven from this fixture; `.scrivx` sufficiency unresolved; binder-to-body mapping not yet proven; sanitization may hide edge conditions | authority observation; limited mapping observation; governance-only reference; not negative-case support | accepted with restrictions |
| `faith-in-a-firestorm-sanitized-v1` | `fixtures/scrivener/positive/faith-in-a-firestorm-sanitized-v1/` | positive | Windows | `Creator="SCRWIN-3.1.5.1"`, project `Version="2.0"` | sanitized derivative with extracted project and locally packaged sanitized archive; same raw source lineage as the canonical sanitized-corrupted negative packet | readable sanitized `.scrivx` shows `DraftFolder`, `ResearchFolder`, `TrashFolder`, `TemplateFolderUUID`, and `BookmarksFolderUUID` surfaces | multiple `BinderItem UUID` values are mirrored under `Files/Data/<UUID>/...`, including `323E42B8-881A-4A72-938C-1D683D78D8DF`, `B23591A4-8EC3-4E1F-B242-ECCA0207561C`, `209B8706-36B7-4E9F-BDC7-5B1E918EACD0`, and `972FDF2E-6367-49D9-9EB5-8197175E526A` | sanitized derivative only; `.scrivx` sufficiency unresolved; deterministic mapping unproven; manuscript inclusion and exclusion rules unresolved; no compat coverage | authority observation; same-source clean baseline comparison; manuscript/research boundary observation; governance-only reference | accepted with restrictions |
| `scriv-mixed-structure-sanitized-v1` | `fixtures/scrivener/positive/scriv-mixed-structure-sanitized-v1/` | positive | Windows | `Creator="SCRWIN-3.1.6.0"`, project `Version="2.0"` | sanitized derivative with extracted project and retained source archive | extracted fixture shows top-level `.scriv` + `.scrivx` plus explicit `DraftFolder`, `ResearchFolder`, `TrashFolder`, `TemplateFolderUUID`, and `BookmarksFolderUUID` surfaces in `.scrivx` | extracted fixture shows multiple `BinderItem UUID` values mirrored under `Files/Data/<UUID>/...`, including content, notes, synopsis, and style sidecars | sanitized derivative; research asset payloads were removed; `.scrivx` sufficiency unresolved; deterministic mapping across all item types unproven; negative coverage remains narrow; no compat coverage | authority observation; binder-to-content mapping observation; manuscript/research boundary observation; governance-only reference | accepted with restrictions |
| `faith-in-a-firestorm-sanitized-corrupt-scrivx-negative-fixture` | `fixtures/scrivener/negative/faith-in-a-firestorm-sanitized-corrupt-scrivx-negative-fixture/` | negative | Windows | `Creator="SCRWIN-3.1.5.1"`, project `Version="2.0"` | sanitized derivative lineage documented in packet; intentional `.scrivx` corruption applied only after sanitization; extracted project plus retained source archive | extracted sanitized-derived negative shows package-shaped `.scriv` with top-level `.scrivx`; direct local inspection shows sanitized metadata plus malformed `<Binder><BROKEN` after the binder start, and the packet-local corruption note documents removal of the closing `</ScrivenerProject>` tag | extracted sanitized-derived negative preserves `Files/Data/<UUID>/...` structure and ordinary project auxiliaries, which strengthens the fail-closed claim that package resemblance alone is insufficient when readable authority is absent | narrow malformed-authority case only; same-source clean baseline now exists, but this remains negative evidence only; no compat coverage; `.scrivx` sufficiency unresolved; deterministic mapping still unproven | malformed-authority observation; fail-closed denial evidence; governance-only reference | accepted with restrictions |
| `faith-in-a-firestorm-corrupt-scrivx-negative-fixture` | `fixtures/scrivener/negative/faith-in-a-firestorm-corrupt-scrivx-negative-fixture/` | negative | Windows-like metadata | corrupted `.scrivx` header still shows `Creator="SCRWIN-3.1.5.1"`, project `Version="2.0"` | raw-derived corrupted copy; not sanitized; retained only as superseded provenance history after sanitized-derived replacement | legacy raw-derived packet records the earlier provenance posture but is no longer used as the canonical malformed-authority packet | none beyond provenance audit history; excluded from active authority judgment | non-canonical audit history only; do not use for support, authority sufficiency, or gate advancement | provenance audit only | superseded / non-canonical |

Current active evidence packets:

- `fixtures/scrivener/ambiguous/scriv-sanitized-fixture-v1/`
- `fixtures/scrivener/negative/faith-in-a-firestorm-sanitized-corrupt-scrivx-negative-fixture/`
- `fixtures/scrivener/positive/faith-in-a-firestorm-sanitized-v1/`
- `fixtures/scrivener/positive/scriv-mixed-structure-sanitized-v1/`

Retained superseded packet:

- `fixtures/scrivener/negative/faith-in-a-firestorm-corrupt-scrivx-negative-fixture/`

Packet contents:

- `fixtures/scrivener/ambiguous/scriv-sanitized-fixture-v1/README.md`
- `fixtures/scrivener/ambiguous/scriv-sanitized-fixture-v1/sanitization-report.md`
- `fixtures/scrivener/ambiguous/scriv-sanitized-fixture-v1/scriv-sanitized-fixture-v1.scriv/`
- `fixtures/scrivener/ambiguous/scriv-sanitized-fixture-v1/scriv-sanitized-fixture-v1.zip`
- `fixtures/scrivener/negative/faith-in-a-firestorm-sanitized-corrupt-scrivx-negative-fixture/README.md`
- `fixtures/scrivener/negative/faith-in-a-firestorm-sanitized-corrupt-scrivx-negative-fixture/README-source-sanitized.md`
- `fixtures/scrivener/negative/faith-in-a-firestorm-sanitized-corrupt-scrivx-negative-fixture/sanitization-report-source.md`
- `fixtures/scrivener/negative/faith-in-a-firestorm-sanitized-corrupt-scrivx-negative-fixture/corruption-note.md`
- `fixtures/scrivener/negative/faith-in-a-firestorm-sanitized-corrupt-scrivx-negative-fixture/faith-in-a-firestorm-sanitized-v1.scriv/`
- `fixtures/scrivener/negative/faith-in-a-firestorm-sanitized-corrupt-scrivx-negative-fixture/faith-in-a-firestorm-sanitized-corrupt-scrivx-negative-fixture.zip`
- `fixtures/scrivener/negative/faith-in-a-firestorm-corrupt-scrivx-negative-fixture/README.md`
- `fixtures/scrivener/negative/faith-in-a-firestorm-corrupt-scrivx-negative-fixture/faith-in-a-firestorm-corrupt-scrivx-negative-fixture.zip`
- `fixtures/scrivener/positive/faith-in-a-firestorm-sanitized-v1/README.md`
- `fixtures/scrivener/positive/faith-in-a-firestorm-sanitized-v1/sanitization-report.md`
- `fixtures/scrivener/positive/faith-in-a-firestorm-sanitized-v1/faith-in-a-firestorm-sanitized-v1.scriv/`
- `fixtures/scrivener/positive/faith-in-a-firestorm-sanitized-v1/faith-in-a-firestorm-sanitized-v1.zip`
- `fixtures/scrivener/positive/scriv-mixed-structure-sanitized-v1/README.md`
- `fixtures/scrivener/positive/scriv-mixed-structure-sanitized-v1/sanitization-report.md`
- `fixtures/scrivener/positive/scriv-mixed-structure-sanitized-v1/scriv-mixed-structure-sanitized-v1.scriv/`
- `fixtures/scrivener/positive/scriv-mixed-structure-sanitized-v1/scriv-sanitized-fixture-v2.zip`

These index entries record local evidence fixtures, not admitted support.

The canonical fixture intake surface remains:

- `fixtures/scrivener/`

The canonical malformed-authority packet is now the sanitized-derived negative packet.
Its clean same-source baseline is now retained as `fixtures/scrivener/positive/faith-in-a-firestorm-sanitized-v1/`.
The earlier raw-derived negative packet is retained only as superseded provenance history.

Do not infer support, compatibility, or production-safe sanitization from these restricted entries.
