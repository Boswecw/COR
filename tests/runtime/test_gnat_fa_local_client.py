from __future__ import annotations

import unittest

from cortex_runtime.gnats import (
    FaLocalCapabilityState,
    GnatDispatchError,
    GnatSourceInput,
    build_dispatch_envelope,
    negotiate_dispatch,
    plan_gnat_run,
)
from tests.runtime.runtime_test_support import ROOT, assert_schema_valid


GNAT_FIXTURE_DIR = ROOT / "tests/runtime/fixtures/gnats/text-batch-small"
MARKDOWN_FIXTURE = GNAT_FIXTURE_DIR / "chapter-01.md"
TEXT_FIXTURE = GNAT_FIXTURE_DIR / "note-plain.txt"


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


class GnatFaLocalClientRuntimeTests(unittest.TestCase):
    def test_build_dispatch_envelope_is_schema_valid_and_path_redacted(self) -> None:
        plan = plan_gnat_run(
            [
                GnatSourceInput(MARKDOWN_FIXTURE, media_type="text/markdown", source_ref="chapter-01"),
                GnatSourceInput(TEXT_FIXTURE, media_type="text/plain", source_ref="note-plain"),
            ],
            request_id="gnat-dispatch-001",
            correlation_id="aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa",
            requested_concurrency=4,
        )

        envelope = build_dispatch_envelope(plan)

        assert_schema_valid(self, envelope, schema_name="gnat-dispatch-envelope.schema.json")
        self.assertEqual(envelope["correlation_id"], "aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa")
        self.assertEqual(envelope["plan"]["run_id"], plan.run_id)
        self.assertEqual(envelope["plan"]["shard_count"], 2)
        self.assertEqual(
            envelope["required_capabilities"]["worker_types"],
            ["markdown_syntax", "plain_text_syntax"],
        )
        self.assertTrue(envelope["route_policy"]["fa_local_owns_execution_routing"])
        self.assertTrue(envelope["route_policy"]["cortex_validates_receipts"])
        self.assertNotIn(str(MARKDOWN_FIXTURE), str(envelope))
        self.assertNotIn("raw_content_preview", str(envelope))

    def test_non_uuid_plan_correlation_is_mapped_to_deterministic_fa_local_uuid(self) -> None:
        left = plan_gnat_run([MARKDOWN_FIXTURE], request_id="gnat-dispatch-uuid", correlation_id="local-corr")
        right = plan_gnat_run([MARKDOWN_FIXTURE], request_id="gnat-dispatch-uuid", correlation_id="local-corr")

        left_envelope = build_dispatch_envelope(left)
        right_envelope = build_dispatch_envelope(right)

        assert_schema_valid(self, left_envelope, schema_name="gnat-dispatch-envelope.schema.json")
        self.assertEqual(left_envelope["correlation_id"], right_envelope["correlation_id"])
        self.assertNotEqual(left_envelope["correlation_id"], "local-corr")

    def test_ready_fa_local_negotiation_clamps_concurrency(self) -> None:
        plan = plan_gnat_run(
            [MARKDOWN_FIXTURE, TEXT_FIXTURE],
            request_id="gnat-dispatch-ready",
            requested_concurrency=8,
        )

        negotiation = negotiate_dispatch(plan, ready_fa_local_state(max_concurrency=3))

        self.assertEqual(negotiation.state, "fa_local_dispatch_ready")
        self.assertEqual(negotiation.effective_concurrency, 3)
        self.assertEqual(
            negotiation.admitted_worker_types,
            ("markdown_syntax", "plain_text_syntax"),
        )

    def test_unavailable_fa_local_uses_serial_fallback_when_allowed(self) -> None:
        plan = plan_gnat_run([MARKDOWN_FIXTURE], request_id="gnat-dispatch-fallback")
        state = FaLocalCapabilityState(fa_local_state="unavailable")

        negotiation = negotiate_dispatch(plan, state)

        self.assertEqual(negotiation.state, "serial_fallback")
        self.assertEqual(negotiation.effective_concurrency, 1)
        self.assertIn("serial fallback", negotiation.operator_visible_summary)

    def test_unavailable_fa_local_denies_when_fallback_is_not_allowed(self) -> None:
        plan = plan_gnat_run(
            [MARKDOWN_FIXTURE],
            request_id="gnat-dispatch-no-fallback",
            serial_fallback_allowed=False,
        )

        with self.assertRaises(GnatDispatchError):
            negotiate_dispatch(plan, FaLocalCapabilityState(fa_local_state="unavailable"))

    def test_missing_contract_or_worker_admission_denies_ready_dispatch(self) -> None:
        plan = plan_gnat_run([MARKDOWN_FIXTURE, TEXT_FIXTURE], request_id="gnat-dispatch-deny")

        with self.assertRaises(GnatDispatchError):
            negotiate_dispatch(
                plan,
                FaLocalCapabilityState(
                    fa_local_state="ready",
                    supported_contract_versions=("GnatDispatchEnvelope.v1",),
                    admitted_worker_types=("markdown_syntax", "plain_text_syntax"),
                    max_concurrency=4,
                    cancellation_supported=True,
                ),
            )

        with self.assertRaises(GnatDispatchError):
            negotiate_dispatch(
                plan,
                FaLocalCapabilityState(
                    fa_local_state="ready",
                    supported_contract_versions=(
                        "GnatDispatchEnvelope.v1",
                        "GnatRunPlan.v1",
                        "GnatWorkerReceipt.v1",
                    ),
                    admitted_worker_types=("markdown_syntax",),
                    max_concurrency=4,
                    cancellation_supported=True,
                ),
            )


if __name__ == "__main__":
    unittest.main()
