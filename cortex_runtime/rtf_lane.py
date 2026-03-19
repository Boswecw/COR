from __future__ import annotations

import re
from dataclasses import dataclass


_HEX_ESCAPE = re.compile(r"[0-9a-fA-F]{2}")
_DENIED_DESTINATIONS = {
    "annotation",
    "comment",
    "atnid",
    "atnauthor",
    "pict",
    "object",
    "objdata",
    "field",
    "fldinst",
    "fldrslt",
    "footnote",
    "endnote",
    "header",
    "footer",
    "shp",
    "sp",
}
_SKIPPED_DESTINATIONS = {
    "fonttbl",
    "colortbl",
    "stylesheet",
    "info",
    "generator",
    "filetbl",
    "listtable",
    "listoverridetable",
    "revtbl",
    "rsidtbl",
}


class RtfDeniedError(ValueError):
    pass


class RtfUnavailableError(ValueError):
    pass


@dataclass
class _GroupState:
    skip: bool = False
    fresh: bool = True


def _decode_hex_escape(value: str) -> str:
    if not _HEX_ESCAPE.fullmatch(value):
        raise RtfUnavailableError("invalid RTF hex escape")
    return bytes([int(value, 16)]).decode("cp1252", errors="replace")


def _decode_unicode_escape(value: int) -> str:
    if value < 0:
        value += 65536
    try:
        return chr(value)
    except ValueError as exc:
        raise RtfUnavailableError("invalid RTF unicode escape") from exc


def _flush_paragraph(current_chars: list[str], paragraphs: list[str]) -> None:
    paragraph = "".join(current_chars)
    current_chars.clear()
    normalized = " ".join(paragraph.replace("\u00a0", " ").split())
    if normalized:
        paragraphs.append(normalized)


def _skip_unicode_fallback(text: str, index: int) -> int:
    if index >= len(text):
        return index
    if text[index] == "\\":
        if index + 1 < len(text) and text[index + 1] in "{}\\":
            return index + 2
        if index + 3 < len(text) and text[index + 1] == "'" and _HEX_ESCAPE.fullmatch(
            text[index + 2 : index + 4]
        ):
            return index + 4
    return index + 1


def extract_rtf_paragraphs(raw_bytes: bytes) -> list[str]:
    text = raw_bytes.decode("latin-1")
    if not text.lstrip().startswith("{\\rtf"):
        raise RtfUnavailableError("missing RTF header")

    paragraphs: list[str] = []
    current_chars: list[str] = []
    stack: list[_GroupState] = [_GroupState(skip=False, fresh=False)]
    pending_destination = False
    index = 0

    while index < len(text):
        char = text[index]

        if char == "{":
            stack.append(_GroupState(skip=stack[-1].skip, fresh=True))
            pending_destination = False
            index += 1
            continue

        if char == "}":
            if len(stack) == 1:
                raise RtfUnavailableError("unbalanced RTF group close")
            stack.pop()
            pending_destination = False
            index += 1
            continue

        if char == "\\":
            index += 1
            if index >= len(text):
                break

            control = text[index]
            if control in "{}\\":
                if not stack[-1].skip:
                    current_chars.append(control)
                stack[-1].fresh = False
                index += 1
                continue

            if control == "'":
                hex_value = text[index + 1 : index + 3]
                literal = _decode_hex_escape(hex_value)
                if not stack[-1].skip:
                    current_chars.append(literal)
                stack[-1].fresh = False
                index += 3
                continue

            if control == "*":
                pending_destination = True
                index += 1
                continue

            if not control.isalpha():
                if not stack[-1].skip:
                    if control == "~":
                        current_chars.append(" ")
                    elif control == "_":
                        current_chars.append("-")
                    elif control == "-":
                        pass
                stack[-1].fresh = False
                index += 1
                continue

            start = index
            while index < len(text) and text[index].isalpha():
                index += 1
            control_word = text[start:index]

            sign = 1
            if index < len(text) and text[index] in "+-":
                if text[index] == "-":
                    sign = -1
                index += 1

            number_start = index
            while index < len(text) and text[index].isdigit():
                index += 1
            number_value: int | None = None
            if number_start != index:
                number_value = sign * int(text[number_start:index])

            if index < len(text) and text[index] == " ":
                index += 1

            is_destination = pending_destination or (stack[-1].fresh and control_word in (_DENIED_DESTINATIONS | _SKIPPED_DESTINATIONS))
            pending_destination = False
            if is_destination:
                if control_word in _DENIED_DESTINATIONS:
                    raise RtfDeniedError("RTF destination is outside the bounded lane")
                if control_word in _SKIPPED_DESTINATIONS:
                    stack[-1].skip = True
                stack[-1].fresh = False
                continue

            if stack[-1].skip:
                stack[-1].fresh = False
                continue

            if control_word in {"par", "sect", "page"}:
                _flush_paragraph(current_chars, paragraphs)
            elif control_word == "line":
                current_chars.append("\n")
            elif control_word == "tab":
                current_chars.append("\t")
            elif control_word == "u":
                if number_value is None:
                    raise RtfUnavailableError("missing RTF unicode parameter")
                current_chars.append(_decode_unicode_escape(number_value))
                index = _skip_unicode_fallback(text, index)

            stack[-1].fresh = False
            continue

        if not stack[-1].skip:
            if char not in "\r\n":
                current_chars.append(char)
        if not char.isspace():
            stack[-1].fresh = False
        index += 1

    if len(stack) != 1:
        raise RtfUnavailableError("unbalanced RTF group open")

    _flush_paragraph(current_chars, paragraphs)
    return paragraphs
