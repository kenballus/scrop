#!/usr/bin/env bash

set -euo pipefail

pushd ./interpreter &>/dev/null
make
popd &>/dev/null

pushd ./compiler &>/dev/null
cargo build
popd &>/dev/null
