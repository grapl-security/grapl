#!/usr/bin/env bash

set -euo pipefail
set -o xtrace

THIS_DIR=$(dirname "${BASH_SOURCE[0]}")
# shellcheck source-path=SCRIPTDIR
source "${THIS_DIR}/lib.sh"

REMOTE_DIR="/home/ubuntu/repos/grapl"
"${THIS_DIR}/ssh.sh" -- mkdir --parents "${REMOTE_DIR}"

rsync --archive --verbose --progress \
    --rsh "${THIS_DIR}/ssh.sh" \
    /home/wimax/src/repos/grapl \
    "ubuntu@$(get_devbox_config instance_id):${REMOTE_DIR}"
