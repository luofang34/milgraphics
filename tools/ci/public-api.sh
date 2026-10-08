#!/usr/bin/env bash
# Fails when the public API differs from tools/ci/public-api.txt, so every
# change to it is deliberate and visible in review. `--bless` records the
# current API. Needs the pinned nightly (the one CI uses) and cargo-public-api.
set -euo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
snapshot="$here/public-api.txt"
current="$(cargo +"${PUBLIC_API_TOOLCHAIN:-nightly-2026-08-06}" public-api --package milgraphics -sss --color never)"
# Public enums are non-exhaustive, so a new variant (a shape, a handle, an
# error) is not a breaking change for code matching on them.
exhaustive="$(printf '%s\n' "$current" | grep -E '^pub enum ' || true)"
if [ -n "$exhaustive" ]; then
  printf 'public enums must be #[non_exhaustive]:\n%s\n' "$exhaustive" >&2
  exit 1
fi
if [ "${1:-}" = "--bless" ]; then
  printf '%s\n' "$current" > "$snapshot"
  exit 0
fi
if ! diff -u "$snapshot" <(printf '%s\n' "$current"); then
  echo "public API changed; review the diff and run tools/ci/public-api.sh --bless" >&2
  exit 1
fi
