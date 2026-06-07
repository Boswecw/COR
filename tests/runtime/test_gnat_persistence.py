from __future__ import annotations

import tempfile
import unittest
from pathlib import Path

from cortex_runtime.gnats import (
    FaLocalCapabilityState,
    GnatSourceInput,
    InMemoryGnatPersistenceStore,
    invalidate_changed_source_records,
    plan_gnat_run,
    run_parallel_gnat_plan_with_persistence,
)
from tests.runtime.runtime_test_support import assert_schema_valid


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


def write_sources(tmpdir: str) -> tuple[Path, Path]:
    left = Path(tmpdir) / "chapter.md"
    right = Path(tmpdir) / "note.txt"
    left.write_text("# Chapter\n\nStable markdown body.\n", encoding="utf-8")
    right.write_text("Stable plain text body.\n", encoding="utf-8")
    return left, right


def plan_for_sources(left: Path, right: Path, request_id: str):
    return plan_gnat_run(
        [
            GnatSourceInput(left, media_type="text/markdown", source_ref="chapter"),
            GnatSourceInput(right, media_type="text/plain", source_ref="note"),
        ],
        request_id=request_id,
        requested_concurrency=2,
        max_concurrency=2,
    )


class GnatPersistenceRuntimeTests(unittest.TestCase):
    def test_second_unchanged_run_reuses_exact_cache_with_fresh_receipts(self) -> None:
        with tempfile.TemporaryDirectory() as tmpdir:
            left, right = write_sources(tmpdir)
            store = InMemoryGnatPersistenceStore()
            first_plan = plan_for_sources(left, right, "gnat-persist-first")
            first = run_parallel_gnat_plan_with_persistence(first_plan, ready_fa_local_state(2), store)
            second_plan = plan_for_sources(left, right, "gnat-persist-second")
            second = run_parallel_gnat_plan_with_persistence(second_plan, ready_fa_local_state(2), store)

        self.assertEqual(first.persistence.cache_hits, 0)
        self.assertEqual(first.persistence.cache_misses, 2)
        self.assertEqual(second.persistence.cache_hits, 2)
        self.assertEqual(second.persistence.cache_misses, 0)
        self.assertEqual(second.summary["run_state"], "ready")
        self.assertEqual(second.summary["persistence"]["state"], "ready")
        self.assertTrue(all(receipt["run_id"] == second_plan.run_id for receipt in second.receipts))
        self.assertTrue(all(receipt["duration_ms"] == 0 for receipt in second.receipts))
        assert_schema_valid(self, second.summary, schema_name="gnat-run-summary.schema.json")

    def test_changed_source_invalidates_only_affected_cache_record(self) -> None:
        with tempfile.TemporaryDirectory() as tmpdir:
            left, right = write_sources(tmpdir)
            store = InMemoryGnatPersistenceStore()
            first_plan = plan_for_sources(left, right, "gnat-persist-change-first")
            run_parallel_gnat_plan_with_persistence(first_plan, ready_fa_local_state(2), store)

            right.write_text("Changed plain text body.\n", encoding="utf-8")
            second_plan = plan_for_sources(left, right, "gnat-persist-change-second")
            invalidated = invalidate_changed_source_records(
                store,
                previous_plan=first_plan,
                next_plan=second_plan,
            )
            second = run_parallel_gnat_plan_with_persistence(second_plan, ready_fa_local_state(2), store)

        self.assertEqual(invalidated, 1)
        self.assertEqual(second.persistence.cache_hits, 1)
        self.assertEqual(second.persistence.cache_misses, 1)
        self.assertEqual(second.summary["run_state"], "ready")

    def test_persistence_write_failure_degrades_without_changing_extraction_truth(self) -> None:
        with tempfile.TemporaryDirectory() as tmpdir:
            left, right = write_sources(tmpdir)
            plan = plan_for_sources(left, right, "gnat-persist-write-failure")
            store = InMemoryGnatPersistenceStore(fail_writes=True)
            result = run_parallel_gnat_plan_with_persistence(plan, ready_fa_local_state(2), store)

        self.assertEqual(result.summary["run_state"], "ready")
        self.assertEqual(result.persistence.state, "degraded")
        self.assertEqual(result.summary["persistence"]["state"], "degraded")
        self.assertEqual(result.summary["completed_count"], 2)

    def test_cache_read_failure_degrades_and_executes_without_cache(self) -> None:
        with tempfile.TemporaryDirectory() as tmpdir:
            left, right = write_sources(tmpdir)
            plan = plan_for_sources(left, right, "gnat-persist-read-failure")
            store = InMemoryGnatPersistenceStore(fail_reads=True)
            result = run_parallel_gnat_plan_with_persistence(plan, ready_fa_local_state(2), store)

        self.assertEqual(result.summary["run_state"], "ready")
        self.assertEqual(result.persistence.state, "degraded")
        self.assertEqual(result.persistence.cache_hits, 0)
        self.assertEqual(result.persistence.cache_misses, 2)


if __name__ == "__main__":
    unittest.main()
