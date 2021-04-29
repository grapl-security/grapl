#!/usr/bin/env bash

set -euo pipefail

export GRAPL_LOG_LEVEL="DEBUG"
export DUMP_ARTIFACTS="True"

# Retrieve ZIPs
buildkite-agent artifact download 'src/js/grapl-cdk/zips/*.zip' .

make test-e2e-base
