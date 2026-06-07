# Gnat Worker Receipt

Schema: `schemas/gnat-worker-receipt.schema.json`

`GnatWorkerReceipt.v1` is the evidence record for one worker attempt.
It records worker identity, source fingerprints before and after execution, completion state, timing, bounded finding counts, and either a bounded output or a bounded failure reason.

Complete receipts require an output reference or bounded output.
Non-complete receipts require an error reason code and an operator-visible summary.

Receipts must not include raw-content preview fields or workflow ownership fields.
