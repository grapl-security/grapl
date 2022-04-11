#!/usr/bin/env bash

set -euo pipefail

readonly GRAPL_DEVBOX_DIR="${HOME}/.grapl_devbox"
readonly GRAPL_DEVBOX_CONFIG="${GRAPL_DEVBOX_DIR}/config"

########################################
# Helper functions
########################################

get_devbox_config() {
    local -r key="${1}"
    jq --raw-output --exit-status ".[\"${key}\"]" "${GRAPL_DEVBOX_CONFIG}"
}

########################################
# Main logic
########################################
# Each of these keys is set in the config by devbox/provision/provision.sh

AWS_REGION="$(get_devbox_config region)" \
    ssh \
    -o "IdentitiesOnly=yes" \
    -i "$(get_devbox_config private_key_file)" \
    ubuntu@"$(get_devbox_config instance_id)"
