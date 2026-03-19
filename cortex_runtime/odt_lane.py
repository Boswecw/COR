from __future__ import annotations

from typing import Any
import zipfile
from xml.etree import ElementTree as ET


ODT_TEXT_MIMETYPE = "application/vnd.oasis.opendocument.text"

_OFFICE_NS = "urn:oasis:names:tc:opendocument:xmlns:office:1.0"
_TEXT_NS = "urn:oasis:names:tc:opendocument:xmlns:text:1.0"
_TABLE_NS = "urn:oasis:names:tc:opendocument:xmlns:table:1.0"
_DRAW_NS = "urn:oasis:names:tc:opendocument:xmlns:drawing:1.0"


def _tag(namespace: str, name: str) -> str:
    return f"{{{namespace}}}{name}"


def _attr(namespace: str, name: str) -> str:
    return f"{{{namespace}}}{name}"


_OFFICE_BODY = _tag(_OFFICE_NS, "body")
_OFFICE_TEXT = _tag(_OFFICE_NS, "text")
_OFFICE_ANNOTATION = _tag(_OFFICE_NS, "annotation")
_TEXT_H = _tag(_TEXT_NS, "h")
_TEXT_P = _tag(_TEXT_NS, "p")
_TEXT_LIST = _tag(_TEXT_NS, "list")
_TEXT_LIST_ITEM = _tag(_TEXT_NS, "list-item")
_TEXT_SECTION = _tag(_TEXT_NS, "section")
_TEXT_S = _tag(_TEXT_NS, "s")
_TEXT_TAB = _tag(_TEXT_NS, "tab")
_TEXT_LINE_BREAK = _tag(_TEXT_NS, "line-break")
_TEXT_OUTLINE_LEVEL = _attr(_TEXT_NS, "outline-level")
_TEXT_SPACE_COUNT = _attr(_TEXT_NS, "c")
_TEXT_TRACKED_CHANGES = _tag(_TEXT_NS, "tracked-changes")
_TEXT_CHANGE = _tag(_TEXT_NS, "change")
_TEXT_CHANGE_START = _tag(_TEXT_NS, "change-start")
_TEXT_CHANGE_END = _tag(_TEXT_NS, "change-end")
_TABLE_TABLE = _tag(_TABLE_NS, "table")
_TABLE_ROW = _tag(_TABLE_NS, "table-row")
_TABLE_CELL = _tag(_TABLE_NS, "table-cell")
_TABLE_COVERED_CELL = _tag(_TABLE_NS, "covered-table-cell")
_DRAW_OBJECT = _tag(_DRAW_NS, "object")
_DRAW_OBJECT_OLE = _tag(_DRAW_NS, "object-ole")
_DRAW_IMAGE = _tag(_DRAW_NS, "image")
_DRAW_PLUGIN = _tag(_DRAW_NS, "plugin")
_DRAW_APPLET = _tag(_DRAW_NS, "applet")
_DRAW_FRAME = _tag(_DRAW_NS, "frame")

_DENIED_TAGS = (
    _OFFICE_ANNOTATION,
    _TEXT_TRACKED_CHANGES,
    _TEXT_CHANGE,
    _TEXT_CHANGE_START,
    _TEXT_CHANGE_END,
    _DRAW_OBJECT,
    _DRAW_OBJECT_OLE,
    _DRAW_IMAGE,
    _DRAW_PLUGIN,
    _DRAW_APPLET,
    _DRAW_FRAME,
)


class OdtDeniedError(ValueError):
    pass


class OdtUnavailableError(ValueError):
    pass


def _normalized_text(value: str) -> str:
    return " ".join(value.replace("\u00a0", " ").split())


def _literal_text(element: ET.Element) -> str:
    pieces: list[str] = []

    def walk(node: ET.Element) -> None:
        if node.text:
            pieces.append(node.text)
        for child in node:
            if child.tag == _TEXT_S:
                raw_count = child.get(_TEXT_SPACE_COUNT) or "1"
                try:
                    count = int(raw_count)
                except ValueError as exc:
                    raise OdtUnavailableError("invalid ODT space count") from exc
                pieces.append(" " * max(1, count))
            elif child.tag == _TEXT_TAB:
                pieces.append("\t")
            elif child.tag == _TEXT_LINE_BREAK:
                pieces.append("\n")
            else:
                walk(child)
            if child.tail:
                pieces.append(child.tail)

    walk(element)
    return _normalized_text("".join(pieces))


def _has_denied_markup(root: ET.Element) -> bool:
    return any(root.find(f".//{tag}") is not None for tag in _DENIED_TAGS)


