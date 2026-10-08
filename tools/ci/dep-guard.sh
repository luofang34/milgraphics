#!/usr/bin/env bash
# The library's normal dependency tree (every target) must stay free of
# JavaScript runtimes and interop, JVM bridges, GUI, GPU, windowing and map
# engines. Dev-dependencies (browser test tooling) are not part of the library.
set -euo pipefail
tree=$(cargo tree -p milgraphics -e normal,build --target all --prefix none --no-dedupe)
banned='^(boa[a-z_]*|v8|rusty_v8|deno_[a-z_]+|quickjs[a-z_-]*|rquickjs[a-z_-]*|js-sys|wasm-bindgen[a-z-]*|web-sys|mozjs|duktape[a-z-]*|neon|napi[a-z-]*|jni[a-z-]*|j4rs|egui[a-z_-]*|eframe|winit|wgpu[a-z_-]*|glow|ash|metal|vulkano|maplibre[a-z_-]*|milsymbol|reqwest|hyper|ureq|tokio|async-std|getrandom) '
if echo "$tree" | grep -Eq "$banned"; then
  echo "forbidden dependency in milgraphics:" >&2
  echo "$tree" | grep -E "$banned" | sort -u >&2
  exit 1
fi
echo "milgraphics dependency tree is within bounds:"
echo "$tree" | sort -u
