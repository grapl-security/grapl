#!/usr/bin/env bash
set -euo pipefail

################################################################################
# This is more of a building-block, for day-to-day you likely want devbox-do.sh.
#
# Usage:
# ./devbox/ssh.sh
# ./devbox/ssh.sh -- echo "hello"
# FORWARD_PORT=4646 ./devbox/ssh.sh  # great for accessing a remote localhost!
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
if [ -v FORWARD_PORT ]; then
    PORT_FORWARDING_ARGS+=("-L" "${FORWARD_PORT}:localhost:${FORWARD_PORT}")
fi

# Each of these keys is set in the config by devbox/provision/provision.sh
AWS_REGION="${GRAPL_DEVBOX_REGION}" \
    ssh \
    -o "IdentitiesOnly=yes" \
    -i "${GRAPL_DEVBOX_PRIVATE_KEY_FILE}" \
    "${PORT_FORWARDING_ARGS[@]}" \
    "${GRAPL_DEVBOX_USER}@${GRAPL_DEVBOX_INSTANCE_ID}" \
    "${@}"
