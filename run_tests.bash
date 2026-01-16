#!/usr/bin/env bash

set -euo pipefail

./build.bash
pushd compiler &>/dev/null
cargo test
popd &>/dev/null


for t in tests/*; do
    if diff <(./run.bash < "$t/in") "$t/out"; then
        printf "$t: \x1b[32mok\x1b[0m\n"
    else
        printf "$t: \x1b[31mfail\x1b[0m\n"
    fi
done
