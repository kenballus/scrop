#!/usr/bin/env bash

set -euo pipefail

./build.bash
pushd compiler &>/dev/null
cargo test
popd &>/dev/null


for t in tests/*; do
    printf "$t ... "
    if diff <(./run.bash < "$t/in") "$t/out"; then
        printf '\x1b[32mok'
    else
        printf '\x1b[31mfail'
    fi
    printf '\x1b[0m\n'
done
