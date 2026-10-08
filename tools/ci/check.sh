#!/usr/bin/env bash
# The CI `native` job, runnable locally: a passing local run means the job passes.
set -euo pipefail
export RUSTFLAGS="${RUSTFLAGS:--D warnings}"
step() { echo "::group::$*"; "$@"; echo "::endgroup::"; }
step cargo fmt --all --check
step cargo clippy --locked --workspace --all-targets -- -D warnings
step cargo clippy --locked --workspace --all-targets --target wasm32-unknown-unknown -- -D warnings
step cargo test --locked --workspace --all-targets
step cargo test --locked --workspace --doc
RUSTDOCFLAGS="-D warnings -D missing_docs -D rustdoc::broken_intra_doc_links" \
  step cargo doc --locked --no-deps --workspace
step cargo build --locked --release
step tools/ci/file-size.sh
step tools/ci/lint-config.sh
step tools/ci/dep-guard.sh
