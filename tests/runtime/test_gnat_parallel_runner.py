from __future__ import annotations

import copy
import tempfile
import unittest
from pathlib import Path
from types import SimpleNamespace
from unittest.mock import patch

from cortex_runtime.gnats import (
    FaLocalCapabilityState,
    GnatSourceInput,
    plan_gnat_run,
    reconcile_receipts,
    run_parallel_gnat_plan,
)
from cortex_runtime.gnats.registry import worker_for_type
from tests.runtime.runtime_test_support import ROOT, assert_schema_valid


GNAT_FIXTURE_DIR = ROOT / "tests/runtime/fixtures/gnats/text-batch-small"
MARKDOWN_FIXTURE = GNAT_FIXTURE_DIR / "chapter-01.md"
TEXT_FIXTURE = GNAT_FIXTURE_DIR / "note-plain.txt"
EMPTY_TEXT_FIXTURE = ROOT / "tests/runtime/fixtures/sample-empty.txt"


def ready_fa_local_state(max_concurrency: int = 4) -> FaLocalCapabilityState:
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


def write_temp_sources(tmpdir: str, count: int) -> list[GnatSourceInput]:
    sources: list[GnatSourceInput] = []
    for index in range(count):
        suffix = ".md" if index % 2 == 0 else ".txt"
        media_type = "text/markdown" if suffix == ".md" else "text/plain"
        path = Path(tmpdir) / f"source-{index:02d}{suffix}"
        path.write_text(
            f"# Source {index}\n\nThis is bounded GNAT fixture line {index}.\n",
            encoding="utf-8",
        )
        sources.append(GnatSourceInput(path, media_type=media_type, source_ref=f"source-{index:02d}"))
    return sources


