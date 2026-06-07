from __future__ import annotations

import unittest

from cortex_runtime.gnats import GnatPlanningError, GnatSourceInput, plan_gnat_run
from cortex_runtime.gnats.registry import admitted_worker_types
from tests.runtime.runtime_test_support import ROOT, assert_schema_valid


GNAT_FIXTURE_DIR = ROOT / "tests/runtime/fixtures/gnats/text-batch-small"
MARKDOWN_FIXTURE = GNAT_FIXTURE_DIR / "chapter-01.md"
TEXT_FIXTURE = GNAT_FIXTURE_DIR / "note-plain.txt"
PDF_FIXTURE = ROOT / "tests/runtime/fixtures/sample-note.pdf"


class GnatPlannerRuntimeTests(unittest.TestCase):
    def test_planner_emits_schema_valid_plan_and_shards(self) -> None:
        plan = plan_gnat_run(
            [
                GnatSourceInput(MARKDOWN_FIXTURE, media_type="text/markdown", source_ref="chapter-01"),
                GnatSourceInput(TEXT_FIXTURE, media_type="text/plain", source_ref="note-plain"),
            ],
            request_id="gnat-plan-001",
            correlation_id="corr-gnat-plan-001",
            requested_concurrency=2,
        )

        plan_contract = plan.to_contract()
        assert_schema_valid(self, plan_contract, schema_name="gnat-run-plan.schema.json")
        self.assertEqual(plan_contract["shard_count"], 2)
        self.assertEqual(plan_contract["execution_limits"]["requested_concurrency"], 2)
        self.assertEqual(plan.shards[0].worker_type, "markdown_syntax")
        self.assertEqual(plan.shards[1].worker_type, "plain_text_syntax")
        self.assertNotIn(str(MARKDOWN_FIXTURE), str(plan_contract))

        for shard in plan.shards:
            assert_schema_valid(self, shard.to_contract(), schema_name="gnat-shard.schema.json")
            self.assertTrue(shard.shard_id.startswith(f"{plan.run_id}-shard-"))

    def test_shard_ids_are_deterministic_for_same_inputs(self) -> None:
        left = plan_gnat_run([MARKDOWN_FIXTURE, TEXT_FIXTURE], request_id="gnat-plan-deterministic")
        right = plan_gnat_run([MARKDOWN_FIXTURE, TEXT_FIXTURE], request_id="gnat-plan-deterministic")

        self.assertEqual([shard.shard_id for shard in left.shards], [shard.shard_id for shard in right.shards])
        self.assertEqual(left.run_id, right.run_id)

    def test_registry_admits_only_initial_text_worker_types(self) -> None:
        self.assertEqual(admitted_worker_types(), ["markdown_syntax", "plain_text_syntax"])

    def test_unsupported_lane_is_denied_during_planning(self) -> None:
        with self.assertRaises(GnatPlanningError):
            plan_gnat_run([GnatSourceInput(PDF_FIXTURE, media_type="application/pdf")], request_id="gnat-plan-pdf")

    def test_concurrency_is_hard_capped(self) -> None:
        plan = plan_gnat_run([MARKDOWN_FIXTURE], request_id="gnat-plan-cap", requested_concurrency=99)

        self.assertEqual(plan.requested_concurrency, 8)
        self.assertEqual(plan.max_concurrency, 4)


if __name__ == "__main__":
    unittest.main()
