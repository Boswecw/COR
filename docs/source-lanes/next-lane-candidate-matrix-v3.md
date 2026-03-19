# Post-Slice-9 Next-Lane Candidate Matrix

Status: governance-only selection artifact

## 1. Purpose

This matrix re-evaluates the next bounded Cortex expansion target after Slice 9 delivered governed EPUB admission.

This is a governance selection artifact, not an implementation plan.

It exists to answer one narrow question:

Which candidate should become the next planning target after EPUB, while preserving Cortex's constitutional boundaries?

Candidates in scope:

- `HTML`
- special-track `Scrivener`

Out of scope:

- implementation work
- parser experimentation
- generic document or package abstraction
- implicit momentum from the EPUB slice

## 2. Locked evaluation criteria

All candidates are judged against the same constitutional pressures:

1. Boundary fit
   Does the candidate admit a bounded syntax-first or project-source lane without pulling Cortex toward platform behavior?

2. Authority clarity
   Is there a stable source of structural truth that Cortex can read without inventing meaning?

3. Extraction honesty
   Can a useful ready/denied/unavailable contract be defined without best-effort ambiguity?

4. Drift pressure
   How strongly does the candidate try to drag Cortex toward rendering, workflow, editorial, or application semantics?

5. Track-fit honesty
   Is the candidate honestly a routine source-lane continuation, or does it require an explicit track opening with different governance?

6. Strategic leverage
   Does the candidate create meaningful value for the wider Forge ecosystem without changing what Cortex fundamentally is?

7. Fixture-first feasibility
   Can the candidate be admitted through deterministic fixtures and invariant coverage rather than convenience handling?

## 3. Candidate assessment

### A. HTML

Why it is attractive:

- it is the remaining obvious routine file-lane candidate after EPUB
- explicit structural markers can exist in local markup
- input identity is straightforward (`.html`, `.htm`)

Primary pressure:

HTML remains a rendering-oriented, browser-adjacent format whose practical meaning is often entangled with layout, styling, boilerplate, scripting, linked assets, and page-specific conventions.

Advantages:

- explicit markup can expose headings, paragraphs, and lists
- single-file handling is simpler than composite publication packages
- routine lane framing is conceptually simpler than opening a project-source track

Major risks:

- highest browser and rendering drift pressure in the remaining set
- strong temptation to strip boilerplate or infer “real content”
- pressure to admit malformed markup recovery, scripts, metadata variants, and linked assets
- high risk of Cortex drifting toward web-content or document-platform behavior

Governance judgment:

HTML remains viable only through a highly skeptical governance pass, but it is still the highest drift-pressure candidate. Selecting it next would mean confronting browser pressure directly.

### B. Special-track Scrivener

Why it is attractive:

- it now offers higher strategic leverage than HTML in the broader Forge ecosystem
- it aligns with manuscript and project-source ingestion more directly than another routine document-family increment
- after EPUB, a deliberate project-source opening is more meaningful than forcing routine lane momentum into HTML

Primary pressure:

Scrivener is not a routine format peer. It is a project-source environment with hierarchy, binder identity, manuscript segmentation, and application-defined project conventions.

Advantages:

- materially higher ecosystem value if handled read-only and fail-closed
- authority surfaces can be bounded around project index truth, binder truth, and admitted textual item truth
- creates a cleaner strategic step than drifting from EPUB into browser-shaped HTML behavior

Major risks:

- opens a new special-track governance mode rather than routine lane continuation
- invites workflow, editorial, compile, and project-management drift if framed casually
- requires stricter anti-drift doctrine than ordinary file-family lanes
- likely needs a more explicit planning boundary before implementation can begin safely

Governance judgment:

Scrivener should be selected only if Cortex intentionally opens the special-track project-source path next. If that choice is made explicitly, it is now the more valuable and cleaner next planning move than HTML.

## 4. Comparative summary

| Candidate | Boundary fit | Drift pressure | Complexity type | Track-fit honesty | Strategic leverage | Recommended posture |
| --- | --- | --- | --- | --- | --- | --- |
| HTML | weak-to-moderate | highest | rendering and web interpretation | high as routine, weak constitutionally | moderate | defer unless deliberately confronting browser drift |
| Scrivener | conditional but stronger with explicit special-track framing | high if framed casually, moderate if tightly governed | project-source and binder authority | requires explicit special-track opening | high | select as next planning target only through project-source governance |

## 5. Selection recommendation

### Recommended next target

Special-track `Scrivener` should be selected as the next planning target after Slice 9.

### Reason

Scrivener presents the best balance of:

- strategic leverage
- explicit post-EPUB governance reset
- lower browser/rendering drift pressure than HTML
- clearer intentionality about opening a new project-source track instead of pretending routine parser momentum should continue

### Explicit non-selection statements

- HTML is deferred because it remains the highest browser and rendering drift-pressure candidate in the post-EPUB state.
- Scrivener is selected only as a planning target, not as authorized implementation and not as a routine document-family peer.
- This selection does not authorize generic package, project, workflow, or application-host behavior.
