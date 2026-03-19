from __future__ import annotations

import shutil
from dataclasses import dataclass
from pathlib import Path


PDF_INFO_COMMAND = "pdfinfo"
PDF_TO_TEXT_COMMAND = "pdftotext"


@dataclass(frozen=True)
class SourceLaneSpec:
    lane_id: str
    metadata_id: str
    suffix: str
    media_types: tuple[str, ...]
    operator_label: str
    runtime_slice_id: str | None = None
    runtime_slice_label: str | None = None
    dependency_summary: str | None = None


@dataclass(frozen=True)
class LaneEligibilityDecision:
    admitted: bool
    lane: SourceLaneSpec | None = None
    failure_state: str | None = None
    reason_class: str | None = None
    operator_visible_summary: str | None = None


def pdf_lane_runtime_available() -> bool:
    return shutil.which(PDF_INFO_COMMAND) is not None and shutil.which(PDF_TO_TEXT_COMMAND) is not None


def docx_lane_runtime_available() -> bool:
    return True


def odt_lane_runtime_available() -> bool:
    return True


MARKDOWN_LANE = SourceLaneSpec(
    lane_id="local_file_markdown",
    metadata_id="markdown_text",
    suffix=".md",
    media_types=("text/markdown",),
    operator_label="local Markdown files",
)

PLAIN_TEXT_LANE = SourceLaneSpec(
    lane_id="local_file_plain_text",
    metadata_id="plain_text",
    suffix=".txt",
    media_types=("text/plain",),
    operator_label="local plain-text files",
)

PDF_TEXT_LANE = SourceLaneSpec(
    lane_id="local_file_pdf_text",
    metadata_id="pdf_text",
    suffix=".pdf",
    media_types=("application/pdf",),
    operator_label="local text-layer PDF files",
    runtime_slice_id="slice5_pdf_source_lane",
    runtime_slice_label="bounded PDF source lane",
    dependency_summary="Extraction is unavailable because bounded local PDF text tooling is not present.",
)

RTF_TEXT_LANE = SourceLaneSpec(
    lane_id="local_file_rtf_text",
    metadata_id="rtf_text",
    suffix=".rtf",
    media_types=("application/rtf", "text/rtf"),
    operator_label="local RTF files",
    runtime_slice_id="slice7_rtf_source_lane",
    runtime_slice_label="bounded RTF source lane",
)

DOCX_TEXT_LANE = SourceLaneSpec(
    lane_id="local_file_docx_text",
    metadata_id="docx_text",
    suffix=".docx",
    media_types=("application/vnd.openxmlformats-officedocument.wordprocessingml.document",),
    operator_label="local DOCX files",
    runtime_slice_id="slice6_docx_source_lane",
    runtime_slice_label="bounded DOCX source lane",
)

ODT_TEXT_LANE = SourceLaneSpec(
    lane_id="local_file_odt_text",
    metadata_id="odt_text",
    suffix=".odt",
    media_types=("application/vnd.oasis.opendocument.text",),
    operator_label="local ODT files",
    runtime_slice_id="slice8_odt_source_lane",
    runtime_slice_label="bounded ODT source lane",
)

ALL_SOURCE_LANES = (
    MARKDOWN_LANE,
    PLAIN_TEXT_LANE,
    PDF_TEXT_LANE,
    DOCX_TEXT_LANE,
    RTF_TEXT_LANE,
    ODT_TEXT_LANE,
)

SOURCE_LANE_LABELS = {lane.lane_id: lane.operator_label for lane in ALL_SOURCE_LANES}
RUNTIME_SLICE_LABELS = {
    lane.runtime_slice_id: lane.runtime_slice_label
    for lane in ALL_SOURCE_LANES
    if lane.runtime_slice_id is not None and lane.runtime_slice_label is not None
}
_LANE_BY_SUFFIX = {lane.suffix: lane for lane in ALL_SOURCE_LANES}
_LANE_BY_ID = {lane.lane_id: lane for lane in ALL_SOURCE_LANES}
_LANE_BY_SLICE_ID = {
    lane.runtime_slice_id: lane for lane in ALL_SOURCE_LANES if lane.runtime_slice_id is not None
}


