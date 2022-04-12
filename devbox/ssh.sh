#!/usr/bin/env bash

set -euo pipefail

THIS_DIR=$(dirname "${BASH_SOURCE[0]}")
# shellcheck source-path=SCRIPTDIR
source "${THIS_DIR}/lib.sh"
# shellcheck disable=SC1090
source "${GRAPL_DEVBOX_CONFIG}"

########################################
# Main logic
########################################
# Each of these keys is set in the config by devbox/provision/provision.sh

AWS_REGION="${GRAPL_DEVBOX_REGION}" \
    ssh \
    -o "IdentitiesOnly=yes" \
    -i "${GRAPL_DEVBOX_PRIVATE_KEY_FILE}" \
    "${GRAPL_DEVBOX_USER}@${GRAPL_DEVBOX_INSTANCE_ID}" \
    "${@}"
