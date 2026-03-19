from __future__ import annotations

import posixpath
from pathlib import PurePosixPath
from typing import Any
import zipfile
from xml.etree import ElementTree as ET


EPUB_MIMETYPE = "application/epub+zip"
PACKAGE_DOCUMENT_MIMETYPE = "application/oebps-package+xml"
ADMITTED_CONTENT_MEDIA_TYPES = {"application/xhtml+xml"}

_DENIED_TAGS = {
    "audio",
    "button",
    "canvas",
    "embed",
    "form",
    "iframe",
    "img",
    "input",
    "math",
    "nav",
    "object",
    "picture",
    "script",
    "select",
    "source",
    "svg",
    "textarea",
    "video",
}
_CONTAINER_TAGS = {
    "article",
    "aside",
    "body",
    "div",
    "footer",
    "header",
    "main",
    "section",
}
_HEADING_LEVELS = {
    "h1": 1,
    "h2": 2,
    "h3": 3,
    "h4": 4,
    "h5": 5,
    "h6": 6,
}
_TABLE_SECTION_TAGS = {"tbody", "tfoot", "thead"}
_TABLE_CELL_TAGS = {"td", "th"}
_INLINE_JAVASCRIPT_ATTRS = {"href", "src"}


class EpubDeniedError(ValueError):
    pass


class EpubUnavailableError(ValueError):
    pass


def _local_name(tag: str) -> str:
    if tag.startswith("{"):
        _, _, local = tag.rpartition("}")
        return local
    return tag


def _normalized_text(value: str) -> str:
    return " ".join(value.replace("\u00a0", " ").split())


def _literal_text(element: ET.Element) -> str:
    pieces: list[str] = []

    def walk(node: ET.Element) -> None:
        if node.text:
            pieces.append(node.text)
        for child in node:
            if not isinstance(child.tag, str):
                continue
            if _local_name(child.tag) == "br":
                pieces.append("\n")
            else:
                walk(child)
            if child.tail:
                pieces.append(child.tail)

    walk(element)
    return _normalized_text("".join(pieces))


def _contains_denied_markup(root: ET.Element) -> bool:
    for element in root.iter():
        if not isinstance(element.tag, str):
            continue

        tag_name = _local_name(element.tag)
        if tag_name in _DENIED_TAGS:
            return True

        for attr_name, attr_value in element.attrib.items():
            normalized_attr = _local_name(attr_name).lower()
            if normalized_attr.startswith("on"):
                return True
            if (
                normalized_attr in _INLINE_JAVASCRIPT_ATTRS
                and isinstance(attr_value, str)
                and attr_value.strip().lower().startswith("javascript:")
            ):
                return True

    return False


def _normalize_archive_path(base_dir: PurePosixPath, relative_path: str) -> str:
    raw_path = relative_path.split("#", 1)[0].split("?", 1)[0].strip()
    if not raw_path:
        raise EpubUnavailableError("empty package path")
    if raw_path.startswith("/"):
        raise EpubUnavailableError("absolute package paths are outside the bounded EPUB lane")

    normalized = posixpath.normpath(str(base_dir / raw_path))
    if normalized in {".", ""} or normalized.startswith("../"):
        raise EpubUnavailableError("package path escapes the bounded EPUB archive root")
    return normalized


def _require_single_package_document(epub_archive: zipfile.ZipFile) -> str:
    try:
        container_xml = epub_archive.read("META-INF/container.xml")
    except KeyError as exc:
        raise EpubUnavailableError("missing META-INF/container.xml") from exc

    try:
        container_root = ET.fromstring(container_xml)
    except ET.ParseError as exc:
        raise EpubUnavailableError("unparseable container.xml") from exc

    rootfiles: list[str] = []
    for element in container_root.iter():
        if not isinstance(element.tag, str) or _local_name(element.tag) != "rootfile":
            continue

        media_type = element.get("media-type")
        if media_type is not None and media_type != PACKAGE_DOCUMENT_MIMETYPE:
            continue

        full_path = element.get("full-path")
        if isinstance(full_path, str) and full_path.strip():
            rootfiles.append(_normalize_archive_path(PurePosixPath("."), full_path))

    if len(rootfiles) != 1:
        raise EpubUnavailableError("container.xml must identify exactly one package document")

    return rootfiles[0]


