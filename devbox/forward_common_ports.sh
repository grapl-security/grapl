#!/usr/bin/env bash
set -euo pipefail

THIS_DIR=$(dirname "${BASH_SOURCE[0]}")
readonly THIS_DIR

# 4646 = Nomad
# 8500 = Consul
# 1234 = grapl-web-ui

FORWARD_PORTS="4646,8500,1234" \
    "${THIS_DIR}"/ssh.sh
