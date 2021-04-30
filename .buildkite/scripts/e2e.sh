#!/usr/bin/env bash

set -euo pipefail

export GRAPL_LOG_LEVEL="DEBUG"
export DUMP_ARTIFACTS="True"

make test-e2e
