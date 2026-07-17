#!/bin/bash

set -euo pipefail

./interpreter/interpreter <(./compiler/target/debug/compiler | uv run ./assembler/main.py)
