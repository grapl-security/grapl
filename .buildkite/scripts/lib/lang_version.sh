#!/usr/bin/env bash

# Get the python version
python_version() {
    cat .python-version
}
# Hacky way to extract a value from a TOML file T_T
#
# This at least automatically keeps things in sync with with our Rust
# toolchain.
rust_version() {
    grep channel src/rust/rust-toolchain.toml | sed -E 's/channel = "(.*)"/\1/g'
}
