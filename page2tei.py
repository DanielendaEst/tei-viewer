#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
PAGE XML -> TEI P5 converter with interactive metadata prompts and full tag support.

Features:
- Auto-detects edition type (diplomatic/translation) from filename pattern
- Provides preset metadata for diplomatic vs translation editions
- Reads PAGE-XML and writes TEI P5 with comprehensive tag support
- Builds teiHeader with titleStmt, editionStmt, publicationStmt, msDesc
- Maps PAGE TextRegion/TextLine to TEI facsimile/surface/zone with polygons
- Links text lines via <lb facs="#zone"> with line numbers
- Applies PAGE TextLine @custom inline annotations:
    abbrev{offset;length;expansion} -> <choice><abbr>…</abbr><expan>…</expan></choice>
    sic{offset;length;correction}   -> <choice><sic>…</sic><corr>…</corr></choice>
    regularised{offset;length;original} -> <choice><orig>…</orig><reg>…</reg></choice>
    num{offset;length;type;value}   -> <num type="…" value="…">…</num>
    person{...}                      -> <persName type="…" ref="…">…</persName>
    place{...}                       -> <placeName><country>…</country></placeName>
    ref{offset;length;type;target}  -> <ref type="…" target="…">…</ref>
    unclear{offset;length;reason}   -> <unclear reason="…">…</unclear>
    textStyle{bold/italic/underline/superscript/subscript} -> <hi rend="…">…</hi>
- Automatically prefixes image paths with "images/"
- Language-aware based on edition type (grc for diplomatic, es for translation)

CLI:
  page2tei.py --input in.xml --output out.xml [--non-interactive flags...]
  Use "-" for stdin/stdout. Without flags, the script will prompt interactively.
