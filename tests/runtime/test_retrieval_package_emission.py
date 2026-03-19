from __future__ import annotations

import io
import json
import unittest
from contextlib import redirect_stdout
from pathlib import Path
from typing import Any

from jsonschema import Draft202012Validator

from cortex_runtime.extraction_emission import emit_extraction_result_from_source_file
from cortex_runtime.retrieval_package_emission import (
    emit_retrieval_package_from_extraction_json_text,
    emit_retrieval_package_from_extraction_result,
    emit_retrieval_package_from_source_file,
    main,
)


ROOT = Path(__file__).resolve().parents[2]
RETRIEVAL_SCHEMA_PATH = ROOT / "schemas/retrieval-package.schema.json"
SUPPORTED_SOURCE_FIXTURE = ROOT / "tests/runtime/fixtures/sample-note.md"
UNSUPPORTED_SOURCE_FIXTURE = ROOT / "tests/runtime/fixtures/sample-unsupported.bin"
INVALID_UPSTREAM_FIXTURE = ROOT / "tests/contracts/fixtures/invalid/extraction-result-denied-missing-refusal.json"
STALE_UPSTREAM_FIXTURE = ROOT / "tests/contracts/fixtures/valid/extraction-result-stale.json"


def load_json(path: Path) -> Any:
    with path.open("r", encoding="utf-8") as handle:
        return json.load(handle)


def retrieval_validator() -> Draft202012Validator:
    with RETRIEVAL_SCHEMA_PATH.open("r", encoding="utf-8") as handle:
        return Draft202012Validator(json.load(handle))


def assert_schema_valid(testcase: unittest.TestCase, payload: dict[str, Any]) -> None:
    errors = sorted(
        retrieval_validator().iter_errors(payload),
        key=lambda error: (".".join(str(part) for part in error.path), error.message),
    )
    testcase.assertEqual(
        [],
        [f"{'.'.join(str(part) for part in error.path) or '<root>'}: {error.message}" for error in errors],
    )


class RetrievalPackageEmissionRuntimeTests(unittest.TestCase):
    def test_ready_extraction_upstream_emits_ready_retrieval_package(self) -> None:
        extraction_result = emit_extraction_result_from_source_file(
            SUPPORTED_SOURCE_FIXTURE,
            request_id="slice3-001",
            source_ref="src-slice3",
        )

        result = emit_retrieval_package_from_extraction_result(extraction_result)

        assert_schema_valid(self, result)
        self.assertEqual(result["state"], "ready")
        self.assertEqual(result["retrieval_profile"]["chunking_mode"], "section")
        self.assertEqual(result["completeness"]["status"], "complete")
        self.assertTrue(result["non_canonical"])
        self.assertTrue(result["non_semantic_default"])

    def test_chunk_ordering_is_deterministic(self) -> None:
        extraction_result = emit_extraction_result_from_source_file(
            SUPPORTED_SOURCE_FIXTURE,
            request_id="slice3-002",
            source_ref="src-deterministic",
        )

        first = emit_retrieval_package_from_extraction_result(extraction_result)
        second = emit_retrieval_package_from_extraction_result(extraction_result)

        assert_schema_valid(self, first)
        assert_schema_valid(self, second)
        self.assertEqual(first["chunks"], second["chunks"])
        self.assertEqual([chunk["ordinal"] for chunk in first["chunks"]], [0, 1])

    def test_unsupported_input_is_denied(self) -> None:
        result = emit_retrieval_package_from_source_file(
            UNSUPPORTED_SOURCE_FIXTURE,
            request_id="slice3-003",
            source_ref="src-unsupported",
        )

        assert_schema_valid(self, result)
        self.assertEqual(result["state"], "denied")
        self.assertEqual(result["refusal"]["reason_class"], "unsupported_route")

    def test_unreadable_dependency_path_fails_closed_as_denied(self) -> None:
        result = emit_retrieval_package_from_source_file(
            ROOT / "tests/runtime/fixtures/not-present.md",
            request_id="slice3-004",
            source_ref="src-missing",
        )

        assert_schema_valid(self, result)
        self.assertEqual(result["state"], "denied")
        self.assertEqual(result["refusal"]["reason_class"], "unsupported_route")

    def test_malformed_upstream_input_is_denied(self) -> None:
        malformed = load_json(INVALID_UPSTREAM_FIXTURE)

        result = emit_retrieval_package_from_extraction_result(malformed)

        assert_schema_valid(self, result)
        self.assertEqual(result["state"], "denied")
        self.assertEqual(result["refusal"]["reason_class"], "unsupported_route")

    def test_stale_upstream_input_is_denied(self) -> None:
        stale = load_json(STALE_UPSTREAM_FIXTURE)

        result = emit_retrieval_package_from_extraction_result(stale)

        assert_schema_valid(self, result)
        self.assertEqual(result["state"], "denied")
        self.assertEqual(result["refusal"]["reason_class"], "stale_input")

    def test_output_remains_infrastructure_only(self) -> None:
        extraction_result = emit_extraction_result_from_source_file(
            SUPPORTED_SOURCE_FIXTURE,
            request_id="slice3-005",
            source_ref="src-bounded",
        )

        result = emit_retrieval_package_from_extraction_result(extraction_result)

        assert_schema_valid(self, result)
        self.assertNotIn("ranking", result)
        self.assertNotIn("best_chunk", result)
        self.assertNotIn("recommendations", result)
        self.assertNotIn("workflow_id", result)
        self.assertNotIn("dispatch_plan", result)
        self.assertEqual({chunk["structure_kind"] for chunk in result["chunks"]}, {"section"})

    def test_cli_entrypoint_emits_ready_json_for_direct_source(self) -> None:
        output = io.StringIO()
        with redirect_stdout(output):
            exit_code = main(
                [
                    "--source-path",
                    str(SUPPORTED_SOURCE_FIXTURE),
                    "--request-id",
                    "slice3-cli",
                    "--source-ref",
                    "src-cli",
                ]
            )

        result = json.loads(output.getvalue())
        assert_schema_valid(self, result)
        self.assertEqual(exit_code, 0)
        self.assertEqual(result["state"], "ready")

    def test_invalid_json_text_is_denied(self) -> None:
        result = emit_retrieval_package_from_extraction_json_text("{")

        assert_schema_valid(self, result)
        self.assertEqual(result["state"], "denied")
        self.assertEqual(result["refusal"]["reason_class"], "unsupported_route")


if __name__ == "__main__":
    unittest.main()