def _read_required_mimetype(epub_archive: zipfile.ZipFile) -> None:
    try:
        mimetype = epub_archive.read("mimetype").decode("utf-8", errors="replace").strip()
    except KeyError as exc:
        raise EpubUnavailableError("missing EPUB mimetype declaration") from exc

    if mimetype != EPUB_MIMETYPE:
        raise EpubDeniedError("package mimetype is outside the bounded EPUB lane")


def _spine_member_paths(epub_archive: zipfile.ZipFile) -> list[str]:
    package_path = _require_single_package_document(epub_archive)
    try:
        package_xml = epub_archive.read(package_path)
    except KeyError as exc:
        raise EpubUnavailableError("missing EPUB package document") from exc

    try:
        package_root = ET.fromstring(package_xml)
    except ET.ParseError as exc:
        raise EpubUnavailableError("unparseable package document") from exc

    manifest_element: ET.Element | None = None
    spine_element: ET.Element | None = None
    for child in package_root:
        if not isinstance(child.tag, str):
            continue
        tag_name = _local_name(child.tag)
        if tag_name == "manifest":
            manifest_element = child
        elif tag_name == "spine":
            spine_element = child

    if manifest_element is None:
        raise EpubUnavailableError("missing package manifest")
    if spine_element is None:
        raise EpubUnavailableError("missing package spine")

    package_dir = PurePosixPath(package_path).parent
    manifest_items: dict[str, dict[str, Any]] = {}
    for item in manifest_element:
        if not isinstance(item.tag, str) or _local_name(item.tag) != "item":
            continue

        item_id = item.get("id")
        href = item.get("href")
        media_type = item.get("media-type")
        if not item_id or not href or not media_type:
            raise EpubUnavailableError("manifest item is missing required authority fields")
        if item_id in manifest_items:
            raise EpubUnavailableError("manifest item identifiers must be unique")

        manifest_items[item_id] = {
            "media_type": media_type,
            "path": _normalize_archive_path(package_dir, href),
            "properties": {part for part in (item.get("properties") or "").split() if part},
        }

    spine_paths: list[str] = []
    for itemref in spine_element:
        if not isinstance(itemref.tag, str) or _local_name(itemref.tag) != "itemref":
            continue

        linear = (itemref.get("linear") or "yes").strip().lower()
        if linear == "no":
            continue
        if linear != "yes":
            raise EpubUnavailableError("spine linear attribute is malformed")

        idref = itemref.get("idref")
        if not idref or idref not in manifest_items:
            raise EpubUnavailableError("spine itemref does not resolve to manifest authority")

        manifest_item = manifest_items[idref]
        if "nav" in manifest_item["properties"]:
            raise EpubDeniedError("navigation documents are outside the bounded EPUB lane")
        if manifest_item["media_type"] not in ADMITTED_CONTENT_MEDIA_TYPES:
            raise EpubDeniedError("spine item is outside the bounded EPUB textual lane")

        spine_paths.append(manifest_item["path"])

    if not spine_paths:
        raise EpubUnavailableError("package spine has no admitted textual members")

    return spine_paths


def _container_has_block_children(element: ET.Element) -> bool:
    for child in element:
        if not isinstance(child.tag, str):
            continue
        child_name = _local_name(child.tag)
        if child_name in _CONTAINER_TAGS or child_name in _HEADING_LEVELS or child_name in {
            "ol",
            "p",
            "table",
            "ul",
        }:
            return True
    return False


def _simple_list_blocks(list_element: ET.Element) -> list[dict[str, Any]]:
    blocks: list[dict[str, Any]] = []
    for item in list_element:
        if not isinstance(item.tag, str):
            continue
        if _local_name(item.tag) != "li":
            continue

        if any(
            isinstance(descendant.tag, str) and _local_name(descendant.tag) in {"ol", "table", "ul"}
            for descendant in item.iter()
            if descendant is not item
        ):
            raise ValueError("nested EPUB list or table markup is outside the bounded lane")

        item_text = _literal_text(item)
        if item_text:
            blocks.append({"block_kind": "list", "text": item_text})

    return blocks