"""

import argparse
import os
import re
import string
import sys
import xml.etree.ElementTree as ET
from typing import Any, Dict, List, Optional
from xml.dom import minidom

PAGE_NS = {"pg": "http://schema.primaresearch.org/PAGE/gts/pagecontent/2013-07-15"}
TEI_NS = "http://www.tei-c.org/ns/1.0"
XML_NS = "http://www.w3.org/XML/1998/namespace"
ET.register_namespace("", TEI_NS)

# -------------------------
# Utilities
# -------------------------


def parse_bool(val) -> bool:
    """Parse boolean from string."""
    return str(val).strip().lower() in ("1", "true", "yes", "y")


def parse_int(val, default=0) -> int:
    """Parse integer with fallback."""
    try:
        return int(val)
    except Exception:
        return default


def qn(name: str) -> ET.QName:
    """Create qualified name in TEI namespace."""
    return ET.QName(TEI_NS, name)


def scale_points(points_str: str, sx: float, sy: float) -> str:
    """Scale a PAGE points string like 'x1,y1 x2,y2' by sx/sy.

    Returns scaled points string with integer coordinates.
    """
    if not points_str:
        return points_str
    parts = []
    for pair in points_str.split():
        if "," not in pair:
            continue
        try:
            x_s, y_s = pair.split(",")
            x = round(float(x_s) * sx)
            y = round(float(y_s) * sy)
            parts.append(f"{x},{y}")
        except Exception:
            parts.append(pair)
    return " ".join(parts)


def prettify(elem: ET.Element) -> str:
    """Serialize XML element compactly (no added indentation/newlines).

    Use ElementTree's serialization to avoid introducing indentation text nodes
    that can become spurious whitespace in downstream processing.
    """
    return ET.tostring(elem, encoding="utf-8", method="xml").decode("utf-8")


def detect_edition_type(filename: str) -> Optional[str]:
    """
    Auto-detect edition type from filename pattern.
    Returns 'diplomatic', 'translation', or None.
    """
    basename = os.path.basename(filename).lower()
    if "_dip" in basename or "diplomatic" in basename:
        return "diplomatic"
    elif "_trad" in basename or "translation" in basename or "trans" in basename:
        return "translation"
    return None


# -------------------------
# PAGE custom parsing
# -------------------------


def parse_custom_ops(custom_str: str) -> List[Dict[str, Any]]:
    """
    Parse PAGE @custom like:
      'readingOrder {index:3;} abbrev {offset:9;length:2;expansion:...;}
       person {offset:X;length:Y;firstname:Name;type:humano;wikiData:Q...;}'
    Return list of ops with keys: kind, offset, length, end, plus additional attributes.
    """
    ops = []
    if not custom_str:
        return ops

    # Match pattern: word{key:value;key:value;}
    for m in re.finditer(r"(\w+)\s*\{([^}]*)\}", custom_str):
        kind = m.group(1)
        body = m.group(2)
        kv = {}

        # Parse key:value pairs
        for part in re.finditer(r"(\w+)\s*:\s*([^;]+);", body):
            kv[part.group(1)] = part.group(2).strip()

        off = parse_int(kv.get("offset", "0"), 0)
        length = parse_int(kv.get("length", "0"), 0)
        end = off + length

        op = {"kind": kind, "offset": off, "length": length, "end": end}

        # Add all other attributes
        for k, v in kv.items():
            if k in ("offset", "length"):
                continue
            op[k] = v

        # Special handling for textStyle
        if kind == "textStyle":
            for k in ("bold", "italic", "underline", "superscript", "subscript"):
                if k in op:
                    op[k] = "true" if parse_bool(op[k]) else "false"

        # Normalize common boolean-like attributes such as 'continued' so downstream
        # logic can treat them uniformly as 'true'/'false' strings.
        if "continued" in op:
            op["continued"] = "true" if parse_bool(op["continued"]) else "false"

        ops.append(op)

    return ops


# -------------------------
# Inline TEI builders
# -------------------------


def slice_text_nodes(text: str, ranges: List[tuple]) -> List[tuple]:
    """
    Slice text into segments based on annotation ranges.
    ranges: list of (start, end, label)
    returns list of (start, end, segment_text, labels_set)
    """
    cut_points = {0, len(text)}
    for s, e, _ in ranges:
        cut_points.add(max(0, min(len(text), s)))
        cut_points.add(max(0, min(len(text), e)))

    cuts = sorted(cut_points)
    segments = []

    for i in range(len(cuts) - 1):
        s, e = cuts[i], cuts[i + 1]
        if s == e:
            continue
        segment = text[s:e]
        labels = {lab for ss, ee, lab in ranges if ss <= s and e <= ee and ss < ee}
        segments.append((s, e, segment, labels))

    return segments


def append_text(parent: ET.Element, s: str):
    """Append text to element, handling text vs tail."""
    if not s:
        return
    if parent.text is None and len(parent) == 0:
        parent.text = s
    else:
        if len(parent):
            last = list(parent)[-1]
            last.tail = (last.tail or "") + s
        else:
            parent.text = (parent.text or "") + s


def build_styled_nodes(parent: ET.Element, text: str, style_ops: List[Dict[str, Any]]):
    """Build text with <hi rend="..."> styling.

    Avoid creating <hi> elements for spans that consist only of punctuation
    (possibly surrounded by whitespace). Punctuation-only <hi> elements can
    produce unnecessary element boundaries that split inline flow and may
    lead to unintended line breaks when serialized/rendered.
    """
    ranges = []
    for op in style_ops:
        rend = []
        if op.get("bold", "false") == "true":
            rend.append("bold")
        if op.get("italic", "false") == "true":
            rend.append("italic")
        if op.get("underline", "false") == "true":
            rend.append("underline")
        if op.get("superscript", "false") == "true":
            rend.append("superscript")
        if op.get("subscript", "false") == "true":
            rend.append("subscript")
        if not rend:
            continue
        ranges.append((op["offset"], op["end"], " ".join(rend)))

    if not ranges:
        append_text(parent, text)
        return

    for s, e, seg_text, labels in slice_text_nodes(text, ranges):
        if not labels:
            append_text(parent, seg_text)
        else:
            # If the segment (ignoring surrounding whitespace) consists only of
            # punctuation characters, do not wrap it in a <hi> element. Instead,
            # append it as plain text to avoid creating element boundaries that
            # can split inline content across tags and cause layout/line-break
            # issues.
            seg_stripped = seg_text.strip()
            if seg_stripped and all(ch in string.punctuation for ch in seg_stripped):
                append_text(parent, seg_text)
            else:
                hi = ET.Element(qn("hi"))
                hi.set("rend", " ".join(sorted(labels)))
                hi.text = seg_text
                parent.append(hi)


def build_choice_with_styles(
    kind: str,
    witness_text: str,
    alt_text: str,
    inner_style_ops: List[Dict[str, Any]],
    global_offset: int,
) -> ET.Element:
    """Build <choice> element with abbreviation, correction, or regularisation."""
    choice = ET.Element(qn("choice"))

    if kind == "abbrev":
        a_tag = "abbr"
        b_tag = "expan"
        first_text = witness_text
        second_text = alt_text
    elif kind == "sic":
        a_tag = "sic"
        b_tag = "corr"
        first_text = witness_text
        second_text = alt_text
    elif kind == "regularised":
        # For regularised, witness_text is the regularised form (goes in <reg>)
        # and alt_text (from 'original' attribute) is the original form (goes in <orig>)
        a_tag = "orig"
        b_tag = "reg"
        first_text = alt_text  # original goes first in <orig>
        second_text = witness_text  # regularised goes second in <reg>
    else:
        a_tag = "orig"
        b_tag = "reg"
        first_text = witness_text
        second_text = alt_text

    a = ET.SubElement(choice, qn(a_tag))

    # Adjust styles relative to this span
    adj_styles = []
    for op in inner_style_ops:
        if op["offset"] >= global_offset and op["end"] <= global_offset + len(
            witness_text
        ):
            cp = dict(op)
            cp["offset"] = cp["offset"] - global_offset
            cp["end"] = cp["end"] - global_offset
            adj_styles.append(cp)

    # For regularised, styles apply to the second element (reg), not the first (orig)
    if kind == "regularised":
        a.text = first_text or ""
        b = ET.SubElement(choice, qn(b_tag))
        build_styled_nodes(b, second_text, adj_styles)
    else:
        build_styled_nodes(a, first_text, adj_styles)
        b = ET.SubElement(choice, qn(b_tag))
        b.text = second_text or ""

    return choice


def build_num_with_styles(
    witness_text: str,
    num_op: Dict[str, Any],
    inner_style_ops: List[Dict[str, Any]],
    global_offset: int,
) -> ET.Element:
    """Build <num> element with type and value."""
    num = ET.Element(qn("num"))
    if "type" in num_op:
        num.set("type", num_op["type"])
    if "value" in num_op:
        num.set("value", num_op["value"])

    adj_styles = []
    for op in inner_style_ops:
        if op["offset"] >= global_offset and op["end"] <= global_offset + len(
            witness_text
        ):
            cp = dict(op)
            cp["offset"] = cp["offset"] - global_offset
            cp["end"] = cp["end"] - global_offset
            adj_styles.append(cp)

    build_styled_nodes(num, witness_text, adj_styles)
    return num


def build_persName(
    witness_text: str,
    person_op: Dict[str, Any],
    inner_style_ops: List[Dict[str, Any]],
    global_offset: int,
) -> ET.Element:
    """Build <persName> element with type, optional ref, firstname and continued flag."""
    persName = ET.Element(qn("persName"))

    # Type is always set
    if "type" in person_op:
        persName.set("type", person_op["type"])

    # Ref is optional (for wikiData)
    if "wikiData" in person_op:
        wikidata_id = person_op["wikiData"]
        persName.set("ref", f"https://www.wikidata.org/wiki/{wikidata_id}")

    # Preserve firstname attribute if present (keep as attribute so it survives
    # even when the element is continued across lines)
    if "firstname" in person_op:
        persName.set("firstname", person_op["firstname"])

    # Preserve and normalize continued flag (string 'true'/'false')
    if "continued" in person_op:
        persName.set(
            "continued", "true" if parse_bool(person_op["continued"]) else "false"
        )

    adj_styles = []
    for op in inner_style_ops:
        if op["offset"] >= global_offset and op["end"] <= global_offset + len(
            witness_text
        ):
            cp = dict(op)
            cp["offset"] = cp["offset"] - global_offset
            cp["end"] = cp["end"] - global_offset
            adj_styles.append(cp)

    build_styled_nodes(persName, witness_text, adj_styles)
    return persName


def build_placeName(
    witness_text: str,
    place_op: Dict[str, Any],
    inner_style_ops: List[Dict[str, Any]],
    global_offset: int,
) -> ET.Element:
    """Build <placeName> element with nested attributes like <country> while preserving the original text."""
    placeName = ET.Element(qn("placeName"))

    # Always preserve the original text as the main content
    adj_styles = []
    for op in inner_style_ops:
        if op["offset"] >= global_offset and op["end"] <= global_offset + len(
            witness_text
        ):
            cp = dict(op)
            cp["offset"] = cp["offset"] - global_offset
            cp["end"] = cp["end"] - global_offset
            adj_styles.append(cp)
    build_styled_nodes(placeName, witness_text, adj_styles)

    # Append nested attributes as child elements
    nested_attrs = ["country", "region", "settlement", "district"]
    for attr in nested_attrs:
        if attr in place_op:
            child = ET.SubElement(placeName, qn(attr))
            child.text = place_op[attr]

    return placeName


def build_ref(
    witness_text: str,
    ref_op: Dict[str, Any],
    inner_style_ops: List[Dict[str, Any]],
    global_offset: int,
) -> ET.Element:
    """Build <ref> element with type and target."""
    ref = ET.Element(qn("ref"))

    if "type" in ref_op:
        ref.set("type", ref_op["type"])
    if "target" in ref_op:
        ref.set("target", ref_op["target"])

    adj_styles = []
    for op in inner_style_ops:
        if op["offset"] >= global_offset and op["end"] <= global_offset + len(
            witness_text
        ):
            cp = dict(op)
            cp["offset"] = cp["offset"] - global_offset
            cp["end"] = cp["end"] - global_offset
            adj_styles.append(cp)

    build_styled_nodes(ref, witness_text, adj_styles)
    return ref


def build_unclear(
    witness_text: str,
    unclear_op: Dict[str, Any],
    inner_style_ops: List[Dict[str, Any]],
    global_offset: int,
) -> ET.Element:
    """Build <unclear> element with reason attribute."""
    unclear = ET.Element(qn("unclear"))

    if "reason" in unclear_op:
        unclear.set("reason", unclear_op["reason"])

    adj_styles = []
    for op in inner_style_ops:
        if op["offset"] >= global_offset and op["end"] <= global_offset + len(
            witness_text
        ):
            cp = dict(op)
            cp["offset"] = cp["offset"] - global_offset
            cp["end"] = cp["end"] - global_offset
            adj_styles.append(cp)

    build_styled_nodes(unclear, witness_text, adj_styles)
    return unclear


def build_fallback_seg(
    witness_text: str,
    op: Dict[str, Any],
    inner_style_ops: List[Dict[str, Any]],
    global_offset: int,
) -> ET.Element:
    """Build generic <seg> for unknown tag types."""
    seg = ET.Element(qn("seg"))
    seg.set("type", op.get("kind", "custom"))

    # Echo unknown attributes as data-* attributes
    for key, value in op.items():
        if key not in {"kind", "offset", "length", "end"}:
            seg.set(f"data-{key}", value)

    # Apply styles to the witness text
    adj_styles = []
    for style_op in inner_style_ops:
        if style_op["offset"] >= global_offset and style_op[
            "end"
        ] <= global_offset + len(witness_text):
            adjusted_op = dict(style_op)
            adjusted_op["offset"] -= global_offset
            adjusted_op["end"] -= global_offset
            adj_styles.append(adjusted_op)

    build_styled_nodes(seg, witness_text, adj_styles)

    return seg
    for k, v in op.items():
        if k in ("kind", "offset", "length", "end"):
            continue
        seg.set(f"data-{k}", v)

    adj_styles = []
    for s in inner_style_ops:
        if s["offset"] >= global_offset and s["end"] <= global_offset + len(
            witness_text
        ):
            cp = dict(s)
            cp["offset"] = cp["offset"] - global_offset
            cp["end"] = cp["end"] - global_offset
            adj_styles.append(cp)

    build_styled_nodes(seg, witness_text, adj_styles)
    return seg


def build_inline_nodes_for_line(text: str, ops: List[Dict[str, Any]]) -> List[Any]:
    """
    Build inline TEI nodes from text and operations.
    Returns list of strings and ET.Elements.
    """
    # Separate operations by type
    choice_ops = [
        o
        for o in ops
        if o["kind"] in ("abbrev", "sic", "regularised") and o["length"] > 0
    ]
    num_ops = [o for o in ops if o["kind"] == "num" and o["length"] > 0]
    person_ops = [o for o in ops if o["kind"] == "person" and o["length"] > 0]
    place_ops = [o for o in ops if o["kind"] == "place" and o["length"] > 0]
    ref_ops = [o for o in ops if o["kind"] == "ref" and o["length"] > 0]
    unclear_ops = [o for o in ops if o["kind"] == "unclear" and o["length"] > 0]
    style_ops = [o for o in ops if o["kind"] == "textStyle" and o["length"] > 0]

    # Known wrapper types
    known_kinds = {
        "abbrev",
        "sic",
        "regularised",
        "num",
        "person",
        "place",
        "ref",
        "unclear",
        "textStyle",
    }
    other_ops = [o for o in ops if o["kind"] not in known_kinds and o["length"] > 0]

    # Primary wrapper ops in document order (start asc, end desc for nesting)
    # Priority: ensure certain semantic wrappers (person/place/ref/unclear) are
    # considered outermost when they share the exact same span as others.
    priority = {
        "person": 0,
        "place": 0,
        "ref": 0,
        "unclear": 0,
        "abbrev": 1,
        "sic": 1,
        "regularised": 1,
        "num": 1,
    }

    def op_priority(kind: str) -> int:
        return priority.get(kind, 2)

    # Make wrapper ordering deterministic: put semantic wrappers first so that
    # when multiple ops share the exact same span a semantic wrapper (e.g.
    # person/place/ref/unclear) will wrap choice/abbrev rather than being
    # nested inside it. This ensures the TEI reflects entity markup outside of
    # abbreviation choice elements.
    wrappers = sorted(
        person_ops
        + place_ops
        + ref_ops
        + unclear_ops
        + choice_ops
        + num_ops
        + other_ops,
        key=lambda x: (x["offset"], -x["end"], op_priority(x["kind"])),
    )

    # Build containment tree so nested wrappers are preserved instead of discarded.
    # Each wrapper knows its children (wrappers fully contained within it).
    children = {i: [] for i in range(len(wrappers))}
    roots: List[int] = []

    for i, w in enumerate(wrappers):
        parent_idx = None
        # find the nearest enclosing wrapper (if any)
        for j in range(i - 1, -1, -1):
            pw = wrappers[j]
            if pw["offset"] <= w["offset"] and pw["end"] >= w["end"]:
                parent_idx = j
                break
        if parent_idx is None:
            roots.append(i)
        else:
            children[parent_idx].append(i)

    def render_plain_segment(seg_start: int, seg_end: int) -> List[Any]:
        """Render a plain text segment (no wrapper boundaries inside) with styles."""
        seg_nodes: List[Any] = []
        seg_text = text[seg_start:seg_end]
        if not seg_text:
            return seg_nodes
        tmp = ET.Element("tmp")
        seg_styles = [
            s for s in style_ops if seg_start <= s["offset"] and s["end"] <= seg_end
        ]
        build_styled_nodes(tmp, seg_text, seg_styles)
        if tmp.text:
            seg_nodes.append(tmp.text)
        for ch in list(tmp):
            seg_nodes.append(ch)
            if ch.tail:
                seg_nodes.append(ch.tail)
                ch.tail = None
        return seg_nodes

    def build_wrapper_element(idx: int, inner_nodes: List[Any]) -> Any:
        """Create wrapper element for wrappers[idx] and populate with inner_nodes.

        To avoid including surrounding whitespace inside wrapper elements
        (which makes later processing and comparisons brittle), this function
        uses a helper that returns a tuple (lead, element, trail) when the
        inserted content has leading or trailing whitespace. Callers must
        accept either an Element or a (lead, Element, trail) tuple and place
        lead/trail outside the wrapper.
        """
        w = wrappers[idx]
        kind = w["kind"]
        witness_text = text[w["offset"] : w["end"]]

        def populate_target_with_nodes(target: ET.Element, nodes_to_insert: List[Any]):
            # Insert nodes (strings and elements) into a temporary container
            # preserving text/tails, then extract any leading/trailing
            # whitespace so it can be placed outside the wrapper element.
            tmp = ET.Element("tmp")
            for n in nodes_to_insert:
                if isinstance(n, str):
                    append_text(tmp, n)
                else:
                    tmp.append(n)

            # Extract leading whitespace from tmp.text (if present)
            lead = ""
            if tmp.text:
                # leading whitespace characters
                m = re.match(r"^(\s+)(.*)$", tmp.text, flags=re.DOTALL)
                if m:
                    lead = m.group(1)
                    tmp.text = m.group(2)

            # Extract trailing whitespace from last child's tail or tmp.text
            trail = ""
            children = list(tmp)
            if children:
                last = children[-1]
                if last.tail:
                    m2 = re.match(r"^(.*?)(\s+)$", last.tail, flags=re.DOTALL)
                    if m2:
                        last.tail = m2.group(1)
                        trail = m2.group(2)
            else:
                if tmp.text:
                    m3 = re.match(r"^(.*?)(\s+)$", tmp.text, flags=re.DOTALL)
                    if m3:
                        tmp.text = m3.group(1)
                        trail = m3.group(2)

            # Move content from tmp into target
            if tmp.text:
                target.text = (target.text or "") + tmp.text
            for child in list(tmp):
                target.append(child)

            # Return leading/trailing whitespace to be emitted outside the target
            return (lead, target, trail)

        # Helper to convert populate result into a return value: either
        # return an Element (if no lead/trail) or a (lead, el, trail) tuple.
        def _maybe_wrap_with_edge_whitespace(pop_res, wrap_el):
            lead, el, trail = pop_res
            if lead or trail:
                return (lead, wrap_el, trail)
            return wrap_el

        if kind in ("abbrev", "sic", "regularised"):
            if kind == "abbrev":
                a_tag, b_tag = "abbr", "expan"
                alt = w.get("expansion", "") or ""
                first_is_witness = True
            elif kind == "sic":
                a_tag, b_tag = "sic", "corr"
                alt = w.get("correction", "") or ""
                first_is_witness = True
            else:  # regularised
                a_tag, b_tag = "orig", "reg"
                alt = w.get("original", "") or ""
                # for regularised the witness_text is the regularised form (reg),
                # and the original goes in <orig> before it. We will place
                # inner nodes into the appropriate element below.
                first_is_witness = False

            choice = ET.Element(qn("choice"))
            a = ET.SubElement(choice, qn(a_tag))
            b = ET.SubElement(choice, qn(b_tag))

            if kind == "regularised":
                # Prefer explicit attributes when provided: `original` -> <orig>
                # and `regularised` -> <reg>. Fall back to inner_nodes/witness
                # content when attrs are missing.
                orig_attr = w.get("original") or w.get("orig")
                reg_attr = w.get("regularised") or w.get("reg")

                # a (orig)
                a.text = orig_attr if orig_attr is not None else alt

                # If an explicit regularised form is provided, prefer it.
                if reg_attr is not None:
                    # Filter out plain strings that exactly match the witness
                    # text so we don't duplicate the witness in <reg>.
                    filtered_inner = []
                    for n in inner_nodes:
                        if isinstance(n, str) and n.strip() == witness_text.strip():
                            continue
                        filtered_inner.append(n)

                    # If no element children remain, emit reg_attr directly.
                    element_children = [n for n in filtered_inner if isinstance(n, ET.Element)]
                    if not element_children:
                        b.text = reg_attr
                        return choice

                    # Otherwise populate a temporary container with element
                    # children and append them to <reg>, keeping reg_attr as
                    # the base text. Do not copy plain tmp.text to avoid
                    # duplicating witness text.
                    tmp = ET.Element("tmp")
                    pop_res = populate_target_with_nodes(tmp, element_children)
                    b.text = reg_attr
                    for child in list(tmp):
                        b.append(child)
                    return _maybe_wrap_with_edge_whitespace(pop_res, choice)
                else:
                    # No explicit reg attribute: emit inner content as before
                    pop_res = populate_target_with_nodes(b, inner_nodes)
                    return _maybe_wrap_with_edge_whitespace(pop_res, choice)
            else:
                # a gets the witness content (may contain nested nodes)
                pop_res = populate_target_with_nodes(a, inner_nodes)
                # b gets the alt expansion as plain text
                b.text = alt
                return _maybe_wrap_with_edge_whitespace(pop_res, choice)

        elif kind == "num":
            el = ET.Element(qn("num"))
            if "type" in w:
                el.set("type", w["type"])
            if "value" in w:
                el.set("value", w["value"])
            pop_res = populate_target_with_nodes(el, inner_nodes)
            return _maybe_wrap_with_edge_whitespace(pop_res, el)

        elif kind == "person":
            el = ET.Element(qn("persName"))
            if "type" in w:
                el.set("type", w["type"])
            if "wikiData" in w:
                el.set("ref", f"https://www.wikidata.org/wiki/{w['wikiData']}")

            # Preserve firstname and continued on wrapper so the information
            # survives wrapping/nesting across lines. continued is expected
            # to be normalized to 'true'/'false' by parse_custom_ops.
            if "firstname" in w:
                el.set("firstname", w["firstname"])
            if "continued" in w and w["continued"] == "true":
                el.set("continued", "true")

            pop_res = populate_target_with_nodes(el, inner_nodes)
            return _maybe_wrap_with_edge_whitespace(pop_res, el)

        elif kind == "place":
            el = ET.Element(qn("placeName"))
            pop_res = populate_target_with_nodes(el, inner_nodes)
            nested_attrs = ["country", "region", "settlement", "district"]
            for attr in nested_attrs:
                if attr in w:
                    child = ET.SubElement(el, qn(attr))
                    child.text = w[attr]
            return _maybe_wrap_with_edge_whitespace(pop_res, el)

        elif kind == "ref":
            el = ET.Element(qn("ref"))
            if "type" in w:
                el.set("type", w["type"])
            if "target" in w:
                el.set("target", w["target"])
            pop_res = populate_target_with_nodes(el, inner_nodes)
            return _maybe_wrap_with_edge_whitespace(pop_res, el)

        elif kind == "unclear":
            el = ET.Element(qn("unclear"))
            if "reason" in w:
                el.set("reason", w["reason"])
            pop_res = populate_target_with_nodes(el, inner_nodes)
            return _maybe_wrap_with_edge_whitespace(pop_res, el)

        else:
            # fallback seg with echoed attributes
            seg = ET.Element(qn("seg"))
            seg.set("type", w.get("kind", "custom"))
            for key, value in w.items():
                if key not in {"kind", "offset", "length", "end"}:
                    seg.set(f"data-{key}", value)
            pop_res = populate_target_with_nodes(seg, inner_nodes)
            return _maybe_wrap_with_edge_whitespace(pop_res, seg)

    def render_span(span_start: int, span_end: int, child_idxs: List[int]) -> List[Any]:
        """Render content between span_start and span_end, inserting child wrappers."""
        output: List[Any] = []
        if not child_idxs:
            return render_plain_segment(span_start, span_end)

        # sort children by offset
        child_idxs_sorted = sorted(child_idxs, key=lambda i: wrappers[i]["offset"])
        cursor_local = span_start

        for cidx in child_idxs_sorted:
            cw = wrappers[cidx]
            cstart, cend = cw["offset"], cw["end"]
            # text before child
            if cursor_local < cstart:
                output.extend(render_plain_segment(cursor_local, cstart))
            # render child's inner content recursively
            inner = render_span(cstart, cend, children.get(cidx, []))
            built = build_wrapper_element(cidx, inner)
            # build_wrapper_element may return either an Element or a tuple
            # (lead, Element, trail) where lead/trail are whitespace strings that
            # should sit outside the element.
            if isinstance(built, tuple):
                lead, el, trail = built
                if lead:
                    output.append(lead)
                output.append(el)
                if trail:
                    output.append(trail)
            else:
                output.append(built)
            cursor_local = cend

        # tail after last child
        if cursor_local < span_end:
            output.extend(render_plain_segment(cursor_local, span_end))

        return output

    # Render top-level sequence (from 0..len(text)) including roots
    nodes: List[Any] = []
    if not roots:
        # no wrappers at all
        nodes = render_plain_segment(0, len(text))
    else:
        # render text from start to end using roots as top-level wrappers
        # For each root wrapper, render its inner span and then create the
        # wrapper element using build_wrapper_element so the wrapper itself
        # appears in the output (not just its inner content).
        roots_sorted = sorted(roots, key=lambda i: wrappers[i]["offset"])
        cursor = 0
        for ridx in roots_sorted:
            r = wrappers[ridx]
            rstart, rend = r["offset"], r["end"]
            if cursor < rstart:
                nodes.extend(render_plain_segment(cursor, rstart))
            # render the inner content of this root (children)
            inner_nodes = render_span(rstart, rend, children.get(ridx, []))
            built = build_wrapper_element(ridx, inner_nodes)
            # Accept either Element or (lead, Element, trail)
            if isinstance(built, tuple):
                lead, el, trail = built
                if lead:
                    nodes.append(lead)
                nodes.append(el)
                if trail:
                    nodes.append(trail)
            else:
                nodes.append(built)
            cursor = rend
        if cursor < len(text):
            nodes.extend(render_plain_segment(cursor, len(text)))

    # Post-process step: in some cases a <choice> (abbr/expan) gets emitted but the
    # corresponding person wrapper (persName) was not created by the containment
    # logic (ordering/edge-case). As a fallback, if there exists a person op that
    # shares exactly the same span as an abbrev/choice op, wrap any matching
    # <choice> element with a <persName> preserving attributes like type,
    # firstname and continued so entity information is not lost.
    #
    # This implementation walks the top-level `nodes` and their descendant
    # element children, wrapping any <choice> it finds. It attempts to do this
    # for each person op that matches a choice op by span. This is best-effort:
    # it only wraps choice elements that occur in the rendered node tree.
    def _wrap_choice_in_persname_in_element(
        elem: ET.Element, person_op: Dict[str, Any]
    ) -> bool:
        """Recursively search children of elem and wrap first matching <choice> child.

        Returns True if a wrapping happened.
        """
        # Do not attempt to wrap if we're already inside a persName element.
        if isinstance(elem, ET.Element) and elem.tag == qn("persName"):
            return False

        for i, child in enumerate(list(elem)):
            # Skip recursing into existing persName children to avoid double-wrapping
            if isinstance(child, ET.Element) and child.tag == qn("persName"):
                continue

            # If the child itself is a <choice>, wrap it (but ensure it's not already wrapped)
            if isinstance(child, ET.Element) and child.tag == qn("choice"):
                # Create persName element from person_op attributes
                pers = ET.Element(qn("persName"))
                if "type" in person_op:
                    pers.set("type", person_op["type"])
                if "wikiData" in person_op:
                    pers.set(
                        "ref", f"https://www.wikidata.org/wiki/{person_op['wikiData']}"
                    )
                if "firstname" in person_op:
                    pers.set("firstname", person_op["firstname"])
                if person_op.get("continued") == "true":
                    pers.set("continued", "true")

                # Preserve the tail of the choice node so whitespace is not lost
                tail = child.tail

                # Move the existing choice node under persName and keep tail intact
                elem.remove(child)
                pers.append(child)
                child.tail = tail

                # Insert persName in the same position
                elem.insert(i, pers)
                return True

            # Otherwise recurse into the child element
            if isinstance(child, ET.Element):
                wrapped = _wrap_choice_in_persname_in_element(child, person_op)
                if wrapped:
                    return True
        return False

    def _wrap_choice_in_nodes_list(
        nodes_list: List[Any], person_op: Dict[str, Any]
    ) -> bool:
        """Search the top-level nodes array and wrap the first matching choice.

        Returns True if a wrapping happened.
        """
        for idx, n in enumerate(nodes_list):
            if isinstance(n, ET.Element):
                # If this top-level node is already a persName, skip it.
                if n.tag == qn("persName"):
                    continue

                # direct choice at top-level
                if n.tag == qn("choice"):
                    pers = ET.Element(qn("persName"))
                    if "type" in person_op:
                        pers.set("type", person_op["type"])
                    if "wikiData" in person_op:
                        pers.set(
                            "ref",
                            f"https://www.wikidata.org/wiki/{person_op['wikiData']}",
                        )
                    if "firstname" in person_op:
                        pers.set("firstname", person_op["firstname"])
                    if person_op.get("continued") == "true":
                        pers.set("continued", "true")

                    # Preserve any tail text on the choice node so spacing is maintained
                    tail = n.tail
                    pers.append(n)
                    n.tail = None
                    pers.tail = tail

                    nodes_list[idx] = pers
                    return True

                # otherwise try to wrap inside this element recursively (skip persName children)
                if _wrap_choice_in_persname_in_element(n, person_op):
                    return True
        return False

    # Run the fallback wrapping for any person/choice pairs that share the same span
    # or overlap. Previously we only wrapped when spans were exactly equal; that was
    # brittle with off-by-one / whitespace differences in PAGE offsets. Here we treat
    # any non-empty intersection between the person span and the choice span as a
    # best-effort candidate for wrapping. If a wrap succeeds we stop trying further
    # choices for that person to avoid double-wrapping.
    for p in person_ops:
        for c in choice_ops:
            # compute overlap (start < end means non-empty intersection)
            start = max(p["offset"], c["offset"])
            end = min(p["end"], c["end"])
            if start < end:
                # Overlapping (or identical) spans — attempt to wrap a matching <choice>.
                wrapped = _wrap_choice_in_nodes_list(nodes, p)
                if wrapped:
                    # Stop attempting other choice ops for this person once wrapped.
                    break

    return nodes


# -------------------------
# Metadata prompts and presets
# -------------------------


def get_default_metadata(edition_type: str) -> Dict[str, Any]:
    """Get default metadata based on edition type."""
    if edition_type == "diplomatic":
        return {
            "title": "Tractatus de facinatione",
            "author": "Diego Álvarez Chanca",
            "edition_editor": "Daniel S. López",
            "resp": "Transcripción diplomática y marcación TEI",
            "resp_name": "Daniel S. López",
            "publisher": "Petrus Brun",
            "pub_date": "1499",
            "country": "España",
            "region": "",
            "settlement": "Córdoba",
            "district": "",
            "geogName": "",
            "institution": "Biblioteca Provincial de Córdoba",
            "repository": "",
            "collection": "Fondo Antiguo",
            "idno_old": "N/A",
            "idno_new": "N/A",
            "idno_siglum": "N/A",
            "orig_place": "España",
            "orig_notBefore": "1450",
            "orig_notAfter": "1515",
            "orig_label": "1499",
            "page_n": "",
            "page_side": "",
            "edition_type": "Transcripción diplomática",
            "language": "lat",
            "translator": "",
        }
    else:  # translation
        return {
            "title": "Tractatus de fascinatione",
            "author": "Diego Álvarez Chanca",
            "edition_editor": "Daniel S. López",
            "translator": "Daniel S. López",
            "resp": "Traducción al español y marcación TEI",
            "resp_name": "Daniel S. López",
            "publisher": "Patrus Brun",
            "pub_date": "1499",
            "country": "",
            "region": "",
            "settlement": "España",
            "district": "",
            "geogName": "",
            "institution": "Biblioteca Provincial de Córdoba",
            "repository": "",
            "collection": "Fondo Antiguo",
            "idno_old": "N/A",
            "idno_new": "N/A",
            "idno_siglum": "N/A",
            "orig_place": "España",
            "orig_notBefore": "1450",
            "orig_notAfter": "1515",
            "orig_label": "1499",
            "page_n": "",
            "page_side": "",
            "edition_type": "Traducción al español",
            "language": "es",
        }


def prompt_or_flag(value: Any, prompt_text: str, default: str = "") -> str:
    """Prompt user or use provided value."""
    if value is not None:
        return str(value)
    try:
        result = input(f"{prompt_text} [{default}]: ").strip()
        return result if result else default
    except (EOFError, KeyboardInterrupt):
        return default


def collect_metadata(args: argparse.Namespace, input_file: str) -> Dict[str, Any]:
    """Collect metadata interactively or from arguments."""

    # Auto-detect edition type from filename
    detected_type = detect_edition_type(input_file)

    # Confirm edition type
    if detected_type:
        print(f"\n📋 Detected edition type: {detected_type}")
        # If user passed the --yes/-y flag, assume confirmation automatically.
        if getattr(args, "yes", False):
            confirm = "y"
        else:
            confirm = input(f"Is this correct? (y/n) [y]: ").strip().lower()

    if not detected_type:
        print("\n📋 Select edition type:")
        print("  1) Diplomatic transcription")
        print("  2) Translation")
        choice = input("Enter choice (1/2): ").strip()
        detected_type = "diplomatic" if choice == "1" else "translation"

    # Get preset metadata
    meta = get_default_metadata(detected_type)

    # Show presets and ask if user wants to modify
    print(f"\n📋 Using preset metadata for {detected_type} edition:")
    print(f"  Title: {meta['title']}")
    print(f"  Language: {meta['language']}")
    print(f"  Edition type: {meta['edition_type']}")
    if meta["translator"]:
        print(f"  Translator: {meta['translator']}")
    else:
        print(f"  Editor: {meta['edition_editor']}")

    # Respect the --yes/-y flag for non-interactive runs: if supplied, assume
    # the user does NOT want to modify defaults (i.e., default 'n').
    if getattr(args, "yes", False):
        modify = "n"
    else:
        modify = (
            input("\nDo you want to modify these defaults? (y/n) [n]: ").strip().lower()
        )

    if modify in ("y", "yes"):
        # Allow modification of key fields
        meta["title"] = prompt_or_flag(args.title, "Title", meta["title"])
        meta["author"] = prompt_or_flag(
            args.author, "Author (original work)", meta["author"]
        )

        if detected_type == "translation":
            meta["translator"] = prompt_or_flag(
                getattr(args, "translator", None), "Translator", meta["translator"]
            )
        else:
            meta["edition_editor"] = prompt_or_flag(
                args.edition_editor,
                "Editor of diplomatic edition",
                meta["edition_editor"],
            )

        meta["resp"] = prompt_or_flag(args.resp, "Your responsibility", meta["resp"])
        meta["resp_name"] = prompt_or_flag(
            args.resp_name, "Your name", meta["resp_name"]
        )
        meta["publisher"] = prompt_or_flag(
            args.publisher, "Publisher", meta["publisher"]
        )
        meta["pub_date"] = prompt_or_flag(
            args.pub_date, "Publication date", meta["pub_date"]
        )

        # msIdentifier
        meta["country"] = prompt_or_flag(args.country, "Country", meta["country"])
        meta["settlement"] = prompt_or_flag(
            args.settlement, "Settlement (city)", meta["settlement"]
        )
        meta["institution"] = prompt_or_flag(
            args.institution, "Institution", meta["institution"]
        )
        meta["collection"] = prompt_or_flag(
            args.collection, "Collection", meta["collection"]
        )
        meta["idno_siglum"] = prompt_or_flag(
            args.idno_siglum, "Siglum", meta["idno_siglum"]
        )

        # Origin
        meta["orig_place"] = prompt_or_flag(
            args.orig_place, "Original place", meta["orig_place"]
        )
        meta["orig_label"] = prompt_or_flag(
            args.orig_label, "Origin date label", meta["orig_label"]
        )

    return meta


# -------------------------
# TEI header builder
# -------------------------


def build_header(meta: Dict[str, Any]) -> ET.Element:
    """Build TEI header from metadata."""
    teiHeader = ET.Element(qn("teiHeader"))
    fileDesc = ET.SubElement(teiHeader, qn("fileDesc"))

    # titleStmt
    titleStmt = ET.SubElement(fileDesc, qn("titleStmt"))
    ET.SubElement(titleStmt, qn("title")).text = meta["title"]
    ET.SubElement(titleStmt, qn("author")).text = meta["author"]

    if meta.get("translator"):
        # For translations
        ET.SubElement(titleStmt, qn("editor"), {"role": "translator"}).text = meta[
            "translator"
        ]

    if meta.get("edition_editor"):
        ET.SubElement(titleStmt, qn("editor")).text = meta["edition_editor"]

    rs = ET.SubElement(titleStmt, qn("respStmt"))
    ET.SubElement(rs, qn("resp")).text = meta["resp"]
    ET.SubElement(rs, qn("name")).text = meta["resp_name"]

    # editionStmt
    editionStmt = ET.SubElement(fileDesc, qn("editionStmt"))
    ET.SubElement(editionStmt, qn("edition")).text = meta.get(
        "edition_type", "Digital edition"
    )

    # publicationStmt
    publicationStmt = ET.SubElement(fileDesc, qn("publicationStmt"))
    if meta.get("publisher"):
        ET.SubElement(publicationStmt, qn("publisher")).text = meta["publisher"]
    if meta.get("pub_date"):
        ET.SubElement(publicationStmt, qn("date")).text = meta["pub_date"]

    # sourceDesc
    sourceDesc = ET.SubElement(fileDesc, qn("sourceDesc"))
    msDesc = ET.SubElement(sourceDesc, qn("msDesc"))
    msIdentifier = ET.SubElement(msDesc, qn("msIdentifier"))

    # msIdentifier fields
    if meta.get("country"):
        ET.SubElement(msIdentifier, qn("country")).text = meta["country"]
    if meta.get("region"):
        ET.SubElement(msIdentifier, qn("region")).text = meta["region"]
    if meta.get("settlement"):
        ET.SubElement(msIdentifier, qn("settlement")).text = meta["settlement"]
    if meta.get("district"):
        ET.SubElement(msIdentifier, qn("district")).text = meta["district"]
    if meta.get("geogName"):
        ET.SubElement(msIdentifier, qn("geogName")).text = meta["geogName"]
    if meta.get("institution"):
        ET.SubElement(msIdentifier, qn("institution")).text = meta["institution"]
    if meta.get("repository"):
        ET.SubElement(msIdentifier, qn("repository")).text = meta["repository"]
    if meta.get("collection"):
        ET.SubElement(msIdentifier, qn("collection")).text = meta["collection"]
    if meta.get("idno_old"):
        ET.SubElement(msIdentifier, qn("idno"), {"type": "oldCatalog"}).text = meta[
            "idno_old"
        ]
    if meta.get("idno_new"):
        ET.SubElement(msIdentifier, qn("idno"), {"type": "museumNew"}).text = meta[
            "idno_new"
        ]
    if meta.get("idno_siglum"):
        ET.SubElement(msIdentifier, qn("idno"), {"type": "siglum"}).text = meta[
            "idno_siglum"
        ]

    # physDesc
    if meta.get("page_n"):
        physDesc = ET.SubElement(msDesc, qn("physDesc"))
        objectDesc = ET.SubElement(physDesc, qn("objectDesc"))
        supportDesc = ET.SubElement(objectDesc, qn("supportDesc"))
        fol = ET.SubElement(supportDesc, qn("foliation"))
        fol.text = f'Numbered as "{meta["page_n"]}" in the current collection.'

    # history/origin
    history = ET.SubElement(msDesc, qn("history"))
    origin = ET.SubElement(history, qn("origin"))
    if meta.get("orig_place"):
        op = ET.SubElement(origin, qn("origPlace"))
        ET.SubElement(op, qn("placeName")).text = meta["orig_place"]

    od = ET.SubElement(origin, qn("origDate"))
    if meta.get("orig_notBefore"):
        od.set("notBefore", meta["orig_notBefore"])
    if meta.get("orig_notAfter"):
        od.set("notAfter", meta["orig_notAfter"])
    if meta.get("orig_label"):
        od.text = meta["orig_label"]

    # encodingDesc
    encodingDesc = ET.SubElement(teiHeader, qn("encodingDesc"))
    ET.SubElement(encodingDesc, qn("p")).text = (
        "Digital edition for research and display purposes. "
        "Converted from PAGE-XML with full semantic markup including "
        "abbreviations, corrections, regularisations, numbers, person names, place names, "
        "references, and text styling."
    )

    # profileDesc
    profileDesc = ET.SubElement(teiHeader, qn("profileDesc"))
    langUsage = ET.SubElement(profileDesc, qn("langUsage"))
    lang_code = meta.get("language", "grc")
    lang_name = "Ancient Greek" if lang_code == "grc" else "Spanish"
    ET.SubElement(langUsage, qn("language"), {"ident": lang_code}).text = lang_name

    # revisionDesc
    revisionDesc = ET.SubElement(teiHeader, qn("revisionDesc"))
    ET.SubElement(
        revisionDesc, qn("change")
    ).text = "Automated conversion from PAGE-XML with preservation of all annotations."

    return teiHeader


# -------------------------
# PAGE -> TEI converter
# -------------------------


def convert_page_to_tei(
    page_root: ET.Element, meta: Dict[str, Any], debug_inline: bool = False
) -> ET.Element:
    """Convert PAGE XML to TEI."""
    tei = ET.Element(qn("TEI"))
    tei.append(build_header(meta))

    facsimile = ET.SubElement(tei, qn("facsimile"))
    text_el = ET.SubElement(tei, qn("text"))
    body = ET.SubElement(text_el, qn("body"))
    div = ET.SubElement(body, qn("div"), {"type": "transcription"})

    # Set language
    lang_code = meta.get("language", "grc")
    div.set(ET.QName(XML_NS, "lang"), lang_code)

    # Iterate pages
    for page_idx, page in enumerate(page_root.findall("pg:Page", PAGE_NS), start=1):
        image_fn = page.get("imageFilename")
        width = page.get("imageWidth")
        height = page.get("imageHeight")

        # Force target image size (px) for output and scaling
        TARGET_IMG_W = 960
        TARGET_IMG_H = 1358

        # Parse PAGE-provided image size if available (may include 'px')
        page_img_w = None
        page_img_h = None
        try:
            if width:
                page_img_w = int(str(width).rstrip("px"))
        except Exception:
            page_img_w = None
        try:
            if height:
                page_img_h = int(str(height).rstrip("px"))
        except Exception:
            page_img_h = None

        surface = ET.SubElement(facsimile, qn("surface"), {"n": str(page_idx)})
        surface.set(ET.QName(XML_NS, "id"), f"p{page_idx}")

        if meta.get("page_side"):
            surface.set("type", meta["page_side"])
        if meta.get("page_n"):
            surface.set("n", meta["page_n"])

        graphic = ET.SubElement(surface, qn("graphic"))
        if image_fn:
            # Add "images/" prefix if not already present
            if not image_fn.startswith("images/"):
                image_fn = f"images/{image_fn}"
            graphic.set("url", image_fn)

        # Always emit the forced target image size for TEI output
        graphic.set("width", f"{TARGET_IMG_W}px")
        graphic.set("height", f"{TARGET_IMG_H}px")

        # Page break
        ET.SubElement(div, qn("pb"), {"n": str(page_idx), "facs": f"#p{page_idx}"})

        # Map regions to zones
        for region in page.findall(".//*", PAGE_NS):
            local = region.tag.split("}")[-1]
            if local.endswith("Region"):
                rid = region.get("id") or f"reg_{local}_{page_idx}"
                coords = region.find("pg:Coords", PAGE_NS)
                if coords is not None:
                    z = ET.SubElement(surface, qn("zone"), {"type": local})
                    z.set(ET.QName(XML_NS, "id"), f"z_{rid}")
                    pts = coords.get("points")
                    if pts:
                        # Scale PAGE coords to forced TARGET_IMG size when possible
                        if page_img_w and page_img_h:
                            sx = TARGET_IMG_W / page_img_w if page_img_w else 1.0
                            sy = TARGET_IMG_H / page_img_h if page_img_h else 1.0
                            z.set("points", scale_points(pts, sx, sy))
                        else:
                            z.set("points", pts)

        # Collect TextRegions with their lines
        # Helper function to get Y coordinate from baseline/points
        def get_y(baseline, points):
            if baseline:
                try:
                    coords = baseline.split()
                    y_vals = [int(c.split(",")[1]) for c in coords]
                    return sum(y_vals) // len(y_vals)
                except Exception:
                    pass
            if points:
                try:
                    coords = points.split()
                    y_vals = [int(c.split(",")[1]) for c in coords]
                    return min(y_vals)
                except Exception:
                    pass
            return 999999

        regions = []
        for tregion in page.findall(".//pg:TextRegion", PAGE_NS):
            # Get region's reading order index
            region_cust = tregion.get("custom") or ""
            region_idx = 999999
            if "index:" in region_cust:
                try:
                    region_idx = int(
                        region_cust.split("index:")[1].split(";")[0].strip(" }")
                    )
                except Exception:
                    pass

            region_lines = []
            for tl in tregion.findall("pg:TextLine", PAGE_NS):
                cust = tl.get("custom") or ""
                line_idx = 999999
                if "index:" in cust:
                    try:
                        line_idx = int(
                            cust.split("index:")[1].split(";")[0].strip(" }")
                        )
                    except Exception:
                        pass

                coords_el = tl.find("pg:Coords", PAGE_NS)
                points = coords_el.get("points") if coords_el is not None else None

                baseline_el = tl.find("pg:Baseline", PAGE_NS)
                baseline = (
                    baseline_el.get("points") if baseline_el is not None else None
                )

                text_val = (
                    tl.findtext(
                        "pg:TextEquiv/pg:Unicode", default="", namespaces=PAGE_NS
                    )
                    or ""
                )

                tl_id = tl.get("id") or f"tl_{len(region_lines) + 1}"
                # Store as (line_idx, y_coord, tl_id, points, baseline, text_val, cust)
                y_coord = get_y(baseline, points)
                region_lines.append(
                    (line_idx, y_coord, tl_id, points, baseline, text_val, cust)
                )

            # Sort lines within this region by Y coordinate (vertical position)
            # This handles cases where readingOrder indices are inconsistent
            region_lines.sort(key=lambda x: x[1])  # Sort by y_coord

            # Store region with its sorted lines
            regions.append((region_idx, region_lines))

        # Sort regions by their reading order index
        regions.sort(key=lambda x: x[0])

        # Flatten regions into a single list of lines
        lines = []
        for region_idx, region_lines in regions:
            for line_data in region_lines:
                # Convert to format: (tl_id, points, baseline, text_val, cust)
                _, _, tl_id, points, baseline, text_val, cust = line_data
                lines.append((tl_id, points, baseline, text_val, cust))

        # Create line zones and text
        for line_num, (tl_id, points, baseline, text_val, cust) in enumerate(
            lines, start=1
        ):
            zid = f"z_{tl_id}"
            z = ET.SubElement(surface, qn("zone"), {"type": "line"})
            z.set(ET.QName(XML_NS, "id"), zid)
            if points:
                if page_img_w and page_img_h:
                    sx = TARGET_IMG_W / page_img_w if page_img_w else 1.0
                    sy = TARGET_IMG_H / page_img_h if page_img_h else 1.0
                    z.set("points", scale_points(points, sx, sy))
                else:
                    z.set("points", points)
            # Store baseline in a note element (baseline attribute not allowed on zone)
            if baseline:
                baseline_note = ET.SubElement(z, qn("note"), {"type": "baseline"})
                if page_img_w and page_img_h:
                    sx = TARGET_IMG_W / page_img_w if page_img_w else 1.0
                    sy = TARGET_IMG_H / page_img_h if page_img_h else 1.0
                    baseline_note.text = scale_points(baseline, sx, sy)
                else:
                    baseline_note.text = baseline

            # Line break with number
            ET.SubElement(div, qn("lb"), {"facs": f"#{zid}", "n": str(line_num)})
            ab = ET.SubElement(div, qn("ab"))

            # Parse custom annotations and build inline nodes
            ops = parse_custom_ops(cust)
            inline_nodes = build_inline_nodes_for_line(text_val, ops)

            # Debugging: optionally print parsed ops and rendered inline nodes for
            # lines that include person or numeric annotations.
            # (Previously this only printed when both person AND abbrev were present;
            # broaden the condition so num+person cases and other person/num occurrences
            # are visible during debugging.)
            if debug_inline:
                has_person = any(o.get("kind") == "person" for o in ops)
                has_num = any(o.get("kind") == "num" for o in ops)
                # Show debug output when a line contains person OR num annotations.
                if has_person or has_num:
                    try:
                        print(
                            f"DEBUG inline for line {tl_id}: ops={ops}", file=sys.stderr
                        )
                    except Exception:
                        # fallback to safer print if formatting fails
                        print(
                            "DEBUG inline for line:",
                            tl_id,
                            "ops:",
                            ops,
                            file=sys.stderr,
                        )
                    serial = []
                    for n in inline_nodes:
                        if isinstance(n, str):
                            serial.append(repr(n))
                        else:
                            try:
                                serial.append(
                                    ET.tostring(n, encoding="unicode", method="xml")
                                )
                            except Exception:
                                serial.append(str(n))
                    print("DEBUG inline nodes: " + ", ".join(serial), file=sys.stderr)

            for node in inline_nodes:
                if isinstance(node, str):
                    append_text(ab, node)
                else:
                    ab.append(node)

            # Fallback if ab is empty
            if ab.text is None and len(ab) == 0:
                ab.text = text_val

    return tei


# -------------------------
# CLI
# -------------------------


def main():
    ap = argparse.ArgumentParser(
        description="Convert Transkribus PAGE-XML to TEI P5 with comprehensive semantic markup.",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Examples:
  # Interactive mode (recommended)
  python page2tei.py --input page_p1_dip.xml --output p1_dip.xml

  # Non-interactive with presets
  python page2tei.py -i page_p1_trad.xml -o p1_trad.xml --title "PGM XIII Translation"

  # Use stdin/stdout
  cat input.xml | python page2tei.py -i - -o - > output.xml
        """,
    )

    ap.add_argument(
        "--input", "-i", default="-", help='Input PAGE-XML file or "-" for stdin'
    )
    ap.add_argument(
        "--output", "-o", default="-", help='Output TEI-XML file or "-" for stdout'
    )
    ap.add_argument(
        "--yes",
        "-y",
        action="store_true",
        help="Assume 'yes' for interactive prompts (allow non-interactive runs)",
    )
    ap.add_argument(
        "--debug-inline",
        action="store_true",
        help="Print parsed custom ops and generated inline nodes for lines containing person or num annotations (debug output to stderr)",
    )

    # Metadata arguments (optional, will prompt if not provided)
    ap.add_argument("--title", help="Title of the work")
    ap.add_argument("--author", help="Original author")
    ap.add_argument(
        "--edition-editor", dest="edition_editor", help="Editor of diplomatic edition"
    )
    ap.add_argument("--translator", help="Translator (for translation editions)")
    ap.add_argument("--resp", help="Your responsibility")
    ap.add_argument("--resp-name", dest="resp_name", help="Your name")
    ap.add_argument("--publisher", help="Publisher")
    ap.add_argument("--pub-date", dest="pub_date", help="Publication date")

    # msIdentifier
    ap.add_argument("--country", help="Holding country")
    ap.add_argument("--region", help="Region")
    ap.add_argument("--settlement", help="Settlement/city")
    ap.add_argument("--district", help="District")
    ap.add_argument("--geogName", help="Geographic name")
    ap.add_argument("--institution", help="Holding institution")
    ap.add_argument("--repository", help="Repository")
    ap.add_argument("--collection", help="Collection")
    ap.add_argument("--idno-old", dest="idno_old", help="Old catalog ID")
    ap.add_argument("--idno-new", dest="idno_new", help="New museum ID")
    ap.add_argument("--idno-siglum", dest="idno_siglum", help="Siglum")

    # Origin
    ap.add_argument("--orig-place", dest="orig_place", help="Original place")
    ap.add_argument(
        "--orig-notBefore", dest="orig_notBefore", help="Origin notBefore date"
    )
    ap.add_argument(
        "--orig-notAfter", dest="orig_notAfter", help="Origin notAfter date"
    )
    ap.add_argument("--orig-label", dest="orig_label", help="Origin date label")

    # Page info
    ap.add_argument("--page-n", dest="page_n", help="Page number/label")
    ap.add_argument(
        "--page-side", dest="page_side", choices=["recto", "verso"], help="Page side"
    )

    args = ap.parse_args()

    # Read PAGE XML
    if args.input == "-":
        data = sys.stdin.read()
        input_file = "stdin"
    else:
        input_file = args.input
        with open(args.input, "r", encoding="utf-8", errors="ignore") as f:
            data = f.read()

    page_tree = ET.ElementTree(ET.fromstring(data))
    page_root = page_tree.getroot()

    # Collect metadata
    meta = collect_metadata(args, input_file)

    # Convert
    tei = convert_page_to_tei(page_root, meta, debug_inline=args.debug_inline)
    out = prettify(tei)

    # Write TEI
    if args.output == "-":
        sys.stdout.write(out)
    else:
        with open(args.output, "w", encoding="utf-8") as f:
            f.write(out)
        print(f"\n✅ Successfully converted to {args.output}")


if __name__ == "__main__":
    main()
