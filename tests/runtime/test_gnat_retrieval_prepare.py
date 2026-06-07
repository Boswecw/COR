from __future__ import annotations

import unittest

from cortex_runtime.gnats import (
    FaLocalCapabilityState,
    GnatSourceInput,
    emit_retrieval_package_from_gnat_receipts,
    emit_retrieval_package_from_gnat_result,
    plan_gnat_run,
    run_parallel_gnat_plan,
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


class GnatRetrievalPrepareRuntimeTests(unittest.TestCase):
    def test_ready_gnat_receipts_emit_one_schema_valid_retrieval_package(self) -> None:
        plan = plan_gnat_run(
            [
                GnatSourceInput(MARKDOWN_FIXTURE, media_type="text/markdown", source_ref="chapter"),
                GnatSourceInput(TEXT_FIXTURE, media_type="text/plain", source_ref="note"),
            ],
            request_id="gnat-retrieval-ready",
            requested_concurrency=2,
        )
        result = run_parallel_gnat_plan(plan, ready_fa_local_state(2))

        package = emit_retrieval_package_from_gnat_result(result)

        assert_schema_valid(self, package, schema_name="retrieval-package.schema.json")
        self.assertEqual(package["state"], "ready")
        self.assertEqual(package["source_refs"], ["chapter", "note"])
        self.assertEqual([chunk["ordinal"] for chunk in package["chunks"]], list(range(len(package["chunks"]))))
        self.assertTrue(package["non_semantic_default"])
        for chunk in package["chunks"]:
            self.assertNotIn("score", chunk)
            self.assertNotIn("rank", chunk)

    def test_retrieval_package_is_deterministic_for_receipt_order(self) -> None:
        plan = plan_gnat_run(
            [
                GnatSourceInput(MARKDOWN_FIXTURE, media_type="text/markdown", source_ref="chapter"),
                GnatSourceInput(TEXT_FIXTURE, media_type="text/plain", source_ref="note"),
            ],
            request_id="gnat-retrieval-order",
            requested_concurrency=2,
        )
        result = run_parallel_gnat_plan(plan, ready_fa_local_state(2))

        forward = emit_retrieval_package_from_gnat_receipts(plan, result.receipts)
        reversed_package = emit_retrieval_package_from_gnat_receipts(plan, reversed(result.receipts))

        self.assertEqual(forward["package_id"], reversed_package["package_id"])
        self.assertEqual(forward["source_refs"], reversed_package["source_refs"])
        self.assertEqual(forward["retrieval_profile"], reversed_package["retrieval_profile"])
        self.assertEqual(forward["chunks"], reversed_package["chunks"])

    def test_partial_gnat_receipts_emit_partial_success_package(self) -> None:
        plan = plan_gnat_run(
            [
                GnatSourceInput(MARKDOWN_FIXTURE, media_type="text/markdown", source_ref="chapter"),
                GnatSourceInput(EMPTY_TEXT_FIXTURE, media_type="text/plain", source_ref="empty"),
            ],
            request_id="gnat-retrieval-partial",
            requested_concurrency=2,
        )
        result = run_parallel_gnat_plan(plan, ready_fa_local_state(2))

        package = emit_retrieval_package_from_gnat_result(result)

        assert_schema_valid(self, package, schema_name="retrieval-package.schema.json")
        self.assertEqual(package["state"], "partial_success")
        self.assertEqual(package["completeness"]["status"], "incomplete")
        self.assertEqual({chunk["source_ref"] for chunk in package["chunks"]}, {"chapter"})

    def test_all_failed_gnat_receipts_emit_denied_package(self) -> None:
        plan = plan_gnat_run(
            [GnatSourceInput(EMPTY_TEXT_FIXTURE, media_type="text/plain", source_ref="empty")],
            request_id="gnat-retrieval-denied",
        )
        result = run_parallel_gnat_plan(plan, ready_fa_local_state(1))

        package = emit_retrieval_package_from_gnat_result(result)

        assert_schema_valid(self, package, schema_name="retrieval-package.schema.json")
        self.assertEqual(package["state"], "denied")
        self.assertEqual(package["refusal"]["reason_class"], "missing_required_structure")
        self.assertNotIn("chunks", package)


if __name__ == "__main__":
    unittest.main()
