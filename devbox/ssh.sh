#!/usr/bin/env bash

set -euo pipefail

readonly GRAPL_DEVBOX_DIR="${HOME}/.grapl_devbox"
readonly GRAPL_DEVBOX_CONFIG="${GRAPL_DEVBOX_DIR}/config.env"
source "${GRAPL_DEVBOX_CONFIG}"

########################################
# Main logic
########################################
# Each of these keys is set in the config by devbox/provision/provision.sh

AWS_REGION="${GRAPL_DEVBOX_REGION}" \
    ssh \
    -o "IdentitiesOnly=yes" \
    -i "${GRAPL_DEVBOX_PRIVATE_KEY_FILE}" \
    ubuntu@"${GRAPL_DEVBOX_INSTANCE_ID}"