class GnatParallelRunnerRuntimeTests(unittest.TestCase):
    def test_two_worker_parallel_run_emits_ready_summary(self) -> None:
        plan = plan_gnat_run(
            [
                GnatSourceInput(MARKDOWN_FIXTURE, media_type="text/markdown", source_ref="chapter-01"),
                GnatSourceInput(TEXT_FIXTURE, media_type="text/plain", source_ref="note-plain"),
            ],
            request_id="gnat-parallel-two",
            requested_concurrency=2,
            max_concurrency=2,
        )

        result = run_parallel_gnat_plan(plan, ready_fa_local_state(max_concurrency=2))

        self.assertEqual(result.negotiation.state, "fa_local_dispatch_ready")
        self.assertEqual(len(result.receipts), 2)
        self.assertEqual(result.summary["run_state"], "ready")
        self.assertEqual(result.summary["concurrency_used"], 2)
        self.assertFalse(result.summary["fallback_used"])
        for receipt in result.receipts:
            self.assertEqual(receipt["state"], "complete")
            assert_schema_valid(self, receipt, schema_name="gnat-worker-receipt.schema.json")
        assert_schema_valid(self, result.summary, schema_name="gnat-run-summary.schema.json")

    def test_four_worker_parallel_run_uses_admitted_concurrency(self) -> None:
        with tempfile.TemporaryDirectory() as tmpdir:
            plan = plan_gnat_run(
                write_temp_sources(tmpdir, 4),
                request_id="gnat-parallel-four",
                requested_concurrency=4,
                max_concurrency=4,
            )
            result = run_parallel_gnat_plan(plan, ready_fa_local_state(max_concurrency=4))

        self.assertEqual(result.summary["run_state"], "ready")
        self.assertEqual(result.summary["completed_count"], 4)
        self.assertEqual(result.summary["concurrency_used"], 4)

    def test_eight_worker_parallel_run_respects_hard_cap(self) -> None:
        with tempfile.TemporaryDirectory() as tmpdir:
            plan = plan_gnat_run(
                write_temp_sources(tmpdir, 8),
                request_id="gnat-parallel-eight",
                requested_concurrency=99,
                max_concurrency=99,
            )
            result = run_parallel_gnat_plan(plan, ready_fa_local_state(max_concurrency=99))

        self.assertEqual(plan.requested_concurrency, 8)
        self.assertEqual(plan.max_concurrency, 8)
        self.assertEqual(result.summary["run_state"], "ready")
        self.assertEqual(result.summary["concurrency_used"], 8)

    def test_parallel_serial_fallback_uses_serial_summary_when_fa_local_unavailable(self) -> None:
        plan = plan_gnat_run([MARKDOWN_FIXTURE], request_id="gnat-parallel-fallback")

        result = run_parallel_gnat_plan(plan, FaLocalCapabilityState(fa_local_state="unavailable"))

        self.assertEqual(result.negotiation.state, "serial_fallback")
        self.assertEqual(result.summary["run_state"], "ready")
        self.assertEqual(result.summary["concurrency_used"], 1)
        self.assertTrue(result.summary["fallback_used"])

    def test_worker_crash_produces_visible_partial_success(self) -> None:
        original_worker_for_type = worker_for_type

        def patched_worker_for_type(worker_type: str) -> object:
            if worker_type == "plain_text_syntax":
                return SimpleNamespace(run=lambda _shard: (_ for _ in ()).throw(RuntimeError("boom")))
            return original_worker_for_type(worker_type)

        plan = plan_gnat_run(
            [MARKDOWN_FIXTURE, TEXT_FIXTURE],
            request_id="gnat-parallel-crash",
            requested_concurrency=2,
            max_concurrency=2,
        )

        with patch("cortex_runtime.gnats.parallel_runner.worker_for_type", patched_worker_for_type):
            result = run_parallel_gnat_plan(plan, ready_fa_local_state(max_concurrency=2))

        self.assertEqual(result.summary["run_state"], "partial_success")
        self.assertEqual(result.summary["completed_count"], 1)
        self.assertEqual(result.summary["failed_count"], 1)
        self.assertEqual(
            sorted(receipt["state"] for receipt in result.receipts),
            ["complete", "failed"],
        )

    def test_infrastructure_exception_is_retried_once(self) -> None:
        original_worker_for_type = worker_for_type
        calls = {"count": 0}

        def flaky_worker(shard: object) -> dict[str, object]:
            calls["count"] += 1
            if calls["count"] == 1:
                raise RuntimeError("temporary worker crash")
            return dict(original_worker_for_type("plain_text_syntax").run(shard))

        def patched_worker_for_type(worker_type: str) -> object:
            if worker_type == "plain_text_syntax":
                return SimpleNamespace(run=flaky_worker)
            return original_worker_for_type(worker_type)

        plan = plan_gnat_run(
            [GnatSourceInput(TEXT_FIXTURE, media_type="text/plain", source_ref="note-plain")],
            request_id="gnat-parallel-retry",
        )

        with patch("cortex_runtime.gnats.parallel_runner.worker_for_type", patched_worker_for_type):
            result = run_parallel_gnat_plan(plan, ready_fa_local_state(max_concurrency=1))

        self.assertEqual(calls["count"], 2)
        self.assertEqual(result.summary["run_state"], "ready")

    def test_deterministic_denial_is_not_retried(self) -> None:
        original_worker_for_type = worker_for_type
        calls = {"count": 0}

        def counting_worker(shard: object) -> dict[str, object]:
            calls["count"] += 1
            return dict(original_worker_for_type("plain_text_syntax").run(shard))

        plan = plan_gnat_run(
            [GnatSourceInput(EMPTY_TEXT_FIXTURE, media_type="text/plain", source_ref="empty-text")],
            request_id="gnat-parallel-no-deterministic-retry",
        )

        with patch(
            "cortex_runtime.gnats.parallel_runner.worker_for_type",
            lambda _worker_type: SimpleNamespace(run=counting_worker),
        ):
            result = run_parallel_gnat_plan(
                plan,
                ready_fa_local_state(max_concurrency=1),
                retry_infrastructure_failures=3,
            )

        self.assertEqual(calls["count"], 1)
        self.assertEqual(result.receipts[0]["state"], "denied")
        self.assertEqual(result.summary["run_state"], "failed")

    def test_parallel_stale_source_is_reported(self) -> None:
        with tempfile.TemporaryDirectory() as tmpdir:
            source = Path(tmpdir) / "stale-note.txt"
            source.write_text("before planning\n", encoding="utf-8")
            plan = plan_gnat_run(
                [GnatSourceInput(source, media_type="text/plain")],
                request_id="gnat-parallel-stale",
            )
            source.write_text("after planning\n", encoding="utf-8")
            result = run_parallel_gnat_plan(plan, ready_fa_local_state(max_concurrency=1))

        self.assertEqual(result.receipts[0]["state"], "stale")
        self.assertEqual(result.summary["run_state"], "stale")
        self.assertEqual(result.summary["stale_count"], 1)

    def test_parallel_cancellation_produces_cancelled_summary(self) -> None:
        plan = plan_gnat_run(
            [MARKDOWN_FIXTURE, TEXT_FIXTURE],
            request_id="gnat-parallel-cancel",
            requested_concurrency=2,
            max_concurrency=2,
        )

        result = run_parallel_gnat_plan(
            plan,
            ready_fa_local_state(max_concurrency=2),
            cancel_requested=lambda: True,
        )

        self.assertEqual(result.summary["run_state"], "cancelled")
        self.assertEqual(result.summary["cancelled_count"], 2)
        self.assertTrue(all(receipt["state"] == "cancelled" for receipt in result.receipts))

    def test_complete_receipt_exceeding_shard_deadline_is_timed_out(self) -> None:
        original_worker_for_type = worker_for_type

        def slow_complete_worker(shard: object) -> dict[str, object]:
            receipt = dict(original_worker_for_type("plain_text_syntax").run(shard))
            receipt["duration_ms"] = 100
            return receipt

        plan = plan_gnat_run(
            [GnatSourceInput(TEXT_FIXTURE, media_type="text/plain", source_ref="note-plain")],
            request_id="gnat-parallel-deadline",
            deadline_ms=50,
        )

        with patch(
            "cortex_runtime.gnats.parallel_runner.worker_for_type",
            lambda _worker_type: SimpleNamespace(run=slow_complete_worker),
        ):
            result = run_parallel_gnat_plan(plan, ready_fa_local_state(max_concurrency=1))

        self.assertEqual(result.receipts[0]["state"], "timed_out")
        self.assertEqual(result.receipts[0]["error_reason_code"], "deadline_exceeded")
        self.assertEqual(result.summary["run_state"], "failed")

    def test_parallel_reconciliation_is_completion_order_independent(self) -> None:
        plan = plan_gnat_run([MARKDOWN_FIXTURE, TEXT_FIXTURE], request_id="gnat-parallel-order")
        result = run_parallel_gnat_plan(plan, ready_fa_local_state(max_concurrency=2))

        forward = reconcile_receipts(plan, list(result.receipts), concurrency_used=2, fallback_used=False)
        reverse = reconcile_receipts(plan, list(reversed(result.receipts)), concurrency_used=2, fallback_used=False)

        self.assertEqual(forward, reverse)

    def test_parallel_reconciliation_rejects_duplicate_and_missing_receipts(self) -> None:
        plan = plan_gnat_run([MARKDOWN_FIXTURE, TEXT_FIXTURE], request_id="gnat-parallel-reconcile")
        result = run_parallel_gnat_plan(plan, ready_fa_local_state(max_concurrency=2))

        duplicate_summary = reconcile_receipts(
            plan,
            [*result.receipts, copy.deepcopy(result.receipts[0])],
            concurrency_used=2,
            fallback_used=False,
        )
        missing_summary = reconcile_receipts(
            plan,
            [result.receipts[0]],
            concurrency_used=2,
            fallback_used=False,
        )

        self.assertEqual(duplicate_summary["run_state"], "partial_success")
        self.assertTrue(
            any(item["reason_code"] == "duplicate_shard" for item in duplicate_summary["rejected_receipts"])
        )
        self.assertEqual(missing_summary["run_state"], "partial_success")
        self.assertEqual(missing_summary["missing_count"], 1)


if __name__ == "__main__":
    unittest.main()
