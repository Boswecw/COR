from __future__ import annotations

from cortex_runtime.gnats.models import GnatShard
from cortex_runtime.gnats.workers.text_common import run_text_worker


def run_docx_text_worker(shard: GnatShard) -> dict[str, object]:
    return run_text_worker(shard, expected_worker_type="docx_text_syntax")
