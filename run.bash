#!/bin/bash

set -euo pipefail

./compiler/target/debug/compiler | uv run ./assembler/main.py | ./interpreter/interpreter
