#!/usr/bin/env bash
set -euo pipefail

THIS_DIR=$(dirname "${BASH_SOURCE[0]}")
readonly THIS_DIR
# shellcheck source-path=SCRIPTDIR
source "${THIS_DIR}/lib.sh"

# 4646 = Nomad
# 8500 = Consul
# 1234 = grapl-web-ui
# 16686 = Jaeger

export FORWARD_PORTS="4646,8500,1234,16686"

"${THIS_DIR}"/ssh.sh -t "$(
    cat << EOF
    echo -e "Forwarding ports from devbox: $(bright_green "${FORWARD_PORTS}")."
    echo "Hit <ENTER> to end forwarding.";
    read
EOF
)"
