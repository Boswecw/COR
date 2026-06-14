from __future__ import annotations

import tempfile
import unittest
from pathlib import Path

from cortex_runtime.gnats import (
    FaLocalCapabilityState,
    GnatSourceInput,
    InMemoryGnatPersistenceStore,
    build_operator_run_status,
    plan_gnat_run,
    run_parallel_gnat_plan,
    run_parallel_gnat_plan_with_persistence,
)
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


def _summary(run_state: str) -> dict:
    return {
        "run_id": f"gnat-status-{run_state}",
        "run_state": run_state,
        "expected_shards": 1,
        "completed_count": 1 if run_state == "ready" else 0,
        "failed_count": 1 if run_state in {"failed", "denied"} else 0,
        "stale_count": 1 if run_state == "stale" else 0,
        "cancelled_count": 1 if run_state == "cancelled" else 0,
        "missing_count": 0,
        "aggregate_timing": {"duration_ms": 7},
        "concurrency_used": 1,
        "fallback_used": False,
        "operator_visible_summary": f"State {run_state}",
    }


class GnatOperatorStatusRuntimeTests(unittest.TestCase):
    def test_parallel_status_is_schema_valid_and_redacted(self) -> None:
        plan = plan_gnat_run([MARKDOWN_FIXTURE, TEXT_FIXTURE], request_id="gnat-operator-parallel", requested_concurrency=2)
        result = run_parallel_gnat_plan(plan, ready_fa_local_state(2))

        status = build_operator_run_status(
            summary=result.summary,
            receipts=result.receipts,
            cancellation_available=True,
        )

        assert_schema_valid(self, status, schema_name="gnat-operator-run-status.schema.json")
        self.assertEqual(status["execution_mode"], "parallel")
        self.assertEqual(status["persistence_state"], "unavailable")
        self.assertNotIn("raw_content_preview", str(status))
        self.assertFalse(status["serial_fallback_used"])

    def test_cache_reuse_status_distinguishes_full_and_mixed_cache_modes(self) -> None:
        with tempfile.TemporaryDirectory() as tmpdir:
            left = Path(tmpdir) / "chapter.md"
            right = Path(tmpdir) / "note.txt"
            left.write_text("# Chapter\n\nStable.\n", encoding="utf-8")
            right.write_text("Stable note.\n", encoding="utf-8")
            sources = [
                GnatSourceInput(left, media_type="text/markdown", source_ref="chapter"),
                GnatSourceInput(right, media_type="text/plain", source_ref="note"),
            ]
            store = InMemoryGnatPersistenceStore()
            first_plan = plan_gnat_run(sources, request_id="gnat-operator-cache-first", requested_concurrency=2)
            run_parallel_gnat_plan_with_persistence(first_plan, ready_fa_local_state(2), store)
            second_plan = plan_gnat_run(sources, request_id="gnat-operator-cache-second", requested_concurrency=2)
            second = run_parallel_gnat_plan_with_persistence(second_plan, ready_fa_local_state(2), store)
            right.write_text("Changed note.\n", encoding="utf-8")
            mixed_plan = plan_gnat_run(sources, request_id="gnat-operator-cache-mixed", requested_concurrency=2)
            mixed = run_parallel_gnat_plan_with_persistence(mixed_plan, ready_fa_local_state(2), store)

        full_cache_status = build_operator_run_status(
            summary=second.summary,
            receipts=second.receipts,
            cancellation_available=False,
        )
        mixed_status = build_operator_run_status(
            summary=mixed.summary,
            receipts=mixed.receipts,
            cancellation_available=False,
        )

        self.assertEqual(full_cache_status["execution_mode"], "cache_reuse")
        self.assertEqual(full_cache_status["cache_reuse"]["hits"], 2)
        self.assertEqual(mixed_status["execution_mode"], "mixed_cache_parallel")
        self.assertEqual(mixed_status["cache_reuse"]["hits"], 1)
        self.assertEqual(mixed_status["cache_reuse"]["misses"], 1)

    def test_serial_fallback_status_is_distinguishable(self) -> None:
        plan = plan_gnat_run([MARKDOWN_FIXTURE], request_id="gnat-operator-fallback")
        result = run_parallel_gnat_plan(plan, FaLocalCapabilityState(fa_local_state="unavailable"))

        status = build_operator_run_status(
            summary=result.summary,
            receipts=result.receipts,
            cancellation_available=False,
        )

        self.assertEqual(status["execution_mode"], "serial_fallback")
        self.assertTrue(status["serial_fallback_used"])

    def test_failure_and_stale_statuses_expose_bounded_controls_and_reasons(self) -> None:
        failed_plan = plan_gnat_run(
            [GnatSourceInput(EMPTY_TEXT_FIXTURE, media_type="text/plain", source_ref="empty")],
            request_id="gnat-operator-failed",
        )
        failed = run_parallel_gnat_plan(failed_plan, ready_fa_local_state(1))

        with tempfile.TemporaryDirectory() as tmpdir:
            source = Path(tmpdir) / "stale.txt"
            source.write_text("before\n", encoding="utf-8")
            stale_plan = plan_gnat_run([GnatSourceInput(source, media_type="text/plain")], request_id="gnat-operator-stale")
            source.write_text("after\n", encoding="utf-8")
            stale = run_parallel_gnat_plan(stale_plan, ready_fa_local_state(1))

        failed_status = build_operator_run_status(
            summary=failed.summary,
            receipts=failed.receipts,
            cancellation_available=False,
        )
        stale_status = build_operator_run_status(
            summary=stale.summary,
            receipts=stale.receipts,
            cancellation_available=False,
        )

        self.assertIn("source_ineligible", failed_status["bounded_failure_reasons"])
        self.assertIn("rerun_failed_shards", failed_status["allowed_controls"])
        self.assertIn("source_changed", stale_status["bounded_failure_reasons"])
        self.assertIn("clear_changed_source_cache", stale_status["allowed_controls"])
        self.assertTrue(all(item["aria_label"] for item in failed_status["keyboard_controls"]))

    def test_non_ready_states_have_distinct_operator_labels_and_cancel_control(self) -> None:
        states = ["planned", "submitted", "running", "partial_success", "cancelled", "denied", "failed", "stale"]
        labels = set()
        for state in states:
            status = build_operator_run_status(summary=_summary(state), receipts=[], cancellation_available=True)
            labels.add(status["state_label"])
            assert_schema_valid(self, status, schema_name="gnat-operator-run-status.schema.json")

        running_status = build_operator_run_status(summary=_summary("running"), receipts=[], cancellation_available=True)

        self.assertEqual(len(labels), len(states))
        self.assertIn("cancel_run", running_status["allowed_controls"])
        cancel_control = next(item for item in running_status["keyboard_controls"] if item["control_id"] == "cancel_run")
        self.assertTrue(cancel_control["enabled"])


if __name__ == "__main__":
    unittest.main()
