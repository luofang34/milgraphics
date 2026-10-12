#!/usr/bin/env bash
# Usage: tools/ci/shared-ci.sh [--update <milsymbol commit>]
# The release workflow and these repository checks are milsymbol's files,
# unchanged. They are compared with milsymbol at a pinned commit, so a change
# there reaches this repository only when the pin moves; --update copies the
# files from another commit and moves the pin to it.
set -euo pipefail
cd "$(dirname "$0")/../.."
pin=eb7816905d6ceb71813fbd435888789df803882d
files=(
  .github/workflows/release.yml
  tools/ci/changelog.sh
  tools/ci/crate.sh
  tools/ci/file-size.sh
  tools/ci/lint-config.sh
  tools/ci/release-auth.sh
  tools/ci/required-checks.sh
  tools/ci/workflow-concurrency.sh
)
fetch() { curl -fsSL "https://raw.githubusercontent.com/luofang34/milsymbol/$1/$2"; }
if [ "${1:-}" = --update ]; then
  commit=${2:?usage: tools/ci/shared-ci.sh --update <milsymbol commit>}
  [[ "$commit" =~ ^[0-9a-f]{40}$ ]] || { echo "pass a full commit hash" >&2; exit 2; }
  for f in "${files[@]}"; do fetch "$commit" "$f" > "$f"; done
  sed -i.orig "s/^pin=.*/pin=$commit/" tools/ci/shared-ci.sh && rm tools/ci/shared-ci.sh.orig
  echo "copied ${#files[@]} files from milsymbol $commit"
  exit 0
fi
differ=0
for f in "${files[@]}"; do
  if ! diff -u <(fetch "$pin" "$f") "$f"; then
    echo "$f differs from milsymbol $pin" >&2
    differ=1
  fi
done
exit "$differ"
