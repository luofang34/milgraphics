#!/usr/bin/env python3
"""Locate each multipoint graphic's row in the standards' appendix tables.

Usage: python3 -I tools/standards/references.py D_CH1.pdf E_CH1.pdf [CATALOG.rs] [OUT.json]

Emits, per (standard, symbol set, entity), the table heading in force, the PDF
file page (1-based) of the entity's code row, and the draw rule printed there.
Only those three fields are written; no standard text is retained. The tool
reads the catalog solely to decide which entities to look for.
"""

import bisect
import json
import re
import subprocess
import sys
from pathlib import Path

# (first, last) PDF file pages of the appendices that hold the control measure
# and METOC symbol tables. The same six-digit codes recur in the symbol-set
# appendix and the icon appendices, so rows are only accepted inside these
# ranges. Bounds come from the "APPENDIX x" page headers of each document.
STANDARDS = {
    "mil-std-2525d-ch1": {
        "version_bit": 11,
        "ranges": {25: (408, 586), 45: (587, 690), 46: (587, 690)},
        "table_prefix": {25: "H", 45: "I", 46: "I"},
    },
    "mil-std-2525e-ch1": {
        "version_bit": 15,
        "ranges": {25: (444, 637), 45: (638, 735), 46: (638, 735)},
        "table_prefix": {25: "L", 45: "M", 46: "M"},
    },
}
SINGLE_POINT_RULES = {"Point1", "Point2", "Point3", "Point7"}

CATALOG_RE = re.compile(
    r"symbol_set: (\d+), entity: (\d+), name: \"((?:[^\"\\]|\\.)*)\".*?"
    r"from_bits\(0x([0-9a-f]+)\).*?"
    r"CatalogDrawRule::(?:Standard|Metoc)\((?:Mo)?DrawRule::(\w+)\)"
)
CODE_RE = re.compile(r"(?:Code|Value):\s*(\d{6})\b")
SET_RE = re.compile(r"(?:Symbol Set(?: Code)?:|Code:|Value:)\s*(\d{2})\b")
TABLE_RE = re.compile(r"^\s*TABLE ([A-Z]-[IVXLC]+)\.", re.M)
RULE_RE = re.compile(
    r"\b((?:Line|Area|Axis|Point|Corridor|Arc|Circular|Rectangular|Ellipse"
    r"|Polyline|Route|Cake|Torus|Track)\d+)\s*[-\u2013\u2014]\s*(?:Dynamic|Static)"
)


def load_catalog(path):
    """Return {(version_bit, set, entity): (name, catalog_rule)} for wanted rows."""
    targets = {}
    for line in Path(path).read_text(encoding="utf8").splitlines():
        m = CATALOG_RE.search(line)
        if not m:
            continue
        sset, entity = int(m.group(1)), int(m.group(2))
        name, bits, rule = m.group(3), int(m.group(4), 16), m.group(5)
        if rule == "DoNotDraw":
            continue
        if sset == 25 and rule in SINGLE_POINT_RULES:
            continue
        for std in STANDARDS.values():
            bit = std["version_bit"]
            if bits >> bit & 1:
                targets[(bit, sset, entity)] = (name, rule)
    return targets


def read_pages(pdf):
    out = subprocess.run(
        ["pdftotext", "-layout", str(pdf), "-"], check=True, capture_output=True
    ).stdout.decode("utf8", "replace")
    return out.split("\f")


def scan(pages, std):
    """Return {(set, entity): [{table, pdf_page, draw_rule, _rules}]} for one document."""
    found = {}
    for sset, (first, last) in std["ranges"].items():
        text, starts = "", []
        for p in range(first, last + 1):
            starts.append(len(text))
            text += pages[p - 1] + "\n"
        prefix = std["table_prefix"][sset] + "-"
        heads = [(m.start(), m.group(1)) for m in TABLE_RE.finditer(text)
                 if m.group(1).startswith(prefix)]
        head_pos = [h[0] for h in heads]
        sets = [(m.start(), int(m.group(1))) for m in SET_RE.finditer(text)]
        set_pos = [s[0] for s in sets]
        codes = list(CODE_RE.finditer(text))
        for i, m in enumerate(codes):
            pos = m.start()
            j = bisect.bisect_right(set_pos, pos) - 1
            if j < 0 or sets[j][1] != sset:
                continue
            end = codes[i + 1].start() if i + 1 < len(codes) else len(text)
            rules = RULE_RE.findall(text[pos:end])
            k = bisect.bisect_right(head_pos, pos) - 1
            page = first + bisect.bisect_right(starts, pos) - 1
            found.setdefault((sset, int(m.group(1))), []).append({
                "table": f"TABLE {heads[k][1]}" if k >= 0 else None,
                "pdf_page": page,
                "draw_rule": rules[0] if rules else None,
                "_rules": rules,
            })
    return found


def main(argv):
    if len(argv) < 3:
        sys.stderr.write(__doc__)
        return 2
    here = Path(__file__).resolve().parent
    catalog = argv[3] if len(argv) > 3 else here.parent.parent / "src/generated/catalog.rs"
    out_path = Path(argv[4]) if len(argv) > 4 else here.parent / "oracle/references.json"
    targets = load_catalog(catalog)
    result, missing, ambiguous, divergent = {}, [], [], []
    for doc, pdf in zip(STANDARDS, argv[1:3]):
        std = STANDARDS[doc]
        found = scan(read_pages(pdf), std)
        refs = result.setdefault(doc, {})
        for (bit, sset, entity), (name, rule) in sorted(targets.items()):
            if bit != std["version_bit"]:
                continue
            hits = found.get((sset, entity))
            if not hits:
                missing.append((doc, sset, entity, name, rule))
                continue
            if len(hits) > 1:
                ambiguous.append((doc, sset, entity, f"{len(hits)} rows"))
            h = hits[0]
            if h["draw_rule"] is None or len(set(h["_rules"])) > 1:
                ambiguous.append((doc, sset, entity, f"rules={h['_rules']}"))
            refs[f"{sset}:{entity}"] = {k: h[k] for k in ("table", "pdf_page", "draw_rule")}
            if h["draw_rule"] != rule:
                divergent.append((doc, sset, entity, name, rule, h["draw_rule"]))
    write_json(out_path, result)
    emit(targets, result, missing, ambiguous, divergent)
    return 0


def write_json(path, result):
    """One entry per line so reference changes diff cleanly."""
    docs = []
    for doc in sorted(result):
        body = ",\n".join(
            f"  {json.dumps(k)}: {json.dumps(v, separators=(', ', ': '))}"
            for k, v in sorted(result[doc].items())
        )
        docs.append(f"{json.dumps(doc)}: {{\n{body}\n}}")
    path.write_text("{\n" + ",\n".join(docs) + "\n}\n", encoding="utf8")


def emit(targets, result, missing, ambiguous, divergent):
    for doc, std in STANDARDS.items():
        for sset in (25, 45, 46):
            total = sum(1 for (b, s, _) in targets if b == std["version_bit"] and s == sset)
            got = sum(1 for k in result[doc] if k.startswith(f"{sset}:"))
            print(f"{doc} set {sset}: found {got} / {total}")
    print(f"not found: {len(missing)}")
    for row in missing:
        print("  missing", *row)
    print(f"ambiguous: {len(ambiguous)}")
    for row in ambiguous:
        print("  ambiguous", *row)
    print(f"draw rule differs from catalog: {len(divergent)}")
    for row in divergent:
        print("  diverges", *row)


if __name__ == "__main__":
    sys.exit(main(sys.argv))
