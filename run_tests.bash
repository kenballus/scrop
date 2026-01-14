#!/usr/bin/env bash

set -euo pipefail

./build.bash
pushd compiler &>/dev/null
cargo test
popd &>/dev/null


for t in tests/*; do
    if diff <(./compiler/target/debug/compiler < "$t/in" 2>/dev/null | uv run ./assembler/main.py | ./interpreter/interpreter) "$t/out"; then
        echo "$t: pass"
    else
        echo "$t: fail"
    fi
done
