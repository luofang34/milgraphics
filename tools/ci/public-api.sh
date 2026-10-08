#!/usr/bin/env bash
# Fails when the public API differs from tools/ci/public-api.txt, so every
# change to it is deliberate and visible in review. `--bless` records the
# current API. Needs a nightly toolchain and cargo-public-api.
set -euo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
snapshot="$here/public-api.txt"
current="$(cargo +nightly public-api --package milgraphics -sss --color never)"
if [ "${1:-}" = "--bless" ]; then
  printf '%s\n' "$current" > "$snapshot"
  exit 0
fi
if ! diff -u "$snapshot" <(printf '%s\n' "$current"); then
  echo "public API changed; review the diff and run tools/ci/public-api.sh --bless" >&2
  exit 1
fi
