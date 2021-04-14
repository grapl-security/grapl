#!/usr/bin/env bash

set -euo pipefail

export GRAPL_LOG_LEVEL=DEBUG

make test-integration
