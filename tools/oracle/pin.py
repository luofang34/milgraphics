"""Prints values from tools/oracle/pin.json for the shell scripts.

Usage:
  pin.py value KEY...            the value at that key path
  pin.py files REFERENCE         "path sha256" for every pinned file of a reference
  pin.py jars                    "name url sha256" for every oracle jar
"""
import json
import os
import sys

with open(os.path.join(os.path.dirname(os.path.abspath(__file__)), "pin.json"), encoding="utf-8") as f:
    pin = json.load(f)

command, args = sys.argv[1], sys.argv[2:]
if command == "value":
    value = pin
    for key in args:
        value = value[key]
    print(value)
elif command == "files":
    for group in pin["references"][args[0]]["files"].values():
        for path, sha in group.items():
            print(path, sha)
elif command == "jars":
    for name, jar in pin["oracle_runtime"]["jars"].items():
        print(name, jar["url"], jar["sha256"])
else:
    sys.exit(f"unknown command {command}")
