# Gnat Boundary Matrix

Gnats are bounded deterministic workers for syntax-level Cortex work.
They are not agents, semantic judges, workflow owners, or mutation surfaces.

| Concern | Cortex | FA-Local | NeuronForge Local | DF-Local | Operator-Local | Consuming app |
|---|---|---|---|---|---|---|
| Source eligibility | Owns | No | No | No | Displays | Requests |
| Source-lane admission | Owns | Reads | No | No | Displays | Requests |
| Batch plan construction | Owns | Receives | No | May persist later | Displays summary | Requests |
| Execution routing | No | Owns | No | No | Displays | No |
| Worker lifecycle | No in integrated mode | Owns | No | No | Displays | No |
| Serial fallback | Owns only when contract permits | May be unavailable | No | No | Displays | Requests |
| Syntax extraction | Owns deterministic implementation | Routes | No | May cache later | Displays bounded state | Consumes result |
| Semantic interpretation | No | No | Owns candidate generation later | No | Displays candidate labels later | Owns app truth |
| Receipt validation | Owns | No | No | May persist later | Displays state | Consumes summary |
| Durable record/cache | No | No | No | Owns later | Displays state | Reads through app |
| Operator controls | Reports state | Cancels/reroutes when integrated | No | No | Owns presentation | May request |
| Canonical meaning | No | No | No | No | No | Owns |

## GNAT-01 admitted lane

GNAT-01 admits only Markdown and plain-text syntax extraction:

- `text/markdown` with `markdown_syntax`;
- `text/plain` with `plain_text_syntax`.

All other existing source lanes remain on their current serial extraction path until separately admitted.

## Guardrails

- No NeuronForge calls in GNAT-01.
- No DF-Local cache or persistence claims in GNAT-01.
- No watcher activation.
- No source mutation.
- No raw-content diagnostics surface.
- No hidden Cortex scheduler in integrated mode.
