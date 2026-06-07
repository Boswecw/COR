# Plan: Tighten the service-status unsupported-lane negative fixture

## Status

Plan only. No fixture, schema, or runtime change has been made yet.

## The issue

- File: `tests/contracts/fixtures/invalid/service-status-runtime-surface-unsupported-lane.json`
- Schema it is matched to: `schemas/service-status.schema.json`
- Offending field: `runtime_surface_summary.admitted_source_lanes[1]`
- Current value: `"local_file_epub"`

This is a negative fixture. Its job, per its filename, is to prove that
service-status truth rejects a source lane that lies outside the bounded
admitted set. The validator confirms it rejects, at exactly one location:

```
[rej]  service-status-runtime-surface-unsupported-lane.json -> service-status
       at [runtime_surface_summary.admitted_source_lanes.1]:
       'local_file_epub' is not one of ['local_file_markdown', 'local_file_plain_text',
       'local_file_pdf_text', 'local_file_docx_text', 'local_file_rtf_text',
       'local_file_odt_text', 'local_file_epub_text']
```

The defect: `local_file_epub` is a near-miss typo of the genuinely admitted
token `local_file_epub_text` (the `_text` suffix is missing). So the fixture's
name claims it tests an "unsupported lane," but what it actually tests is a
misspelling of a supported lane.

Three concrete consequences:

1. Semantic mismatch — it does not represent an ungoverned or foreign lane
   class; it represents a typo of an in-family, admitted lane.
2. Drift fragility — if the lane-naming convention ever dropped the `_text`
   suffix, `local_file_epub` would become a valid token and this fixture would
   stop being a negative. The `expect_invalid` guard would then flag it as
   "unexpectedly valid," so it fails loudly rather than silently, but the
   negative coverage would be gone and would need re-authoring.
3. Coverage gap — no fixture exercises a clearly foreign lane (wrong class or
   prefix), which is the doctrine-relevant case: Cortex is constitutionally
   local-only, and a status surface must reject lanes outside that boundary.

Scope check: this is the only near-miss among the 20 invalid fixtures. Every
other negative uses an unambiguously foreign token (`retry_exhausted`,
`queued_for_retry`, `workflow_id`, `raw_content_preview`, and so on), so the
fix is isolated to this one file.

## Invariants to preserve

1. Single-rooted negative — the fixture must still fail for exactly one
   documented reason (the enum on `admitted_source_lanes`), not pick up
   incidental errors.
2. Doctrine-aligned — the rejected token should represent a real boundary
   Cortex enforces, not a typo.
3. Drift-resilient — choose a token that can never plausibly be added to the
   admitted enum.
4. No churn to the matcher — keep the `service-status-` filename prefix so
   schema discovery is unaffected.

## Options

| Option | Change | Trade-off |
| --- | --- | --- |
| A. Foreign boundary-violating token (recommended) | Replace `local_file_epub` with `remote_file_text` | Strongest doctrine signal: Cortex is constitutionally local-only, so a `remote_*` lane can never be admitted, making it maximally drift-resilient. Minimal one-line change, same rejection mechanism. |
| B. Deferred-lane token | Replace with `local_file_html_text` | Reads as "reject the not-yet-admitted HTML lane." Honest, but HTML is a future candidate, so slightly less drift-proof than A. |
| C. Add a second fixture | Keep the near-miss as an explicit typo case and add a new foreign-lane fixture | Best coverage (tests both typo and foreign class), but adds maintenance and a fifth service-status invalid fixture; arguably more than needed. |

## Intended fix (Option A)

A single-token edit to the fixture:

```diff
   "admitted_source_lanes": [
     "local_file_markdown",
-    "local_file_epub"
+    "remote_file_text"
   ],
```

Rationale for `remote_file_text`: Cortex's charter is local-only, so a
`remote_*` lane can never be admitted to the enum, making this the most
drift-proof, doctrine-aligned representation of "a lane outside the bounded
set." It rejects through the same enum mechanism at the same field, so the
fixture stays single-rooted.

Also sharpen the now-slightly-inaccurate operator message for honesty:

```diff
-  "operator_visible_message": "Cortex claims an ungoverned source lane in service status.",
+  "operator_visible_message": "Cortex claims a non-local source lane in service status.",
```

Nothing else changes — no schema edits, no runtime edits, no valid-fixture
edits, no filename change (the `service-status-` prefix that drives schema
discovery is preserved).

## Verification

1. `python3 scripts/validate_schemas.py --verbose` — confirm this fixture still
   rejects at exactly `[runtime_surface_summary.admitted_source_lanes.1]` with
   an `is not one of [...]` message (proves it stays single-rooted and the
   rejection is still the enum guard).
2. `make validate` — still `VALIDATION PASSED (19 valid, 20 invalid, 7 schemas)`.
3. `make test-runtime` — still `OK` (no runtime test references this fixture's
   contents).
4. Commit to the working branch.

## Risk

Very low — invalid-fixture content only; no schema, runtime, or valid-fixture
impact, and the `--verbose` output gives immediate proof the rejection reason
is unchanged in kind.
