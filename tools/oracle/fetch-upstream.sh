#!/usr/bin/env bash
# Fetches the pinned mil-sym-java source, oracle jars and label font into
# tools/oracle/upstream/ (git-ignored), and fails unless every commit and
# sha256 matches tools/oracle/pin.json.
set -euo pipefail
cd "$(dirname "$0")"
up=upstream
mkdir -p "$up/lib"
pin() { python3 -I -c "import json,sys; d=json.load(open('pin.json')); print(eval(sys.argv[1], {'d': d}))" "$1"; }
sha256() { if command -v sha256sum >/dev/null; then sha256sum "$1"; else shasum -a 256 "$1"; fi | cut -c1-64; }
check() { # file expected-sha256
  local got; got=$(sha256 "$1")
  [ "$got" = "$2" ] || { echo "$1: sha256 $got, pinned $2" >&2; exit 1; }
}

url=$(pin "d['references']['mil-sym-java']['url']")
commit=$(pin "d['references']['mil-sym-java']['commit']")
src="$up/mil-sym-java"
if [ ! -d "$src/.git" ]; then
  git init -q "$src"
  git -C "$src" fetch -q --depth 1 "$url" "$commit"
  git -C "$src" -c advice.detachedHead=false checkout -q FETCH_HEAD
fi
head=$(git -C "$src" rev-parse HEAD)
[ "$head" = "$commit" ] || { echo "$src is at $head, pinned $commit" >&2; exit 1; }
pin "'\n'.join(f'{f} {s}' for g in d['references']['mil-sym-java']['files'].values() for f, s in g.items())" |
  while read -r f s; do check "$src/$f" "$s"; done

pin "'\n'.join(f\"{n} {j['url']} {j['sha256']}\" for n, j in d['oracle_runtime']['jars'].items())" |
  while read -r name u s; do
    [ -f "$up/lib/$name" ] || curl -sfL -o "$up/lib/$name" "$u"
    check "$up/lib/$name" "$s"
  done
font=$(pin "d['oracle_runtime']['font']['file']")
[ -f "$up/lib/$font" ] || curl -sfL -o "$up/lib/$font" "$(pin "d['oracle_runtime']['font']['url']")"
check "$up/lib/$font" "$(pin "d['oracle_runtime']['font']['sha256']")"
echo "pinned upstream verified: mil-sym-java $commit, $(ls "$up/lib" | wc -l | tr -d ' ') runtime files"
