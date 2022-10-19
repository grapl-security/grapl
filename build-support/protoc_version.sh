#!/usr/bin/env bash
# Extract the version of protoc that we are using from pants.toml.
#
# This can be used as a source of truth any place that need access to
# this information (e.g., build arguments for container images to
# ensure base images are in-sync with our Rust version)
#
# It can safely be invoked from anywhere in the repository, as it
# resolves the .protoc-version file from the repository root.
set -euo pipefail

repo_root="$(git rev-parse --show-toplevel 2> /dev/null)"
readonly repo_root

# Ideally, we'd use something like https://github.com/kislyuk/yq to
# introspect `pants.toml`, but given that the entry in pants.toml for
# the protoc version is relatively unique and easy to grep for,
# we'll just use that.

# -n = quiet
# -r = extended regex (enables \1)
# /\1/ = only return what's in capture group 1 - the contents of the parens
sed -nr 's/^protoc-version = "(.*)"$/\1/p' "${repo_root}/pants.toml"
