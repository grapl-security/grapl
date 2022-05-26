#!/usr/bin/env bash
set -euo pipefail

################################################################################
# This is more of a building-block, for day-to-day you likely want devbox-do.sh.
#
# Usage:
# ./devbox/ssh.sh
# ./devbox/ssh.sh -- echo "hello"
# FORWARD_PORTS=4646,8500,1234 ./devbox/ssh.sh
################################################################################

THIS_DIR=$(dirname "${BASH_SOURCE[0]}")
# shellcheck source-path=SCRIPTDIR
source "${THIS_DIR}/lib.sh"
# shellcheck disable=SC1090
source "${GRAPL_DEVBOX_CONFIG}"

########################################
# Main logic
########################################

declare -a PORT_FORWARDING_ARGS=()
if [ -v FORWARD_PORTS ]; then
    # FORWARD_PORTS_ARRAY = FORWARD_PORTS.split(",")
    IFS="," read -r -a FORWARD_PORTS_ARRAY <<< "$FORWARD_PORTS"
    for port in "${FORWARD_PORTS_ARRAY[@]}"; do
        PORT_FORWARDING_ARGS+=("-L" "${port}:localhost:${port}")
    done
fi

# Each of these keys is set in the config by devbox/provision/provision.sh
AWS_REGION="${GRAPL_DEVBOX_REGION}" \
    ssh \
    -o "IdentitiesOnly=yes" \
    -i "${GRAPL_DEVBOX_PRIVATE_KEY_FILE}" \
    "${PORT_FORWARDING_ARGS[@]}" \
    "${GRAPL_DEVBOX_USER}@${GRAPL_DEVBOX_INSTANCE_ID}" \
    "${@}"
