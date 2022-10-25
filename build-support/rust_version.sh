#!/usr/bin/env bash

# Extract the version of Rust that we are using from our
# rust-toolchain.toml file.
#
# This can be used as a source of truth any place that need access to
# this information (e.g., build arguments for container images to
# ensure base images are in-sync with our Rust version)
#
# It can safely be invoked from anywhere in the repository, as it
# resolves the toolchain file from the repository root.
set -euo pipefail

repo_root="$(git rev-parse --show-toplevel 2> /dev/null)"
readonly repo_root
readonly toolchain_file="${repo_root}/src/rust/rust-toolchain.toml"

grep channel "${toolchain_file}" | sed -E 's/channel = "(.*)"/\1/g'
