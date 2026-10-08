"""Compares two oracle JSON Lines files record by record.

Usage: diff-fixtures.py COMMITTED.jsonl REGENERATED.jsonl [--tolerance T]

With --tolerance, numbers within T of each other are equal, which compares a
compact record with a full one.

Exits non-zero on any difference in the oracle's output and names each
differing case and field. `font_probe` describes the platform's font
metrics rather than the oracle's output: its differences are reported as a
notice, so metric drift between platforms is visible without failing when
every shape and label is unchanged.
"""
import json
import sys


def load(path):
    with open(path, encoding="utf-8") as f:
        return [json.loads(line) for line in f if line.strip()]


TOL = 0.0


def same(a, b):
    if isinstance(a, (int, float)) and isinstance(b, (int, float)) and not isinstance(a, bool):
        return abs(a - b) <= TOL
    if isinstance(a, list) and isinstance(b, list):
        return len(a) == len(b) and all(same(x, y) for x, y in zip(a, b))
    if isinstance(a, dict) and isinstance(b, dict):
        return a.keys() == b.keys() and all(same(a[k], b[k]) for k in a)
    return a == b


def main():
    global TOL
    if "--tolerance" in sys.argv:
        i = sys.argv.index("--tolerance")
        TOL = float(sys.argv[i + 1])
        del sys.argv[i:i + 2]
    old, new = load(sys.argv[1]), load(sys.argv[2])
    problems = []
    notices = []
    if [r["case"] for r in old] != [r["case"] for r in new]:
        a_ids, b_ids = [r["case"] for r in old], [r["case"] for r in new]
        only_old, only_new = sorted(set(a_ids) - set(b_ids)), sorted(set(b_ids) - set(a_ids))
        problems.append(f"case lists differ: only committed {only_old[:10]}, only regenerated {only_new[:10]}")
    for a, b in zip(old, new):
        for key in sorted(set(a) | set(b)):
            if TOL and key == "geojson" and key not in b:
                continue
            if same(a.get(key), b.get(key)):
                continue
            if key == "font_probe":
                notices.append(f"{a['case']}: font metrics differ: {a.get(key)} vs {b.get(key)}")
            else:
                problems.append(f"{a['case']}: field {key!r} differs")
    if notices:
        print(f"notice: {notices[0]} ({len(notices)} records)")
    if problems:
        print("\n".join(problems), file=sys.stderr)
        print(f"{sys.argv[1]} differs from the pinned oracle; regenerate with tools/oracle/oracle.sh",
              file=sys.stderr)
        sys.exit(1)
    print(f"{sys.argv[1]}: {len(old)} records match")


main()
