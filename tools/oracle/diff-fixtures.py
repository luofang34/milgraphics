"""Compares two oracle JSON Lines files record by record.

Usage: diff-fixtures.py COMMITTED.jsonl REGENERATED.jsonl

Exits non-zero on any difference and names each differing case and field,
so that, for example, font-metric drift between platforms (`font_probe`)
is distinguishable from a geometry change.
"""
import json
import sys


def load(path):
    with open(path, encoding="utf-8") as f:
        return [json.loads(line) for line in f if line.strip()]


def main():
    old, new = load(sys.argv[1]), load(sys.argv[2])
    problems = []
    if [r["case"] for r in old] != [r["case"] for r in new]:
        problems.append(f"case lists differ: {[r['case'] for r in old]} vs {[r['case'] for r in new]}")
    for a, b in zip(old, new):
        for key in sorted(set(a) | set(b)):
            if a.get(key) != b.get(key):
                detail = ""
                if key == "font_probe":
                    detail = f": {a.get(key)} vs {b.get(key)}"
                problems.append(f"{a['case']}: field {key!r} differs{detail}")
    if problems:
        print("\n".join(problems), file=sys.stderr)
        print(f"{sys.argv[1]} differs from the pinned oracle; regenerate with tools/oracle/oracle.sh",
              file=sys.stderr)
        sys.exit(1)
    print(f"{sys.argv[1]}: {len(old)} records match")


main()
