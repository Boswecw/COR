# Post-Slice-8 Next-Lane Candidate Matrix

Status: governance-only selection artifact

## 1. Purpose

This matrix re-evaluates the next bounded Cortex expansion target after Slice 8 delivered governed ODT admission.

This is a governance selection artifact, not an implementation plan.

It exists to answer one narrow question:

Which candidate should become the next planning target after ODT, while preserving Cortex's constitutional boundaries?

Candidates in scope:

- `HTML`
- `EPUB`
- special-track `Scrivener`

Out of scope:

- implementation work
- schema expansion beyond governance truth
- parser experimentation
- generalized file-family ambitions

## 2. Locked evaluation criteria

All candidates are judged against the same constitutional pressures:

1. Boundary fit
   Does the format admit a bounded syntax-only lane without pulling Cortex toward semantic or platform behavior?

2. Authority clarity
   Is there a stable source of structural truth that Cortex can read without inventing meaning?

3. Extraction honesty
   Can a useful ready/unavailable/denied contract be defined without best-effort ambiguity?

4. Drift pressure
   How strongly does the format try to drag Cortex toward rendering, workflow, editorial, or application semantics?

5. Framework compatibility
   How naturally does the candidate fit the admitted source-lane framework already proven by `.md`, `.txt`, text-layer `.pdf`, `.docx`, `.rtf`, and `.odt`?

6. Fixture-first feasibility
   Can the lane be admitted through deterministic fixtures and invariant coverage rather than fuzzy real-world convenience handling?

## 3. Candidate assessment

### A. HTML

Why it is attractive:

- widely available source type
- can contain obvious structural markers such as headings, paragraphs, and lists
- could provide value for local web-page-derived content preparation

Primary pressure:

HTML is not merely a document container. It is a rendering-oriented, web-behavior-adjacent format whose practical meaning is often inseparable from browser interpretation, styling, DOM variation, embedded assets, and page-specific conventions.

Advantages:

- structure signals often exist explicitly in markup
- package complexity is lower than EPUB
- input identity is straightforward (`.html`, `.htm`)

Major risks:

- strong rendering drift pressure
- temptation to interpret layout or style as structure
- temptation to normalize malformed or noisy web markup too aggressively
- likely pressure to admit linked assets, scripts, metadata variants, and web-page semantics
- higher risk of Cortex drifting toward document-platform or web-content behavior

Governance judgment:

HTML remains the highest drift-pressure candidate in the current set. It is not disqualified in principle, but it should be selected only if Cortex intentionally wants to confront rendering and web pressure next and is prepared to encode unusually strict exclusions.

### B. EPUB

Why it is attractive:

- remains a document-family format rather than a general web artifact
- package boundaries are explicit
- structural extraction can be framed around bounded textual content rather than live rendering behavior
- best candidate for continued routine lane expansion after ODT

Primary pressure:

EPUB is a composite package format. It introduces package traversal, manifest and spine authority questions, and multi-document assembly pressure that are more complex than single-file lanes.

Advantages:

- still document-centered rather than browser-centered
- has identifiable package authority surfaces
- can likely be bounded around textual spine content, headings, paragraphs, lists, and limited metadata
- fits the existing source-lane philosophy better than HTML

Major risks:

- composite package complexity
- manifest and spine versus embedded-resource truth must be defined carefully
- risk of widening into media, cover, navigation, annotations, and presentation semantics
- pressure to reconstruct reading order beyond what package truth actually guarantees

Governance judgment:

EPUB is now the cleanest next routine lane candidate. It is more complex than ODT, but its risk is mainly bounded package complexity rather than browser or rendering drift. That makes it a better fit for disciplined source-lane expansion.

### C. Special-track Scrivener

Why it is attractive:

- high strategic value for authoring workflows in the broader Forge ecosystem
- strong potential relevance to manuscript and project-source ingestion
- could eventually create meaningful value beyond routine file-lane expansion

Primary pressure:

Scrivener is not a routine document-format peer. It is a project-source environment with hierarchy, binder structure, item identity, manuscript segmentation, and application-defined conventions.

Advantages:

- strategically important in the larger ecosystem
- authority surfaces can be defined if treated as a project-source lane
- rich manuscript structure may be available without semantic invention

Major risks:

- not a normal format lane
- invites project, workflow, and editorial drift if framed casually
- requires a stricter special-lane doctrine than routine file families
- likely requires additional governance artifacts before admission work can safely begin

Governance judgment:

Scrivener should remain special-track unless Cortex explicitly chooses to open the project-source path next. It should not be selected by momentum or treated as just another parser target.

## 4. Comparative summary

| Candidate | Boundary fit | Drift pressure | Complexity type | Routine lane fit | Recommended posture |
| --- | --- | --- | --- | --- | --- |
| HTML | weak-to-moderate | highest | rendering and web interpretation | low | defer unless deliberately confronting web drift |
| EPUB | strong | moderate | composite package and manifest-spine authority | high | select as next routine planning target |
| Scrivener | conditional | high if framed casually | project-source and binder authority | low as routine lane | keep special-track unless explicitly opening project-source path |

## 5. Selection recommendation

### Recommended next target

EPUB should be selected as the next planning target after Slice 8.

### Reason

EPUB presents the best balance of:

- constitutional fit
- bounded syntax-only extractability
- continued source-lane progression
- lower drift pressure than HTML
- less category confusion than Scrivener

### Explicit non-selection statements

- HTML is deferred because rendering and web drift pressure remains materially higher than the current lane framework should take on next.
- Scrivener is deferred because it remains a special project-source candidate and should only advance through an explicit project-source governance opening, not routine lane expansion.
