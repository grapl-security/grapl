#!/usr/bin/env bash

########################################
# Usually you won't need to use this script, but it lets you claw back files
# from your devbox.
# Example: reformatting this very file
#   ./devbox/devbox-do.sh make format-shell
#   ./devbox/devbox-sync-back.sh devbox/devbox-sync-back.sh
# Example: reformatting all your HCLs
#   ./devbox/devbox-do.sh make format-hcl
#   ./devbox/devbox-sync-back.sh **/*.hcl

set -euo pipefail

THIS_DIR=$(dirname "${BASH_SOURCE[0]}")
# shellcheck source-path=SCRIPTDIR
source "${THIS_DIR}/lib.sh"
# shellcheck disable=SC1090
source "${GRAPL_DEVBOX_CONFIG}"

readonly SSH_SCRIPT="${THIS_DIR}/ssh.sh"
readonly VERBOSE_DEVBOX_SYNC="${VERBOSE_DEVBOX_SYNC:-0}"

# Get the relative paths of all the files
mapfile -t resolved_files < <(
    shopt -s globstar
    ls -1 "${@}"
)

rsync_wrapper() {
    local -a rsync_args=()
    if [[ ${VERBOSE_DEVBOX_SYNC} -ne 0 ]]; then
        rsync_args+=(--progress --verbose)
    else
        rsync_args+=(--info=progress2)
    fi
    if [ -v DRY_RUN ]; then
        rsync_args+=(--list-only)
    fi

    # the `--files-from` is a file redirect, containing each member of the array
    # followed by a newline.
    rsync --archive \
        "${rsync_args[@]}" \
        --files-from=<(printf "%s\n" "${resolved_files[@]}") \
        --rsh "${SSH_SCRIPT}" \
        ":${GRAPL_DEVBOX_REMOTE_GRAPL}/" \
        "${GRAPL_DEVBOX_LOCAL_GRAPL}/"
}

if DRY_RUN=1 rsync_wrapper; then

    echo_h1 "$(bright_green "^^^ Overwrite these host files with devbox files?")"
    echo "(Enter or Ctrl-C)"
    read -r
else
    exit 42
fi

rsync_wrapper
