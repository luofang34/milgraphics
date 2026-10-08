#!/usr/bin/env bash
# Usage: [ORACLE_JOBS=N] [ORACLE_COMPACT=1] tools/oracle/oracle.sh CASES.tsv > OUT.jsonl
# Compiles the pinned mil-sym-java with the harness (once) and renders every
# case in its own JVM, because mil-sym keeps renderer settings in static state.
# `oracle.sh --cases` prints the generated case list (tools/oracle/cases/all.tsv) instead.
# Cases run ORACLE_JOBS at a time (default: CPU count); output keeps input order.
# ORACLE_COMPACT=1 drops the GeoJSON field and rounds pixel coordinates to 0.01.
set -euo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
"$here/fetch-upstream.sh" >&2
up="$here/upstream"
src="$up/mil-sym-java/src/main"
classes="$up/classes"
jars="$up/lib/geodesy-1.1.3.jar:$up/lib/jsvg-2.0.0.jar"
font="$up/lib/$(python3 -I "$here/pin.py" value oracle_runtime font file)"
stamp="$classes/.built-from"
want="$(git -C "$up/mil-sym-java" rev-parse HEAD) $(cat "$here"/java/*.java | (sha256sum 2>/dev/null || shasum -a 256) | cut -c1-64)"
if [ "$(cat "$stamp" 2>/dev/null)" != "$want" ]; then
  rm -rf "$classes" && mkdir -p "$classes"
  find "$src/java" -name '*.java' > "$up/sources.txt"
  ls "$here"/java/*.java >> "$up/sources.txt"
  javac -nowarn -encoding UTF-8 --release 21 -cp "$jars" -d "$classes" @"$up/sources.txt" 2>&1 \
    | grep -vE '^Note: |uses unchecked|deprecat' >&2 || true
  [ -f "$classes/Oracle.class" ] && [ -f "$classes/Cases.class" ] || { echo "oracle harness failed to compile" >&2; exit 1; }
  echo "$want" > "$stamp"
fi
cp="$classes:$src/resources:$jars"
if [ "$1" = "--cases" ]; then
  exec java -Djava.awt.headless=true -cp "$cp" Cases
fi
cases="$(cd "$(dirname "$1")" && pwd)/$(basename "$1")"
jobs="${ORACLE_JOBS:-$(getconf _NPROCESSORS_ONLN 2>/dev/null || nproc 2>/dev/null || echo 2)}"
work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT
# One file per case; the zero-padded suffix keeps the later concatenation in input order.
grep -v '^#' "$cases" | grep -v '^[[:space:]]*$' | awk -v d="$work" '{ f = sprintf("%s/case.%06d", d, NR); print > f; close(f) }'
export classes src jars font work cp
if [ "${ORACLE_COMPACT:-0}" = 1 ]; then export ORACLE_COMPACT=true; else export ORACLE_COMPACT=false; fi
ls "$work" | xargs -P "$jobs" -I{} sh -c \
  'java -Djava.awt.headless=true -Dfile.encoding=UTF-8 -XX:TieredStopAtLevel=1 -Doracle.compact="$ORACLE_COMPACT" \
     -cp "$cp" Oracle "$font" "$(cat "$work/{}")" > "$work/{}.out"'
for f in "$work"/case.??????; do cat "$f.out"; done
