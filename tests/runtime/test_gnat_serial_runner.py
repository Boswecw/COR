from __future__ import annotations

import copy
import tempfile
import unittest
from pathlib import Path

from cortex_runtime.extraction_emission import emit_extraction_result_from_source_file
from cortex_runtime.gnats import GnatSourceInput, plan_gnat_run, reconcile_receipts, run_serial_gnat_plan
from tests.runtime.runtime_test_support import ROOT, assert_schema_valid


GNAT_FIXTURE_DIR = ROOT / "tests/runtime/fixtures/gnats/text-batch-small"
MARKDOWN_FIXTURE = GNAT_FIXTURE_DIR / "chapter-01.md"
TEXT_FIXTURE = GNAT_FIXTURE_DIR / "note-plain.txt"
EMPTY_TEXT_FIXTURE = ROOT / "tests/runtime/fixtures/sample-empty.txt"


class GnatSerialRunnerRuntimeTests(unittest.TestCase):
    def test_serial_runner_emits_schema_valid_receipts_and_ready_summary(self) -> None:
        plan = plan_gnat_run(
            [
                GnatSourceInput(MARKDOWN_FIXTURE, media_type="text/markdown", source_ref="chapter-01"),
                GnatSourceInput(TEXT_FIXTURE, media_type="text/plain", source_ref="note-plain"),
            ],
            request_id="gnat-serial-001",
            requested_concurrency=2,
        )

        result = run_serial_gnat_plan(plan)

        self.assertEqual(len(result.receipts), 2)
        for receipt in result.receipts:
            assert_schema_valid(self, receipt, schema_name="gnat-worker-receipt.schema.json")
            self.assertEqual(receipt["state"], "complete")
            self.assertNotIn("raw_content_preview", receipt)

        assert_schema_valid(self, result.summary, schema_name="gnat-run-summary.schema.json")
        self.assertEqual(result.summary["run_state"], "ready")
        self.assertEqual(result.summary["completed_count"], 2)
        self.assertEqual(result.summary["missing_count"], 0)
        self.assertTrue(result.summary["fallback_used"])

    def test_serial_worker_matches_existing_extraction_structures(self) -> None:
        plan = plan_gnat_run(
            [GnatSourceInput(MARKDOWN_FIXTURE, media_type="text/markdown", source_ref="chapter-01")],
            request_id="gnat-serial-equivalence",
        )
        result = run_serial_gnat_plan(plan)
        legacy = emit_extraction_result_from_source_file(
            MARKDOWN_FIXTURE,
            request_id=plan.run_id,
            source_ref="chapter-01",
            media_type="text/markdown",
        )

        receipt_output = result.receipts[0]["bounded_output"]
        self.assertEqual(receipt_output["structures"], legacy["structures"])
        self.assertEqual(receipt_output["state"], legacy["state"])

    def test_denied_shard_produces_partial_success_summary(self) -> None:
        plan = plan_gnat_run(
            [
                GnatSourceInput(MARKDOWN_FIXTURE, media_type="text/markdown", source_ref="chapter-01"),
                GnatSourceInput(EMPTY_TEXT_FIXTURE, media_type="text/plain", source_ref="empty-text"),
            ],
            request_id="gnat-serial-partial",
        )

        result = run_serial_gnat_plan(plan)

        self.assertEqual([receipt["state"] for receipt in result.receipts], ["complete", "denied"])
        self.assertEqual(result.summary["run_state"], "partial_success")
        self.assertEqual(result.summary["completed_count"], 1)
        self.assertEqual(result.summary["failed_count"], 1)

    def test_source_change_after_planning_is_reported_stale(self) -> None:
        with tempfile.TemporaryDirectory() as tmpdir:
            source = Path(tmpdir) / "stale-note.txt"
            source.write_text("before planning\n", encoding="utf-8")
            plan = plan_gnat_run([GnatSourceInput(source, media_type="text/plain")], request_id="gnat-stale-001")
            source.write_text("after planning\n", encoding="utf-8")

            result = run_serial_gnat_plan(plan)

        self.assertEqual(result.receipts[0]["state"], "stale")
        self.assertEqual(result.receipts[0]["error_reason_code"], "source_changed")
        self.assertEqual(result.summary["run_state"], "stale")
        self.assertEqual(result.summary["stale_count"], 1)

    def test_reconciliation_is_completion_order_independent(self) -> None:
        plan = plan_gnat_run([MARKDOWN_FIXTURE, TEXT_FIXTURE], request_id="gnat-order-001")
        result = run_serial_gnat_plan(plan)

        forward = reconcile_receipts(plan, list(result.receipts))
        reverse = reconcile_receipts(plan, list(reversed(result.receipts)))

        self.assertEqual(forward, reverse)

    def test_duplicate_and_missing_receipts_are_visible(self) -> None:
        plan = plan_gnat_run([MARKDOWN_FIXTURE, TEXT_FIXTURE], request_id="gnat-reconcile-001")
        result = run_serial_gnat_plan(plan)

        duplicate_summary = reconcile_receipts(plan, [*result.receipts, copy.deepcopy(result.receipts[0])])
        self.assertEqual(duplicate_summary["run_state"], "partial_success")
        self.assertEqual(duplicate_summary["rejected_receipts"][0]["reason_code"], "duplicate_shard")

        missing_summary = reconcile_receipts(plan, [result.receipts[0]])
        self.assertEqual(missing_summary["run_state"], "partial_success")
        self.assertEqual(missing_summary["missing_count"], 1)
        self.assertTrue(any(item["reason_code"] == "missing_shard" for item in missing_summary["rejected_receipts"]))

    def test_serialized_plan_and_receipts_do_not_expose_local_path(self) -> None:
        plan = plan_gnat_run([MARKDOWN_FIXTURE], request_id="gnat-privacy-001")
        result = run_serial_gnat_plan(plan)
        serialized = str(plan.to_contract()) + str(result.receipts[0]) + str(result.summary)

        self.assertNotIn(str(MARKDOWN_FIXTURE), serialized)
        self.assertNotIn("raw_content_preview", serialized)
        self.assertNotIn("workflow_id", serialized)


if __name__ == "__main__":
    unittest.main()
