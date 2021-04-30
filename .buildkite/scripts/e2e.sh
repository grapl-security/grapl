#!/usr/bin/env bash

set -euo pipefail

export GRAPL_LOG_LEVEL="DEBUG"
export DUMP_ARTIFACTS="True"

make -j8 test-e2e
