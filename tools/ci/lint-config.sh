#!/usr/bin/env bash
# Every workspace member inherits the workspace lint table, and clippy.toml
# carries the test exemptions the forbidden lints rely on.
set -euo pipefail
python3 - <<'PY'
import sys
import tomllib

with open("Cargo.toml", "rb") as f:
    root = tomllib.load(f)
failures = []
for member in root["workspace"]["members"]:
    with open(f"{member}/Cargo.toml", "rb") as f:
        manifest = tomllib.load(f)
    if manifest.get("lints") != {"workspace": True}:
        failures.append(f"{member}: must set [lints] workspace = true")
clippy = root["workspace"]["lints"]["clippy"]
for name in ["unwrap_used", "expect_used", "panic", "todo", "unimplemented",
             "unreachable", "exit", "mem_forget", "get_unwrap"]:
    if clippy.get(name) != "forbid":
        failures.append(f"workspace.lints.clippy.{name} must be 'forbid'")
if root["workspace"]["lints"]["rust"].get("unsafe_code") != "forbid":
    failures.append("workspace.lints.rust.unsafe_code must be 'forbid'")
with open("clippy.toml", "rb") as f:
    conf = tomllib.load(f)
for key in ["allow-dbg-in-tests", "allow-expect-in-tests", "allow-indexing-slicing-in-tests",
            "allow-panic-in-tests", "allow-print-in-tests", "allow-unwrap-in-tests"]:
    if conf.get(key) is not True:
        failures.append(f"clippy.toml: {key} must be true")
if failures:
    print("\n".join(failures), file=sys.stderr)
    sys.exit(1)
print("lint configuration is complete")
PY
