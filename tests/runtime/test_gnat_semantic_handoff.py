from __future__ import annotations

import unittest

from cortex_runtime.gnats import (
    FaLocalCapabilityState,
    GnatSourceInput,
    emit_gnat_semantic_handoff_from_retrieval_package,
    emit_retrieval_package_from_gnat_result,
    plan_gnat_run,
    run_parallel_gnat_plan,
)
from tests.runtime.runtime_test_support import ROOT, assert_schema_valid


GNAT_FIXTURE_DIR = ROOT / "tests/runtime/fixtures/gnats/text-batch-small"
MARKDOWN_FIXTURE = GNAT_FIXTURE_DIR / "chapter-01.md"
TEXT_FIXTURE = GNAT_FIXTURE_DIR / "note-plain.txt"
EMPTY_TEXT_FIXTURE = ROOT / "tests/runtime/fixtures/sample-empty.txt"


MODEL_RESOURCE_DISCLOSURE = {
    "route_class": "WORKHORSE_LOCAL",
    "model_id": "qwen2.5:14b",
    "resource_budget_class": "workhorse_local",
    "execution_mode": "local_model",
}


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


class GnatSemanticHandoffRuntimeTests(unittest.TestCase):
    def test_ready_retrieval_package_emits_schema_valid_neuronforge_handoff(self) -> None:
        plan = plan_gnat_run(
            [
                GnatSourceInput(MARKDOWN_FIXTURE, media_type="text/markdown", source_ref="chapter"),
                GnatSourceInput(TEXT_FIXTURE, media_type="text/plain", source_ref="note"),
            ],
            request_id="gnat-semantic-ready",
            requested_concurrency=2,
        )
        result = run_parallel_gnat_plan(plan, ready_fa_local_state(2))
        package = emit_retrieval_package_from_gnat_result(result)

        handoff = emit_gnat_semantic_handoff_from_retrieval_package(
            package,
            source_gnat_run_id=plan.run_id,
            source_plan_hash=plan.plan_hash,
            requested_by="authorforge",
            model_resource_disclosure=MODEL_RESOURCE_DISCLOSURE,
        )

        assert_schema_valid(self, handoff, schema_name="gnat-semantic-handoff.schema.json")
        self.assertEqual(handoff["destination_service_id"], "neuronforge-local")
        self.assertEqual(handoff["source_artifact"]["artifact_ref"], package["package_id"])
        self.assertEqual(handoff["source_artifact"]["source_state"], "ready")
        self.assertEqual(handoff["source_artifact"]["completeness_status"], "complete")
        self.assertEqual(handoff["candidate_generation"]["semantic_result_posture"], "non_canonical_candidate")
        self.assertTrue(handoff["transfer_guardrails"]["cor_receipts_immutable"])
        self.assertFalse(handoff["transfer_guardrails"]["receipt_mutation_allowed"])
        self.assertFalse(handoff["transfer_guardrails"]["semantic_output_canonical"])
        self.assertFalse(handoff["transfer_guardrails"]["raw_content_included"])
        self.assertNotIn("chunks", handoff)
        self.assertNotIn("text", handoff)
        self.assertNotIn("raw_content", handoff)

    def test_partial_retrieval_package_remains_noncanonical_and_incomplete(self) -> None:
        plan = plan_gnat_run(
            [
                GnatSourceInput(MARKDOWN_FIXTURE, media_type="text/markdown", source_ref="chapter"),
                GnatSourceInput(EMPTY_TEXT_FIXTURE, media_type="text/plain", source_ref="empty"),
            ],
            request_id="gnat-semantic-partial",
            requested_concurrency=2,
        )
        result = run_parallel_gnat_plan(plan, ready_fa_local_state(2))
        package = emit_retrieval_package_from_gnat_result(result)

        handoff = emit_gnat_semantic_handoff_from_retrieval_package(
            package,
            source_gnat_run_id=plan.run_id,
            source_plan_hash=plan.plan_hash,
            requested_by="authorforge",
            model_resource_disclosure=MODEL_RESOURCE_DISCLOSURE,
        )

        assert_schema_valid(self, handoff, schema_name="gnat-semantic-handoff.schema.json")
        self.assertEqual(handoff["source_artifact"]["source_state"], "partial_success")
        self.assertEqual(handoff["source_artifact"]["completeness_status"], "incomplete")
        self.assertFalse(handoff["transfer_guardrails"]["semantic_output_canonical"])

    def test_denied_retrieval_package_cannot_emit_semantic_handoff(self) -> None:
        plan = plan_gnat_run(
            [GnatSourceInput(EMPTY_TEXT_FIXTURE, media_type="text/plain", source_ref="empty")],
            request_id="gnat-semantic-denied",
        )
        result = run_parallel_gnat_plan(plan, ready_fa_local_state(1))
        package = emit_retrieval_package_from_gnat_result(result)

        with self.assertRaises(ValueError):
            emit_gnat_semantic_handoff_from_retrieval_package(
                package,
                source_gnat_run_id=plan.run_id,
                source_plan_hash=plan.plan_hash,
                requested_by="authorforge",
                model_resource_disclosure=MODEL_RESOURCE_DISCLOSURE,
            )

    def test_explicit_request_is_required(self) -> None:
        plan = plan_gnat_run(
            [GnatSourceInput(MARKDOWN_FIXTURE, media_type="text/markdown", source_ref="chapter")],
            request_id="gnat-semantic-request-required",
        )
        result = run_parallel_gnat_plan(plan, ready_fa_local_state(1))
        package = emit_retrieval_package_from_gnat_result(result)

        with self.assertRaises(ValueError):
            emit_gnat_semantic_handoff_from_retrieval_package(
                package,
                source_gnat_run_id=plan.run_id,
                source_plan_hash=plan.plan_hash,
                requested_by="",
                model_resource_disclosure=MODEL_RESOURCE_DISCLOSURE,
            )

    def test_model_resource_disclosure_is_required(self) -> None:
        plan = plan_gnat_run(
            [GnatSourceInput(MARKDOWN_FIXTURE, media_type="text/markdown", source_ref="chapter")],
            request_id="gnat-semantic-model-required",
        )
        result = run_parallel_gnat_plan(plan, ready_fa_local_state(1))
        package = emit_retrieval_package_from_gnat_result(result)

        with self.assertRaises(ValueError):
            emit_gnat_semantic_handoff_from_retrieval_package(
                package,
                source_gnat_run_id=plan.run_id,
                source_plan_hash=plan.plan_hash,
                requested_by="authorforge",
                model_resource_disclosure={
                    "route_class": "WORKHORSE_LOCAL",
                    "resource_budget_class": "workhorse_local",
                    "execution_mode": "local_model",
                },
            )


if __name__ == "__main__":
    unittest.main()