def _heading_level(element: ET.Element) -> int | None:
    raw_level = element.get(_TEXT_OUTLINE_LEVEL)
    if raw_level is None:
        return None
    try:
        level = int(raw_level)
    except ValueError:
        return None
    if 1 <= level <= 8:
        return level
    return None


def _simple_list_blocks(list_element: ET.Element) -> list[dict[str, Any]]:
    blocks: list[dict[str, Any]] = []
    for item in list_element.findall(_TEXT_LIST_ITEM):
        if item.find(f"./{_TEXT_LIST}") is not None:
            raise ValueError("nested ODT lists are outside the bounded lane")

        paragraphs: list[str] = []
        for child in item:
            if child.tag == _TEXT_P:
                text = _literal_text(child)
                if text:
                    paragraphs.append(text)
                continue
            if child.tag == _TEXT_SECTION:
                raise ValueError("section-wrapped list items are outside the bounded lane")
            if child.tag == _TABLE_TABLE:
                raise ValueError("tables inside list items are outside the bounded lane")
            if _literal_text(child):
                raise ValueError("unsupported ODT list item content")

        item_text = "\n".join(paragraphs).strip()
        if item_text:
            blocks.append({"block_kind": "list", "text": item_text})
    return blocks


def _table_text(table_element: ET.Element) -> str:
    rows: list[str] = []
    for row in table_element.findall(_TABLE_ROW):
        cells: list[str] = []
        for cell in row:
            if cell.tag not in {_TABLE_CELL, _TABLE_COVERED_CELL}:
                continue
            if cell.find(f".//{_TABLE_TABLE}") is not None:
                raise ValueError("nested ODT tables are outside the bounded lane")

            cell_paragraphs: list[str] = []
            for child in cell:
                if child.tag in {_TEXT_P, _TEXT_H}:
                    text = _literal_text(child)
                    if text:
                        cell_paragraphs.append(text)
                    continue
                if child.tag == _TEXT_LIST:
                    raise ValueError("lists inside ODT tables are outside the bounded lane")
                if _literal_text(child):
                    raise ValueError("unsupported ODT table cell content")

            cells.append(" / ".join(cell_paragraphs).strip())

        row_text = " | ".join(cells).strip()
        if row_text:
            rows.append(row_text)

    return "\n".join(rows).strip()


def _append_blocks(blocks: list[dict[str, Any]], container: ET.Element) -> int:
    tables_detected = 0
    for child in container:
        if child.tag == _TEXT_SECTION:
            tables_detected += _append_blocks(blocks, child)
            continue

        if child.tag == _TEXT_H:
            text = _literal_text(child)
            if not text:
                continue
            level = _heading_level(child)
            if level is None:
                blocks.append({"block_kind": "paragraph", "text": text})
            else:
                blocks.append({"block_kind": "heading", "text": text, "level": level})
            continue

        if child.tag == _TEXT_P:
            text = _literal_text(child)
            if text:
                blocks.append({"block_kind": "paragraph", "text": text})
            continue

        if child.tag == _TEXT_LIST:
            blocks.extend(_simple_list_blocks(child))
            continue

        if child.tag == _TABLE_TABLE:
            table_text = _table_text(child)
            if table_text:
                blocks.append({"block_kind": "table", "text": table_text})
                tables_detected += 1
            continue

        if _literal_text(child):
            raise ValueError("unsupported top-level ODT structure")

    return tables_detected


def extract_odt_surface(odt_archive: zipfile.ZipFile) -> dict[str, Any]:
    try:
        mimetype = odt_archive.read("mimetype").decode("utf-8", errors="replace").strip()
        if mimetype and mimetype != ODT_TEXT_MIMETYPE:
            raise OdtDeniedError("package mimetype is outside the bounded ODT lane")
    except KeyError:
        pass

    try:
        content_xml = odt_archive.read("content.xml")
    except KeyError as exc:
        raise OdtUnavailableError("missing content.xml") from exc

    try:
        root = ET.fromstring(content_xml)
    except ET.ParseError as exc:
        raise OdtUnavailableError("unparseable content.xml") from exc

    if _has_denied_markup(root):
        raise OdtDeniedError("annotation, review, or embedded-object markup is outside the bounded lane")

    body = root.find(_OFFICE_BODY)
    if body is None:
        raise OdtUnavailableError("missing office:body")
    office_text = body.find(_OFFICE_TEXT)
    if office_text is None:
        raise OdtUnavailableError("missing office:text")

    blocks: list[dict[str, Any]] = []
    tables_detected = _append_blocks(blocks, office_text)
    return {
        "blocks": blocks,
        "tables_detected": tables_detected,
    }