def configured_source_lane_specs() -> tuple[SourceLaneSpec, ...]:
    return ALL_SOURCE_LANES


def configured_source_lane_slice_ids() -> list[str]:
    return [
        lane.runtime_slice_id
        for lane in ALL_SOURCE_LANES
        if lane.runtime_slice_id is not None
    ]


def configured_supported_suffixes() -> set[str]:
    return {lane.suffix for lane in ALL_SOURCE_LANES}


def configured_supported_media_types() -> set[str]:
    media_types: set[str] = set()
    for lane in ALL_SOURCE_LANES:
        media_types.update(lane.media_types)
    return media_types


def supported_suffix_list_text() -> str:
    ordered_suffixes = [lane.suffix for lane in ALL_SOURCE_LANES]
    if len(ordered_suffixes) == 1:
        return ordered_suffixes[0]
    return ", ".join(ordered_suffixes[:-1]) + ", and " + ordered_suffixes[-1]


def source_lane_spec_for_path(source_path: str | Path) -> SourceLaneSpec | None:
    return _LANE_BY_SUFFIX.get(Path(source_path).suffix.lower())


def source_lane_spec_for_lane_id(lane_id: str) -> SourceLaneSpec | None:
    return _LANE_BY_ID.get(lane_id)


def source_lane_slice_available(slice_id: str) -> bool:
    lane = _LANE_BY_SLICE_ID.get(slice_id)
    if lane is None:
        return False
    if lane.lane_id == PDF_TEXT_LANE.lane_id:
        return pdf_lane_runtime_available()
    if lane.lane_id == DOCX_TEXT_LANE.lane_id:
        return docx_lane_runtime_available()
    if lane.lane_id == ODT_TEXT_LANE.lane_id:
        return odt_lane_runtime_available()
    return True


def admitted_lane_specs() -> list[SourceLaneSpec]:
    lanes: list[SourceLaneSpec] = []
    for lane in ALL_SOURCE_LANES:
        if lane.runtime_slice_id is not None and not source_lane_slice_available(lane.runtime_slice_id):
            continue
        lanes.append(lane)
    return lanes


def admitted_source_lanes() -> list[str]:
    return [lane.lane_id for lane in admitted_lane_specs()]


def implemented_source_lane_slices() -> list[str]:
    return [
        lane.runtime_slice_id
        for lane in admitted_lane_specs()
        if lane.runtime_slice_id is not None
    ]


def lane_eligibility_for_path(
    source_path: str | Path,
    *,
    media_type: str | None = None,
) -> LaneEligibilityDecision:
    lane = source_lane_spec_for_path(source_path)
    if lane is None:
        return LaneEligibilityDecision(
            admitted=False,
            failure_state="denied",
            reason_class="unsupported_source_type",
            operator_visible_summary=(
                "Extraction is denied because only bounded local "
                + supported_suffix_list_text()
                + " sources are supported."
            ),
        )

    if media_type is not None and media_type not in lane.media_types:
        return LaneEligibilityDecision(
            admitted=False,
            lane=lane,
            failure_state="denied",
            reason_class="unsupported_source_type",
            operator_visible_summary=(
                "Extraction is denied because the declared media type is outside the bounded "
                f"{lane.operator_label[:-1] if lane.operator_label.endswith('s') else lane.operator_label} lane."
            ),
        )

    if lane.runtime_slice_id is not None and not source_lane_slice_available(lane.runtime_slice_id):
        return LaneEligibilityDecision(
            admitted=False,
            lane=lane,
            failure_state="unavailable",
            reason_class="dependency_unavailable",
            operator_visible_summary=lane.dependency_summary
            or "Extraction is unavailable because the bounded source lane is not currently available.",
        )

    return LaneEligibilityDecision(admitted=True, lane=lane)
