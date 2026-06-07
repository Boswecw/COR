from __future__ import annotations

import unittest

from cortex_runtime.gnats import GnatSourceInput, cache_key_for_identity, cache_identity_for_shard, plan_gnat_run
from cortex_runtime.gnats.persistence import canonical_hash as cortex_canonical_hash
from gnat_core import (
    RunStateCounts,
    bounded_concurrency,
    cache_key_from_identity,
    canonical_hash,
    effective_concurrency,
    run_state_from_counts,
    source_path_token,
    stable_short_digest,
)
from tests.runtime.runtime_test_support import ROOT


GNAT_FIXTURE_DIR = ROOT / "tests/runtime/fixtures/gnats/text-batch-small"
MARKDOWN_FIXTURE = GNAT_FIXTURE_DIR / "chapter-01.md"


class GnatCoreSharedRuntimeTests(unittest.TestCase):
    def test_core_hashing_matches_existing_cortex_compatibility_wrapper(self) -> None:
        payload = {"z": [2, 1], "a": {"nested": True}}

        self.assertEqual(canonical_hash(payload), cortex_canonical_hash(payload))
        self.assertTrue(canonical_hash(payload).startswith("sha256:"))
        self.assertEqual(stable_short_digest("same-input"), stable_short_digest("same-input"))

    def test_source_path_token_remains_stable_and_redacted(self) -> None:
        token = source_path_token(MARKDOWN_FIXTURE)

        self.assertTrue(token.startswith("src:"))
        self.assertNotIn(str(MARKDOWN_FIXTURE), token)
        self.assertEqual(token, source_path_token(MARKDOWN_FIXTURE))

    def test_core_concurrency_helpers_match_cortex_contract_limits(self) -> None:
        self.assertEqual(bounded_concurrency(99, 99, hard_cap=8), (8, 8))
        self.assertEqual(bounded_concurrency(3, 7, hard_cap=8), (3, 7))
        self.assertEqual(effective_concurrency(8, 4, 3, 8), 3)

        with self.assertRaises(ValueError):
            bounded_concurrency(0, 4, hard_cap=8)
        with self.assertRaises(ValueError):
            effective_concurrency(4, 0)

    def test_core_lifecycle_state_matches_cortex_summary_vocabulary(self) -> None:
        self.assertEqual(run_state_from_counts(RunStateCounts(expected=2, completed=2)), "ready")
        self.assertEqual(run_state_from_counts(RunStateCounts(expected=2, completed=1, failed=1)), "partial_success")
        self.assertEqual(run_state_from_counts(RunStateCounts(expected=2, completed=0, stale=1)), "stale")
        self.assertEqual(run_state_from_counts(RunStateCounts(expected=2, completed=0, cancelled=1)), "cancelled")
        self.assertEqual(run_state_from_counts(RunStateCounts(expected=2, completed=0, failed=1)), "failed")

    def test_core_cache_key_matches_existing_cortex_cache_identity_key(self) -> None:
        plan = plan_gnat_run(
            [GnatSourceInput(MARKDOWN_FIXTURE, media_type="text/markdown", source_ref="chapter")],
            request_id="gnat-core-cache-key",
        )
        identity = cache_identity_for_shard(plan.shards[0])

        self.assertEqual(
            cache_key_for_identity(identity),
            cache_key_from_identity("GnatCacheRecord.v1", identity.to_contract()),
        )

    def test_shared_core_does_not_import_application_specific_modules(self) -> None:
        core_dir = ROOT / "gnat_core"
        forbidden = (
            "cortex_runtime",
            "fa_local",
            "neuronforge",
            "dataforge",
            "source_lanes",
            "workers",
        )

        for path in sorted(core_dir.glob("*.py")):
            text = path.read_text(encoding="utf-8")
            for pattern in forbidden:
                self.assertNotIn(pattern, text, f"{path.name} contains forbidden application import marker")


if __name__ == "__main__":
    unittest.main()
