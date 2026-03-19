# Scrivener Content Mapping

## Status

Unproven locally.

## What is currently known

Current evidence supports only these bounded claims:

- the project is a `.scriv` container rather than a standalone single file
- the top-level `.scrivx` file is part of the control surface needed to open the project
- the `.scrivx` file alone does not carry the full project state because the entire `.scriv` folder is required

## What is not yet proven

The following mapping questions remain unresolved:

- how binder nodes map to physical content files
- whether node identifiers are stable across realistic projects
- whether one node can reference multiple content bodies
- whether content storage is always RTF or only commonly RTF
- how nodes with no body content are represented
- how missing content files surface relative to intact binder entries

## Required proof table

| Mapping question | Current status | Evidence needed before implementation |
| --- | --- | --- |
| node identifier source | unresolved | local fixture inspection of project index and stored content |
| binder-to-body path convention | unresolved | local fixture inspection across at least two project variants |
| missing-body behavior | unresolved | irregular or incomplete fixture |
| duplicate-reference risk | unresolved | local fixture inspection of reordered or nested projects |
| content storage format | unresolved | local fixture inspection of content-bearing nodes |

## Current judgment

Content mapping proof has not been achieved.

No deterministic content locator should be designed yet.

## Gate effect

Implementation remains blocked pending local fixture evidence.
