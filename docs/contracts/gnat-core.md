# GNAT Core

Package: `gnat_core`

Version: `0.1.0`

Phase 10 extracts a small scheduling-neutral GNAT kernel from the Cortex GNAT
implementation.

The core is allowed to contain:

- canonical JSON hashing;
- redacted source path token generation;
- bounded and effective concurrency calculations;
- run-state derivation from neutral lifecycle counts;
- cache-key construction from a supplied identity contract;
- small protocol interfaces for cancellation and receipt envelopes.

The core is not allowed to contain:

- Cortex source-lane parsers;
- Cortex worker implementations;
- FA-Local concrete dispatch logic;
- DF-Local concrete storage;
- NeuronForge prompts or candidate generation;
- AuthorForge manuscript policy;
- application UI, watcher, queue, or business policy.

Compatibility rule:

Existing Cortex symbols may remain as compatibility wrappers, but their generic
behavior should delegate to `gnat_core` where it is already stable and
domain-neutral.

Distribution rule:

`gnat_core` remains an in-repository local package until a second application
implementation proves the same interfaces. Wider packaging should happen only
after compatibility tests exist for every consumer.