def _table_rows(table_element: ET.Element) -> list[ET.Element]:
    rows: list[ET.Element] = []
    for child in table_element:
        if not isinstance(child.tag, str):
            continue
        child_name = _local_name(child.tag)
        if child_name == "tr":
            rows.append(child)
            continue
        if child_name in _TABLE_SECTION_TAGS:
            rows.extend(
                grandchild
                for grandchild in child
                if isinstance(grandchild.tag, str) and _local_name(grandchild.tag) == "tr"
            )
    return rows


def _table_text(table_element: ET.Element) -> str:
    rows: list[str] = []
    for row in _table_rows(table_element):
        cells: list[str] = []
        for cell in row:
            if not isinstance(cell.tag, str) or _local_name(cell.tag) not in _TABLE_CELL_TAGS:
                continue
            if any(
                isinstance(descendant.tag, str) and _local_name(descendant.tag) in {"ol", "table", "ul"}
                for descendant in cell.iter()
                if descendant is not cell
            ):
                raise ValueError("nested EPUB table structure is outside the bounded lane")

            cells.append(_literal_text(cell))

        row_text = " | ".join(cell.strip() for cell in cells).strip()
        if row_text:
            rows.append(row_text)

    table_text = "\n".join(rows).strip()
    if len(table_text) > 20000:
        raise ValueError("literal content exceeds bounded extraction limits")
    return table_text


def _append_blocks(blocks: list[dict[str, Any]], container: ET.Element) -> int:
    tables_detected = 0
    for child in container:
        if not isinstance(child.tag, str):
            continue

        child_name = _local_name(child.tag)
        if child_name in _CONTAINER_TAGS:
            if _container_has_block_children(child):
                tables_detected += _append_blocks(blocks, child)
            else:
                text = _literal_text(child)
                if text:
                    blocks.append({"block_kind": "paragraph", "text": text})
            continue

        if child_name in _HEADING_LEVELS:
            text = _literal_text(child)
            if text:
                blocks.append(
                    {
                        "block_kind": "heading",
                        "text": text,
                        "level": _HEADING_LEVELS[child_name],
                    }
                )
            continue

        if child_name == "p":
            text = _literal_text(child)
            if text:
                blocks.append({"block_kind": "paragraph", "text": text})
            continue

        if child_name in {"ol", "ul"}:
            blocks.extend(_simple_list_blocks(child))
            continue

        if child_name == "table":
            table_text = _table_text(child)
            if table_text:
                blocks.append({"block_kind": "table", "text": table_text})
                tables_detected += 1
            continue

        if _literal_text(child):
            raise ValueError("unsupported EPUB top-level structure")

    return tables_detected


def _parse_content_document(content_bytes: bytes) -> dict[str, Any]:
    try:
        document_root = ET.fromstring(content_bytes)
    except ET.ParseError as exc:
        raise EpubUnavailableError("unparseable EPUB content document") from exc

    if _contains_denied_markup(document_root):
        raise EpubDeniedError("active or media markup is outside the bounded EPUB lane")

    body: ET.Element | None = None
    for element in document_root.iter():
        if isinstance(element.tag, str) and _local_name(element.tag) == "body":
            body = element
            break

    if body is None:
        raise EpubUnavailableError("EPUB content document is missing a body element")

    blocks: list[dict[str, Any]] = []
    tables_detected = _append_blocks(blocks, body)
    return {
        "blocks": blocks,
        "tables_detected": tables_detected,
    }


def extract_epub_surface(epub_archive: zipfile.ZipFile) -> dict[str, Any]:
    _read_required_mimetype(epub_archive)

    blocks: list[dict[str, Any]] = []
    tables_detected = 0
    for member_path in _spine_member_paths(epub_archive):
        try:
            content_bytes = epub_archive.read(member_path)
        except KeyError as exc:
            raise EpubUnavailableError("spine member is missing from the EPUB archive") from exc

        content_surface = _parse_content_document(content_bytes)
        blocks.extend(content_surface["blocks"])
        tables_detected += int(content_surface["tables_detected"])

    return {
        "blocks": blocks,
        "tables_detected": tables_detected,
    }
