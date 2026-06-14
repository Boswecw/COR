from __future__ import annotations

import shutil
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

from cortex_runtime.scrivener_authority_recon import (
    OPERATOR_DISABLE_ENV,
    emit_scrivener_authority_recon_from_source_file,
    main as scrivener_authority_recon_main,
)
from tests.runtime.runtime_test_support import ROOT, assert_schema_valid, capture_cli_result


SCRIV_MIXED_FIXTURE = (
    ROOT
    / "fixtures/scrivener/positive/scriv-mixed-structure-sanitized-v1/scriv-mixed-structure-sanitized-v1.scriv"
)
SCRIV_FAITH_FIXTURE = (
    ROOT
    / "fixtures/scrivener/positive/faith-in-a-firestorm-sanitized-v1/faith-in-a-firestorm-sanitized-v1.scriv"
)
SCRIV_MALFORMED_FIXTURE = (
    ROOT
    / "fixtures/scrivener/negative/faith-in-a-firestorm-sanitized-corrupt-scrivx-negative-fixture/faith-in-a-firestorm-sanitized-v1.scriv"
)
SCRIV_MISSING_CONTENT_FIXTURE = (
    ROOT
    / "fixtures/scrivener/negative/the-heart-of-the-storm-sanitized-missing-content-negative-fixture/the-heart-of-the-storm-sanitized-v1.scriv"
)
SCRIV_MULTI_AUTHORITY_PACKET = (
    ROOT / "fixtures/scrivener/negative/symbiogenesis-gunnach-protocol-sanitized-multi-authority-negative-fixture"
)
TEXT_FIXTURE = ROOT / "tests/runtime/fixtures/sample-note.txt"


def _copy_scriv_fixture(source: Path, target: Path) -> None:
    shutil.copytree(source, target)


def _materialize_multi_authority_project(target: Path) -> None:
    target.mkdir(parents=True, exist_ok=True)
    for name in (
        "Symbiogenesis - Gunnach Protocol.scrivx",
        "Symbiogenesis - Gunnach Protocol - conflicting.scrivx",
        "Files",
        "Settings",
    ):
        source = SCRIV_MULTI_AUTHORITY_PACKET / name
        destination = target / name
        if source.is_dir():
            shutil.copytree(source, destination)
        else:
            shutil.copy2(source, destination)


