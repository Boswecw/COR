# FA-Local, NeuronForge, DF-Local, and Operator-Local Integration

## FA-Local

### Role

FA-Local is the execution authority for Gnat runs.

It should own:

- queueing;
- bounded concurrency;
- worker launch and termination;
- deadlines;
- cancellation;
- retry policy;
- resource-policy enforcement;
- execution-state events;
- infrastructure-failure evidence.

COR should send an immutable plan and validate the returned receipts.

### Capability negotiation

Before dispatch, COR asks FA-Local for:

```text
supported Gnat contract versions
supported worker types
maximum concurrency
cancellation support
receipt delivery mode
resource-policy profile
```

Mismatch causes serial fallback or denial according to the request contract.

### Reuse from Forge-Agents

Borrow concepts, not the heavyweight cloud implementation:

- registry;
- precondition wrapper;
- evidence on all invocations;
- authority context;
- cancellation;
- operator intervention;
- lifecycle state vocabulary.

Avoid cloud-provider and general autonomous-agent assumptions.

## DF-Local

### Role

DF-Local owns durable local storage substrate for:

```text
GnatRunPlan
GnatWorkerReceipt
GnatRunSummary
cache records
artifact references
operator decisions
```

COR remains authority for syntax-artifact validity; storage does not transfer meaning authority to DF-Local.

### Persistence behavior

- Store plans before dispatch where possible.
- Append receipts immutably.
- Store summary after reconciliation.
- Record persistence degradation separately from extraction result.
- Use retention and cleanup policies.
- Encrypt or protect local records according to the app security model.

## NeuronForge

### Role

NeuronForge is optional and downstream.

Allowed later:

- summarize structural findings;
- classify ambiguous manuscript structures;
- identify continuity candidates;
- explain code relationships;
- generate non-canonical suggestions.

Not allowed in GNAT-01:

- basic text parsing;
- word counts;
- headings/section extraction;
- source-lane eligibility;
- receipt reconciliation;
- truth promotion.

### Handoff

```text
COR syntax artifact
+ provenance and completeness
+ app request
→ NeuronForge candidate generation
→ app/operator review
```

NeuronForge results must never overwrite COR receipts.

## Operator-Local

### Required surface

- run state;
- source/shard counts;
- concurrency used;
- elapsed time;
- cancellation;
- serial fallback indicator;
- cache-reuse indicator;
- partial/stale/failure counts;
- bounded reason codes;
- persistence status.

### Prohibited surface

- unrestricted source browser;
- raw content previews by default;
- invisible watchers;
- automatic semantic acceptance;
- implied certainty for partial results.

## Failure matrix

| Failure | COR response |
|---|---|
| FA-Local unavailable | Serial fallback or deny, per contract |
| DF-Local unavailable | Continue bounded computation when safe; mark persistence degraded |
| NeuronForge unavailable | Syntax result remains valid; semantic feature unavailable |
| Operator-Local disconnected | Run may continue only according to approved background-execution policy |
| Worker receipt invalid | Reject receipt; partial/fail-closed summary |
| Source changes | Mark shard stale; do not accept output |
