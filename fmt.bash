#!/bin/bash

pushd interpreter &>/dev/null
make fmt
popd &>/dev/null

pushd assembler &>/dev/null
uv format
popd &>/dev/null

pushd compiler &>/dev/null
cargo fmt
popd &>/dev/null