class ScrivenerAuthorityReconRuntimeTests(unittest.TestCase):
    def test_mixed_structure_fixture_emits_ready_stage1_status(self) -> None:
        result = emit_scrivener_authority_recon_from_source_file(
            SCRIV_MIXED_FIXTURE,
            request_id="scriv-stage1-ready-001",
            source_ref="scriv-mixed",
        )

        assert_schema_valid(self, result, schema_name="scrivener-authority-recon-status.schema.json")
        self.assertEqual(result["state"], "ready")
        self.assertEqual(result["authority_status"]["state"], "single_authority_observed")
        self.assertEqual(result["mapping_status"]["state"], "candidate_mapping_observed")
        self.assertEqual(
            result["observed_role_surfaces"],
            ["draft", "research", "trash", "template", "bookmarks"],
        )
        self.assertNotIn("refusal", result)

    def test_faith_fixture_emits_ready_but_mapping_unresolved(self) -> None:
        result = emit_scrivener_authority_recon_from_source_file(
            SCRIV_FAITH_FIXTURE,
            request_id="scriv-stage1-ready-002",
            source_ref="scriv-faith",
        )

        assert_schema_valid(self, result, schema_name="scrivener-authority-recon-status.schema.json")
        self.assertEqual(result["state"], "ready")
        self.assertEqual(result["authority_status"]["state"], "single_authority_observed")
        self.assertEqual(result["mapping_status"]["state"], "mapping_unresolved")
        self.assertGreater(result["mapping_status"]["data_only_uuid_count"], 0)
        self.assertTrue(result["provenance"]["authority_path"].endswith(".scrivx"))

    def test_missing_top_level_authority_is_unavailable(self) -> None:
        with tempfile.TemporaryDirectory() as tmpdir:
            project_path = Path(tmpdir) / "missing-authority.scriv"
            _copy_scriv_fixture(SCRIV_MIXED_FIXTURE, project_path)
            next(project_path.glob("*.scrivx")).unlink()

            result = emit_scrivener_authority_recon_from_source_file(
                project_path,
                request_id="scriv-stage1-missing-auth-001",
                source_ref="scriv-missing-auth",
            )

        assert_schema_valid(self, result, schema_name="scrivener-authority-recon-status.schema.json")
        self.assertEqual(result["state"], "unavailable")
        self.assertEqual(result["authority_status"]["state"], "missing_authority")
        self.assertEqual(result["mapping_status"]["state"], "not_attempted")

    def test_multiple_top_level_authorities_are_unavailable(self) -> None:
        with tempfile.TemporaryDirectory() as tmpdir:
            project_path = Path(tmpdir) / "multi-authority.scriv"
            _materialize_multi_authority_project(project_path)

            result = emit_scrivener_authority_recon_from_source_file(
                project_path,
                request_id="scriv-stage1-ambiguous-001",
                source_ref="scriv-ambiguous",
            )

        assert_schema_valid(self, result, schema_name="scrivener-authority-recon-status.schema.json")
        self.assertEqual(result["state"], "unavailable")
        self.assertEqual(result["authority_status"]["state"], "ambiguous_authority")
        self.assertEqual(result["authority_status"]["candidate_count"], 2)

    def test_malformed_authority_is_unavailable(self) -> None:
        result = emit_scrivener_authority_recon_from_source_file(
            SCRIV_MALFORMED_FIXTURE,
            request_id="scriv-stage1-malformed-001",
            source_ref="scriv-malformed",
        )

        assert_schema_valid(self, result, schema_name="scrivener-authority-recon-status.schema.json")
        self.assertEqual(result["state"], "unavailable")
        self.assertEqual(result["authority_status"]["state"], "malformed_authority")
        self.assertEqual(result["refusal"]["reason_class"], "dependency_unavailable")

    def test_direct_missing_correspondence_is_unavailable(self) -> None:
        result = emit_scrivener_authority_recon_from_source_file(
            SCRIV_MISSING_CONTENT_FIXTURE,
            request_id="scriv-stage1-missing-content-001",
            source_ref="scriv-missing-content",
        )

        assert_schema_valid(self, result, schema_name="scrivener-authority-recon-status.schema.json")
        self.assertEqual(result["state"], "unavailable")
        self.assertEqual(result["authority_status"]["state"], "single_authority_observed")
        self.assertEqual(result["mapping_status"]["state"], "mapping_unavailable")
        self.assertGreater(result["mapping_status"]["direct_missing_content_target_count"], 0)

    def test_non_scriv_input_is_denied(self) -> None:
        result = emit_scrivener_authority_recon_from_source_file(
            TEXT_FIXTURE,
            request_id="scriv-stage1-denied-001",
            source_ref="not-scriv",
        )

        assert_schema_valid(self, result, schema_name="scrivener-authority-recon-status.schema.json")
        self.assertEqual(result["state"], "denied")
        self.assertEqual(result["refusal"]["reason_class"], "unsupported_source_type")
        self.assertEqual(result["authority_status"]["state"], "not_attempted")

    def test_operator_disabled_is_denied(self) -> None:
        with patch.dict("os.environ", {OPERATOR_DISABLE_ENV: "1"}):
            result = emit_scrivener_authority_recon_from_source_file(
                SCRIV_MIXED_FIXTURE,
                request_id="scriv-stage1-denied-002",
                source_ref="scriv-disabled",
            )

        assert_schema_valid(self, result, schema_name="scrivener-authority-recon-status.schema.json")
        self.assertEqual(result["state"], "denied")
        self.assertEqual(result["refusal"]["reason_class"], "operator_disabled")
        self.assertEqual(result["authority_status"]["state"], "not_attempted")

    def test_cli_entrypoint_emits_ready_json(self) -> None:
        exit_code, result = capture_cli_result(
            scrivener_authority_recon_main,
            [
                "--source-path",
                str(SCRIV_MIXED_FIXTURE),
                "--request-id",
                "scriv-stage1-cli-001",
                "--source-ref",
                "scriv-cli",
            ],
        )

        assert_schema_valid(self, result, schema_name="scrivener-authority-recon-status.schema.json")
        self.assertEqual(exit_code, 0)
        self.assertEqual(result["state"], "ready")


if __name__ == "__main__":
    unittest.main()
