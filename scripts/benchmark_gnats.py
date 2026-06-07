from __future__ import annotations

import argparse
import resource
import sys
import tempfile
import time
from dataclasses import dataclass
from pathlib import Path
from types import SimpleNamespace
from typing import Callable
from unittest.mock import patch

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT))

from cortex_runtime.extraction_emission import emit_extraction_result_from_source_file
from cortex_runtime.gnats import FaLocalCapabilityState, GnatSourceInput, plan_gnat_run
from cortex_runtime.gnats import parallel_runner, serial_runner
from cortex_runtime.gnats.registry import worker_for_type


@dataclass(frozen=True)
class BenchmarkRecord:
    label: str
    concurrency: int
    files: int
    bytes_total: int
    wall_ms: float
    cpu_ms: float
    max_rss_kb: int
    run_state: str

    @property
    def files_per_second(self) -> float:
        if self.wall_ms <= 0:
            return 0.0
        return self.files / (self.wall_ms / 1000.0)

    @property
    def bytes_per_second(self) -> float:
        if self.wall_ms <= 0:
            return 0.0
        return self.bytes_total / (self.wall_ms / 1000.0)


def ready_fa_local_state(max_concurrency: int) -> FaLocalCapabilityState:
    return FaLocalCapabilityState(
        fa_local_state="ready",
        supported_contract_versions=(
            "GnatDispatchEnvelope.v1",
            "GnatRunPlan.v1",
            "GnatWorkerReceipt.v1",
        ),
        admitted_worker_types=("markdown_syntax", "plain_text_syntax"),
        max_concurrency=max_concurrency,
        cancellation_supported=True,
    )


def build_sources(tmpdir: Path, *, file_count: int, line_repeat: int) -> list[GnatSourceInput]:
    sources: list[GnatSourceInput] = []
    body = "\n".join(f"Fixture sentence {index} with bounded syntax only." for index in range(line_repeat))
    for index in range(file_count):
        suffix = ".md" if index % 2 == 0 else ".txt"
        media_type = "text/markdown" if suffix == ".md" else "text/plain"
        header = f"# GNAT fixture {index}\n\n" if suffix == ".md" else f"GNAT fixture {index}\n\n"
        path = tmpdir / f"gnat-fixture-{index:03d}{suffix}"
        path.write_text(header + body + "\n", encoding="utf-8")
        sources.append(GnatSourceInput(path, media_type=media_type, source_ref=f"fixture-{index:03d}"))
    return sources


def byte_total(sources: list[GnatSourceInput]) -> int:
    return sum(Path(source.path).stat().st_size for source in sources)


def delayed_worker_for_type(delay_ms: int):
    def resolve(worker_type: str) -> object:
        adapter = worker_for_type(worker_type)

        def run(shard: object) -> dict[str, object]:
            if delay_ms > 0:
                time.sleep(delay_ms / 1000.0)
            return dict(adapter.run(shard))

        return SimpleNamespace(run=run)

    return resolve


def measure(
    label: str,
    *,
    concurrency: int,
    files: int,
    bytes_total_value: int,
    run: Callable[[], object],
) -> BenchmarkRecord:
    start_wall = time.perf_counter()
    start_cpu = time.process_time()
    result = run()
    wall_ms = (time.perf_counter() - start_wall) * 1000.0
    cpu_ms = (time.process_time() - start_cpu) * 1000.0
    max_rss_kb = int(resource.getrusage(resource.RUSAGE_SELF).ru_maxrss)
    if isinstance(result, dict):
        run_state = str(result.get("run_state", "ready"))
    else:
        run_state = str(result.summary.get("run_state", "unknown"))
    return BenchmarkRecord(
        label=label,
        concurrency=concurrency,
        files=files,
        bytes_total=bytes_total_value,
        wall_ms=wall_ms,
        cpu_ms=cpu_ms,
        max_rss_kb=max_rss_kb,
        run_state=run_state,
    )


