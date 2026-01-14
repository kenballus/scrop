#!/usr/bin/env bash

set -euo pipefail

./build.bash
pushd compiler &>/dev/null
cargo test
popd &>/dev/null


for t in tests/*; do
    if diff <(./run.bash < "$t/in") "$t/out"; then
        echo "$t: pass"
    else
        echo "$t: fail"
    fi
done
