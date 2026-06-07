from __future__ import annotations

import unittest

from cortex_runtime.gnats import FaLocalCapabilityState
from cortex_runtime.gnats.status import gnat_status_summary
from cortex_runtime.service_status import emit_service_status
from cortex_runtime.source_lanes import pdf_lane_runtime_available
from tests.runtime.runtime_test_support import assert_schema_valid


class GnatStatusRuntimeTests(unittest.TestCase):
    def test_gnat_status_truth_reports_serial_proof_not_parallel_readiness(self) -> None:
        summary = gnat_status_summary()

        self.assertEqual(summary["profile"], "serial_contract_proof")
        self.assertIn("markdown_syntax", summary["admitted_worker_types"])
        self.assertIn("plain_text_syntax", summary["admitted_worker_types"])
        self.assertIn("docx_text_syntax", summary["admitted_worker_types"])
        if pdf_lane_runtime_available():
            self.assertIn("pdf_text_syntax", summary["admitted_worker_types"])
        self.assertEqual(summary["max_concurrency"], 4)
        self.assertEqual(summary["hard_cap"], 8)
        self.assertTrue(summary["fa_local_required_for_parallel"])
        self.assertEqual(summary["fa_local_state"], "unavailable")
        self.assertFalse(summary["parallel_execution_ready"])
        self.assertTrue(summary["serial_fallback_available"])

    def test_service_status_embeds_schema_valid_gnat_summary(self) -> None:
        status = emit_service_status()

        assert_schema_valid(self, status, schema_name="service-status.schema.json")
        self.assertEqual(status["gnat_summary"]["profile"], "serial_contract_proof")
        self.assertFalse(status["gnat_summary"]["parallel_execution_ready"])

    def test_ready_fa_local_state_reports_parallel_capability_truth(self) -> None:
        summary = gnat_status_summary(
            FaLocalCapabilityState(
                fa_local_state="ready",
                supported_contract_versions=(
                    "GnatDispatchEnvelope.v1",
                    "GnatRunPlan.v1",
                    "GnatWorkerReceipt.v1",
                ),
                admitted_worker_types=("markdown_syntax", "plain_text_syntax"),
                max_concurrency=4,
                cancellation_supported=True,
            )
        )

        self.assertEqual(summary["profile"], "bounded_parallel_extraction")
        self.assertEqual(summary["fa_local_state"], "ready")
        self.assertTrue(summary["parallel_execution_ready"])


if __name__ == "__main__":
    unittest.main()
