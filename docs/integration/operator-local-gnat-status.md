# Operator-Local Gnat Status

Cortex exposes `GnatOperatorRunStatus.v1` as the bounded display/control contract
for Operator-Local.

Operator-Local may display:

- run state;
- source and shard counts;
- concurrency used;
- elapsed time;
- serial fallback indicator;
- cache reuse indicator;
- partial, stale, failed, cancelled, and missing counts;
- bounded failure reason codes;
- DF-Local persistence state;
- accessible controls for cancel, rerun failed shards, and stale-cache clearing.

Operator-Local must not display raw content previews, unrestricted source
browsing, hidden watcher state, or semantic acceptance claims from this contract.

Cancellation remains a control request against the approved execution surface.
This contract only declares operator-visible controls and labels; it does not make
Operator-Local the execution authority.
