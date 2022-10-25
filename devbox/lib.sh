#!/usr/bin/env bash
set -euo pipefail

export GRAPL_DEVBOX_DIR="${HOME}/.grapl_devbox"
export GRAPL_DEVBOX_CONFIG="${GRAPL_DEVBOX_DIR}/config"

########################################
# Print helpers
########################################
# Re-export the logging functions
# shellcheck source-path=SCRIPTDIR
source "$(dirname "${BASH_SOURCE[0]}")"/../src/sh/log.sh

# I'm using HTML <h1>, <h2> terminology for "just how big is this?"
echo_h1() {
    log "\n========================================"
    log "==>" "${@}"
    log "========================================"
}
