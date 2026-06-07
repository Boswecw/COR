from __future__ import annotations

import unittest

from cortex_runtime.gnats import (
    FaLocalCapabilityState,
    GnatSourceInput,
    InMemoryGnatPersistenceStore,
    cache_identity_for_shard,
    plan_gnat_run,
    run_parallel_gnat_plan,
    run_parallel_gnat_plan_with_persistence,
)
from tests.runtime.runtime_test_support import ROOT, assert_schema_valid


RTF_READY_FIXTURE = ROOT / "tests/runtime/fixtures/sample-note.rtf"
RTF_ANNOTATED_FIXTURE = ROOT / "tests/runtime/fixtures/sample-note-annotated.rtf"
RTF_CORRUPT_FIXTURE = ROOT / "tests/runtime/fixtures/sample-note-corrupt.rtf"


def ready_fa_local_state(max_concurrency: int = 4) -> FaLocalCapabilityState:
    return FaLocalCapabilityState(
        fa_local_state="ready",
        supported_contract_versions=(
            "GnatDispatchEnvelope.v1",
            "GnatRunPlan.v1",
            "GnatWorkerReceipt.v1",
        ),
        admitted_worker_types=(
            "markdown_syntax",
            "plain_text_syntax",
            "pdf_text_syntax",
            "docx_text_syntax",
            "rtf_text_syntax",
        ),
        max_concurrency=max_concurrency,
        cancellation_supported=True,
    )


class GnatRtfLaneRuntimeTests(unittest.TestCase):
    def test_rtf_gnat_parallel_run_emits_schema_valid_ready_receipt(self) -> None:
        plan = plan_gnat_run(
            [GnatSourceInput(RTF_READY_FIXTURE, media_type="application/rtf", source_ref="rtf-note")],
            request_id="gnat-rtf-ready",
        )

        result = run_parallel_gnat_plan(plan, ready_fa_local_state(1))

        self.assertEqual(plan.shards[0].worker_type, "rtf_text_syntax")
        self.assertEqual(result.summary["run_state"], "ready")
        self.assertEqual(result.receipts[0]["state"], "complete")
        self.assertEqual(result.receipts[0]["bounded_output"]["structures"]["metadata_fields"]["source_lane"], "rtf_text")
        assert_schema_valid(self, result.receipts[0], schema_name="gnat-worker-receipt.schema.json")

    def test_rtf_gnat_denial_is_visible_for_annotated_rtf(self) -> None:
        plan = plan_gnat_run(
            [GnatSourceInput(RTF_ANNOTATED_FIXTURE, media_type="text/rtf", source_ref="rtf-annotated")],
            request_id="gnat-rtf-denied",
        )

        result = run_parallel_gnat_plan(plan, ready_fa_local_state(1))

        self.assertEqual(result.receipts[0]["state"], "denied")
        self.assertEqual(result.receipts[0]["error_reason_code"], "source_ineligible")
        self.assertEqual(result.summary["run_state"], "failed")

    def test_rtf_gnat_unavailable_is_visible_for_corrupt_rtf(self) -> None:
        plan = plan_gnat_run(
            [GnatSourceInput(RTF_CORRUPT_FIXTURE, media_type="application/rtf", source_ref="rtf-corrupt")],
            request_id="gnat-rtf-unavailable",
        )

        result = run_parallel_gnat_plan(plan, ready_fa_local_state(1))

        self.assertEqual(result.receipts[0]["state"], "failed")
        self.assertEqual(result.receipts[0]["error_reason_code"], "worker_unavailable")
        self.assertEqual(result.summary["run_state"], "failed")

    def test_rtf_gnat_cache_identity_uses_rtf_lane_contract(self) -> None:
        plan = plan_gnat_run(
            [GnatSourceInput(RTF_READY_FIXTURE, media_type="application/rtf", source_ref="rtf-note")],
            request_id="gnat-rtf-cache",
        )
        identity = cache_identity_for_shard(plan.shards[0])

        self.assertEqual(identity.worker_type, "rtf_text_syntax")
        self.assertEqual(identity.lane_contract_version, "local_file_rtf_text.v1")

    def test_rtf_gnat_persistence_records_cache_for_ready_rtf(self) -> None:
        plan = plan_gnat_run(
            [GnatSourceInput(RTF_READY_FIXTURE, media_type="application/rtf", source_ref="rtf-note")],
            request_id="gnat-rtf-persist",
        )
        store = InMemoryGnatPersistenceStore()

        result = run_parallel_gnat_plan_with_persistence(plan, ready_fa_local_state(1), store)

        self.assertEqual(result.persistence.cache_records_written, 1)
        self.assertEqual(len(store.cache_records), 1)
        cache_record = next(iter(store.cache_records.values()))
        self.assertEqual(cache_record["worker_type"], "rtf_text_syntax")
        assert_schema_valid(self, cache_record, schema_name="gnat-cache-record.schema.json")


if __name__ == "__main__":
    unittest.main()
