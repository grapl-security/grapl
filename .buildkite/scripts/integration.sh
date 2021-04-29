#!/usr/bin/env bash

set -euo pipefail

echo "unset RUSTC_WRAPPER" > rust_env.sh
chmod 777 rust_env.sh

export GRAPL_LOG_LEVEL=DEBUG
export GRAPL_RUST_ENV_FILE=rust_env.sh

# Retrieve ZIPs
buildkite-agent artifact download 'src/js/grapl-cdk/zips/*.zip' .

make test-integration-base
