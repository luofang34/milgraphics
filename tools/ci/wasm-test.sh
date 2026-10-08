#!/usr/bin/env bash
# Runs the behaviour tests in a headless browser through wasm-bindgen-test.
# Needs wasm-bindgen-cli at the wasm-bindgen version in Cargo.lock and a
# WebDriver: CHROMEDRIVER, GECKODRIVER or SAFARIDRIVER (found on PATH if unset).
set -euo pipefail
export CARGO_TARGET_WASM32_UNKNOWN_UNKNOWN_RUNNER=wasm-bindgen-test-runner
want=$(awk '/^name = "wasm-bindgen"$/ { getline; gsub(/[^0-9.]/, ""); print }' Cargo.lock)
have=$(wasm-bindgen-test-runner --version | awk '{ print $2 }')
if [ "$want" != "$have" ]; then
  echo "wasm-bindgen-test-runner $have does not match wasm-bindgen $want in Cargo.lock" >&2
  echo "install with: cargo install wasm-bindgen-cli --version $want --locked" >&2
  exit 1
fi
if [ -z "${CHROMEDRIVER:-}${GECKODRIVER:-}${SAFARIDRIVER:-}" ]; then
  for driver in chromedriver geckodriver safaridriver; do
    if path=$(command -v "$driver"); then
      var=$(echo "$driver" | tr '[:lower:]' '[:upper:]')
      export "$var=$path"
      break
    fi
  done
fi
cargo test --locked --target wasm32-unknown-unknown --all-targets "$@"
