#!/usr/bin/env bash

set -euo pipefail

pushd ./interpreter
make
popd

pushd ./compiler
cargo build
popd