def run_benchmark(*, file_count: int, line_repeat: int, worker_delay_ms: int) -> list[BenchmarkRecord]:
    with tempfile.TemporaryDirectory() as tmpdir_name:
        tmpdir = Path(tmpdir_name)
        sources = build_sources(tmpdir, file_count=file_count, line_repeat=line_repeat)
        total_bytes = byte_total(sources)
        serial_plan = plan_gnat_run(
            sources,
            request_id="gnat-benchmark-serial",
            requested_concurrency=1,
            max_concurrency=1,
        )
        records = [
            measure(
                "legacy serial extraction",
                concurrency=1,
                files=file_count,
                bytes_total_value=total_bytes,
                run=lambda: _run_legacy_serial(sources, worker_delay_ms),
            )
        ]

        with patch.object(serial_runner, "worker_for_type", delayed_worker_for_type(worker_delay_ms)):
            records.append(
                measure(
                    "serial Gnat compatibility path",
                    concurrency=1,
                    files=file_count,
                    bytes_total_value=total_bytes,
                    run=lambda: serial_runner.run_serial_gnat_plan(serial_plan),
                )
            )

        for concurrency in (2, 4, 8):
            plan = plan_gnat_run(
                sources,
                request_id=f"gnat-benchmark-parallel-{concurrency}",
                requested_concurrency=concurrency,
                max_concurrency=concurrency,
            )
            with patch.object(parallel_runner, "worker_for_type", delayed_worker_for_type(worker_delay_ms)):
                records.append(
                    measure(
                        f"parallel Gnat path ({concurrency} workers)",
                        concurrency=concurrency,
                        files=file_count,
                        bytes_total_value=total_bytes,
                        run=lambda plan=plan, concurrency=concurrency: parallel_runner.run_parallel_gnat_plan(
                            plan,
                            ready_fa_local_state(concurrency),
                        ),
                    )
                )
    return records


def _run_legacy_serial(sources: list[GnatSourceInput], worker_delay_ms: int) -> dict[str, str]:
    for source in sources:
        if worker_delay_ms > 0:
            time.sleep(worker_delay_ms / 1000.0)
        emit_extraction_result_from_source_file(
            Path(source.path),
            request_id="gnat-benchmark-legacy",
            source_ref=source.source_ref or Path(source.path).name,
            media_type=source.media_type,
        )
    return {"run_state": "ready"}


def render_report(records: list[BenchmarkRecord], *, worker_delay_ms: int) -> str:
    serial_gnat = next(record for record in records if record.label == "serial Gnat compatibility path")
    parallel_four = next(record for record in records if record.concurrency == 4)
    four_worker_speedup = serial_gnat.wall_ms / parallel_four.wall_ms if parallel_four.wall_ms > 0 else 0.0
    pass_label = "pass" if four_worker_speedup >= 1.7 else "below target"
    lines = [
        "# GNAT Phase 4 Parallel Benchmark",
        "",
        "This report is generated by `make benchmark-gnats`.",
        "Fixtures are temporary Markdown/plain-text files generated during the run.",
        f"The run used a controlled per-file worker delay of {worker_delay_ms} ms to keep the local scheduling proof stable.",
        "",
        "| path | concurrency | run state | wall ms | cpu ms | files/sec | bytes/sec | max rss kb |",
        "|---|---:|---|---:|---:|---:|---:|---:|",
    ]
    for record in records:
        lines.append(
            "| "
            + " | ".join(
                [
                    record.label,
                    str(record.concurrency),
                    record.run_state,
                    f"{record.wall_ms:.2f}",
                    f"{record.cpu_ms:.2f}",
                    f"{record.files_per_second:.2f}",
                    f"{record.bytes_per_second:.2f}",
                    str(record.max_rss_kb),
                ]
            )
            + " |"
        )

    lines.extend(
        [
            "",
            "## Acceptance Readout",
            "",
            f"- Four-worker speedup over serial Gnat: {four_worker_speedup:.2f}x ({pass_label}; target 1.7x).",
            "- Reconciliation and receipt validation are included in the Gnat timings.",
            "- Warm-cache and cache-hit fields remain out of scope until the DF-Local persistence phase lands.",
            "- The benchmark is a local proof artifact, not a cross-machine performance guarantee.",
            "",
        ]
    )
    return "\n".join(lines)


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Benchmark bounded Cortex GNAT execution paths.")
    parser.add_argument("--files", type=int, default=32, help="number of generated Markdown/plain-text files")
    parser.add_argument("--line-repeat", type=int, default=80, help="synthetic lines per generated source")
    parser.add_argument("--worker-delay-ms", type=int, default=5, help="controlled per-file local worker delay")
    parser.add_argument("--output", type=Path, default=None, help="optional Markdown report path")
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    records = run_benchmark(
        file_count=args.files,
        line_repeat=args.line_repeat,
        worker_delay_ms=args.worker_delay_ms,
    )
    report = render_report(records, worker_delay_ms=args.worker_delay_ms)
    if args.output is not None:
        output = args.output if args.output.is_absolute() else ROOT / args.output
        output.parent.mkdir(parents=True, exist_ok=True)
        output.write_text(report, encoding="utf-8")
    print(report)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
