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


ODT_MEDIA_TYPE = "application/vnd.oasis.opendocument.text"
ODT_READY_FIXTURE = ROOT / "tests/runtime/fixtures/sample-note.odt"
ODT_REVIEWED_FIXTURE = ROOT / "tests/runtime/fixtures/sample-note-reviewed.odt"
ODT_MISSING_CONTENT_FIXTURE = ROOT / "tests/runtime/fixtures/sample-note-missing-content.odt"


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
            "odt_text_syntax",
        ),
        max_concurrency=max_concurrency,
        cancellation_supported=True,
    )


class GnatOdtLaneRuntimeTests(unittest.TestCase):
    def test_odt_gnat_parallel_run_emits_schema_valid_ready_receipt(self) -> None:
        plan = plan_gnat_run(
            [GnatSourceInput(ODT_READY_FIXTURE, media_type=ODT_MEDIA_TYPE, source_ref="odt-note")],
            request_id="gnat-odt-ready",
        )

        result = run_parallel_gnat_plan(plan, ready_fa_local_state(1))

        self.assertEqual(plan.shards[0].worker_type, "odt_text_syntax")
        self.assertEqual(result.summary["run_state"], "ready")
        self.assertEqual(result.receipts[0]["state"], "complete")
        self.assertEqual(result.receipts[0]["bounded_output"]["structures"]["metadata_fields"]["source_lane"], "odt_text")
        assert_schema_valid(self, result.receipts[0], schema_name="gnat-worker-receipt.schema.json")

    def test_odt_gnat_denial_is_visible_for_reviewed_odt(self) -> None:
        plan = plan_gnat_run(
            [GnatSourceInput(ODT_REVIEWED_FIXTURE, media_type=ODT_MEDIA_TYPE, source_ref="odt-reviewed")],
            request_id="gnat-odt-denied",
        )

        result = run_parallel_gnat_plan(plan, ready_fa_local_state(1))

        self.assertEqual(result.receipts[0]["state"], "denied")
        self.assertEqual(result.receipts[0]["error_reason_code"], "source_ineligible")
        self.assertEqual(result.summary["run_state"], "failed")

    def test_odt_gnat_unavailable_is_visible_for_missing_content_odt(self) -> None:
        plan = plan_gnat_run(
            [GnatSourceInput(ODT_MISSING_CONTENT_FIXTURE, media_type=ODT_MEDIA_TYPE, source_ref="odt-missing")],
            request_id="gnat-odt-unavailable",
        )

        result = run_parallel_gnat_plan(plan, ready_fa_local_state(1))

        self.assertEqual(result.receipts[0]["state"], "failed")
        self.assertEqual(result.receipts[0]["error_reason_code"], "worker_unavailable")
        self.assertEqual(result.summary["run_state"], "failed")

    def test_odt_gnat_cache_identity_uses_odt_lane_contract(self) -> None:
        plan = plan_gnat_run(
            [GnatSourceInput(ODT_READY_FIXTURE, media_type=ODT_MEDIA_TYPE, source_ref="odt-note")],
            request_id="gnat-odt-cache",
        )
        identity = cache_identity_for_shard(plan.shards[0])

        self.assertEqual(identity.worker_type, "odt_text_syntax")
        self.assertEqual(identity.lane_contract_version, "local_file_odt_text.v1")

    def test_odt_gnat_persistence_records_cache_for_ready_odt(self) -> None:
        plan = plan_gnat_run(
            [GnatSourceInput(ODT_READY_FIXTURE, media_type=ODT_MEDIA_TYPE, source_ref="odt-note")],
            request_id="gnat-odt-persist",
        )
        store = InMemoryGnatPersistenceStore()

        result = run_parallel_gnat_plan_with_persistence(plan, ready_fa_local_state(1), store)

        self.assertEqual(result.persistence.cache_records_written, 1)
        self.assertEqual(len(store.cache_records), 1)
        cache_record = next(iter(store.cache_records.values()))
        self.assertEqual(cache_record["worker_type"], "odt_text_syntax")
        assert_schema_valid(self, cache_record, schema_name="gnat-cache-record.schema.json")


if __name__ == "__main__":
    unittest.main()
