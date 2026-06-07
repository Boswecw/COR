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


EPUB_MEDIA_TYPE = "application/epub+zip"
EPUB_READY_FIXTURE = ROOT / "tests/runtime/fixtures/sample-note.epub"
EPUB_SCRIPTED_FIXTURE = ROOT / "tests/runtime/fixtures/sample-note-scripted.epub"
EPUB_MISSING_PACKAGE_FIXTURE = ROOT / "tests/runtime/fixtures/sample-note-missing-package.epub"


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
            "epub_text_syntax",
        ),
        max_concurrency=max_concurrency,
        cancellation_supported=True,
    )


class GnatEpubLaneRuntimeTests(unittest.TestCase):
    def test_epub_gnat_parallel_run_emits_schema_valid_ready_receipt(self) -> None:
        plan = plan_gnat_run(
            [GnatSourceInput(EPUB_READY_FIXTURE, media_type=EPUB_MEDIA_TYPE, source_ref="epub-note")],
            request_id="gnat-epub-ready",
        )

        result = run_parallel_gnat_plan(plan, ready_fa_local_state(1))

        self.assertEqual(plan.shards[0].worker_type, "epub_text_syntax")
        self.assertEqual(result.summary["run_state"], "ready")
        self.assertEqual(result.receipts[0]["state"], "complete")
        self.assertEqual(result.receipts[0]["bounded_output"]["structures"]["metadata_fields"]["source_lane"], "epub_text")
        assert_schema_valid(self, result.receipts[0], schema_name="gnat-worker-receipt.schema.json")

    def test_epub_gnat_denial_is_visible_for_scripted_epub(self) -> None:
        plan = plan_gnat_run(
            [GnatSourceInput(EPUB_SCRIPTED_FIXTURE, media_type=EPUB_MEDIA_TYPE, source_ref="epub-scripted")],
            request_id="gnat-epub-denied",
        )

        result = run_parallel_gnat_plan(plan, ready_fa_local_state(1))

        self.assertEqual(result.receipts[0]["state"], "denied")
        self.assertEqual(result.receipts[0]["error_reason_code"], "source_ineligible")
        self.assertEqual(result.summary["run_state"], "failed")

    def test_epub_gnat_unavailable_is_visible_for_missing_package_epub(self) -> None:
        plan = plan_gnat_run(
            [GnatSourceInput(EPUB_MISSING_PACKAGE_FIXTURE, media_type=EPUB_MEDIA_TYPE, source_ref="epub-missing")],
            request_id="gnat-epub-unavailable",
        )

        result = run_parallel_gnat_plan(plan, ready_fa_local_state(1))

        self.assertEqual(result.receipts[0]["state"], "failed")
        self.assertEqual(result.receipts[0]["error_reason_code"], "worker_unavailable")
        self.assertEqual(result.summary["run_state"], "failed")

    def test_epub_gnat_cache_identity_uses_epub_lane_contract(self) -> None:
        plan = plan_gnat_run(
            [GnatSourceInput(EPUB_READY_FIXTURE, media_type=EPUB_MEDIA_TYPE, source_ref="epub-note")],
            request_id="gnat-epub-cache",
        )
        identity = cache_identity_for_shard(plan.shards[0])

        self.assertEqual(identity.worker_type, "epub_text_syntax")
        self.assertEqual(identity.lane_contract_version, "local_file_epub_text.v1")

    def test_epub_gnat_persistence_records_cache_for_ready_epub(self) -> None:
        plan = plan_gnat_run(
            [GnatSourceInput(EPUB_READY_FIXTURE, media_type=EPUB_MEDIA_TYPE, source_ref="epub-note")],
            request_id="gnat-epub-persist",
        )
        store = InMemoryGnatPersistenceStore()

        result = run_parallel_gnat_plan_with_persistence(plan, ready_fa_local_state(1), store)

        self.assertEqual(result.persistence.cache_records_written, 1)
        self.assertEqual(len(store.cache_records), 1)
        cache_record = next(iter(store.cache_records.values()))
        self.assertEqual(cache_record["worker_type"], "epub_text_syntax")
        assert_schema_valid(self, cache_record, schema_name="gnat-cache-record.schema.json")


if __name__ == "__main__":
    unittest.main()
