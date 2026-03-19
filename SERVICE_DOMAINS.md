# Cortex Service Domains

## Domain A - Intake

Purpose:

- admit eligible local content into Cortex under bounded contracts

May do:

- source eligibility checks
- normalization
- contract-scoped file observation
- intake status reporting

May not do:

- app meaning decisions
- broad sync assumptions
- uncontrolled background expansion

## Domain B - Extraction

Purpose:

- produce syntax-level structured content and metadata

May do:

- text extraction
- structural normalization
- metadata harvest
- provenance reporting
- incomplete or failed extraction signaling

May not do:

- summarization
- classification
- thematic labeling
- sentiment or implication judgments

## Domain C - Indexing Preparation

Purpose:

- prepare retrievable artifacts without becoming policy authority

May do:

- shaping of extraction outputs for retrieval infrastructure
- bounded index artifact generation
- freshness marking
- invalidation signal attachment

May not do:

- define final retrieval policy
- act as semantic memory authority
- imply permanent freshness

## Domain D - Retrieval Preparation

Purpose:

- assemble retrieval-oriented packages for app or NeuronForge Local consumption

May do:

- package structure
- governed retrieval-profile application
- bounded readiness signaling

May not do:

- ungoverned ranking doctrine
- hidden policy hardcoding

## Domain E - Handoff

Purpose:

- validate and package bounded downstream transfer

May do:

- envelope validation
- integrity and status annotation
- explicit denial or re-prep surfaces
- minimal bounded reverse signaling

May not do:

- downstream workflow orchestration
- execution ownership after transfer
- retry coordination outside contract

## Domain F - Operational Truth

Purpose:

- surface truthful service status without privacy collapse

May do:

- health summaries
- degraded, unavailable, denied, stale, and partial-success reporting
- major failure event surfaces
- bounded freshness indicators

May not do:

- content surveillance
- free-form content browsing in diagnostics
- raw preview surfaces by default
