#!/usr/bin/env bash

set -euo pipefail

pushd ./interpreter &>/dev/null
make
popd &>/dev/null

pushd ./assembler &>/dev/null
uv run mypy ./*.py
popd &>/dev/null

pushd ./compiler &>/dev/null
cargo build
popd &>/dev/null
