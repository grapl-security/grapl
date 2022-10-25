#!/usr/bin/env bash
# Extract the version of protoc that we are using from pants.toml.
#
# This can be used as a source of truth any place that need access to
# this information (e.g., build arguments for container images to
# ensure base images are in-sync with our Rust version)
#
# It can safely be invoked from anywhere in the repository.
set -euo pipefail

repo_root="$(git rev-parse --show-toplevel 2> /dev/null)"
readonly repo_root

(
    cd "${repo_root}"
    # This magical bit of `jq` extracts the effective version of
    # `protoc` from Pants, whether we're using the default version or
    # have pinned a specific version in `pants.toml`.
    ./pants help-all |
        jq --raw-output \
            '.scope_to_help_info.protoc.advanced |
            .[] |
            select(.config_key == "version") |
            .value_history.ranked_values[-1].value'
)
