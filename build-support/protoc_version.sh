#!/usr/bin/env bash
# Extract the version of Python that we are using from our
# .python-version file.
#
# This can be used as a source of truth any place that need access to
# this information (e.g., build arguments for container images to
# ensure base images are in-sync with our Rust version)
#
# It can safely be invoked from anywhere in the repository, as it
# resolves the .python-version file from the repository root.
set -euo pipefail

repo_root="$(git rev-parse --show-toplevel 2> /dev/null)"
readonly repo_root
cat "${repo_root}/.protoc-version"