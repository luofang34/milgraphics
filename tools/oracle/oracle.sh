#!/usr/bin/env bash
# Usage: tools/oracle/oracle.sh CASES.tsv > OUT.jsonl
# Compiles the pinned mil-sym-java with the harness (once) and renders every
# case in its own JVM, because mil-sym keeps renderer settings in static state.
set -euo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
cases="$(cd "$(dirname "$1")" && pwd)/$(basename "$1")"
"$here/fetch-upstream.sh" >&2
up="$here/upstream"
src="$up/mil-sym-java/src/main"
classes="$up/classes"
jars="$up/lib/geodesy-1.1.3.jar:$up/lib/jsvg-2.0.0.jar"
font="$up/lib/$(python3 -I -c "import json; print(json.load(open('$here/pin.json'))['oracle_runtime']['font']['file'])")"
stamp="$classes/.built-from"
want="$(git -C "$up/mil-sym-java" rev-parse HEAD) $(cat "$here/java/Oracle.java" | (sha256sum 2>/dev/null || shasum -a 256) | cut -c1-64)"
if [ "$(cat "$stamp" 2>/dev/null)" != "$want" ]; then
  rm -rf "$classes" && mkdir -p "$classes"
  find "$src/java" -name '*.java' > "$up/sources.txt"
  echo "$here/java/Oracle.java" >> "$up/sources.txt"
  javac -nowarn -encoding UTF-8 --release 21 -cp "$jars" -d "$classes" @"$up/sources.txt" 2>&1 \
    | grep -vE '^Note: |uses unchecked|deprecat' >&2 || true
  [ -f "$classes/Oracle.class" ] || { echo "oracle harness failed to compile" >&2; exit 1; }
  echo "$want" > "$stamp"
fi
grep -v '^#' "$cases" | grep -v '^[[:space:]]*$' | while IFS= read -r line; do
  java -Djava.awt.headless=true -Dfile.encoding=UTF-8 -cp "$classes:$src/resources:$jars" Oracle "$font" "$line"
done
