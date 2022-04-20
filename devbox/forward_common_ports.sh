#!/usr/bin/env bash
set -euo pipefail

THIS_DIR=$(dirname "${BASH_SOURCE[0]}")
readonly THIS_DIR

# 4646 = Nomad
# 8500 = Consul
# 1234 = grapl-web-ui
# 16686 = Jaeger

FORWARD_PORTS="4646,8500,1234,16686" \
    "${THIS_DIR}"/ssh.sh
