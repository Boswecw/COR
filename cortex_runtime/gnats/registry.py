from __future__ import annotations

from dataclasses import dataclass
from typing import Callable

from cortex_runtime.gnats.models import GnatShard


GnatWorker = Callable[[GnatShard], dict[str, object]]


@dataclass(frozen=True)
class GnatWorkerAdapter:
    worker_type: str
    media_type: str
    lane_id: str
    run: GnatWorker


class GnatWorkerUnavailable(LookupError):
    pass


def _registry() -> dict[str, GnatWorkerAdapter]:
    from cortex_runtime.gnats.workers.markdown_text import run_markdown_text_worker
    from cortex_runtime.gnats.workers.pdf_text import run_pdf_text_worker
    from cortex_runtime.gnats.workers.plain_text import run_plain_text_worker

    return {
        "markdown_syntax": GnatWorkerAdapter(
            worker_type="markdown_syntax",
            media_type="text/markdown",
            lane_id="local_file_markdown",
            run=run_markdown_text_worker,
        ),
        "plain_text_syntax": GnatWorkerAdapter(
            worker_type="plain_text_syntax",
            media_type="text/plain",
            lane_id="local_file_plain_text",
            run=run_plain_text_worker,
        ),
        "pdf_text_syntax": GnatWorkerAdapter(
            worker_type="pdf_text_syntax",
            media_type="application/pdf",
            lane_id="local_file_pdf_text",
            run=run_pdf_text_worker,
        ),
    }


def admitted_worker_types() -> list[str]:
    return sorted(_registry())


def worker_for_type(worker_type: str) -> GnatWorkerAdapter:
    adapter = _registry().get(worker_type)
    if adapter is None:
        raise GnatWorkerUnavailable(f"no Gnat worker registered for {worker_type}")
    return adapter
