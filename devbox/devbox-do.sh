#!/usr/bin/env bash

set -euo pipefail

################################################################################
# A fair attempt at making remote operations feel as native as possible.
#
# Usage:
# user@chromeos:~/grapl/src/rust$ devbox-do.sh cargo search grapl
# user@chromeos:~/grapl$ devbox-do.sh make test-e2e
################################################################################

THIS_DIR=$(dirname "${BASH_SOURCE[0]}")
readonly THIS_DIR
# shellcheck source-path=SCRIPTDIR
source "${THIS_DIR}/lib.sh"
# shellcheck disable=SC1090
source "${GRAPL_DEVBOX_CONFIG}"

####################
# Step 1: Sync local files to remote
####################
"${THIS_DIR}/devbox-sync.sh"

####################
# Step 2
# - Figure out what local dir in relation to the local grapl root
# - Figure out the corresponding dir remotely
####################
CURRENT_DIR="$(pwd)"
readonly CURRENT_DIR

if [[ ! "${CURRENT_DIR}" =~ ^${GRAPL_DEVBOX_LOCAL_GRAPL} ]]; then
    echo "devbox/do.sh: only works if you're in your local Grapl directory:"
    echo "${GRAPL_DEVBOX_LOCAL_GRAPL}"
    exit 42
fi

CURRENT_DIR_RELATIVE_TO_GRAPL_ROOT="$(
    realpath --relative-to="${GRAPL_DEVBOX_LOCAL_GRAPL}" "${CURRENT_DIR}"
)"
readonly CURRENT_DIR_RELATIVE_TO_GRAPL_ROOT

CURRENT_DIR_REMOTE="${GRAPL_DEVBOX_REMOTE_GRAPL}/${CURRENT_DIR_RELATIVE_TO_GRAPL_ROOT}"

####################
# Step 3: execute the "${@}" remotely
####################
REMOTE_COMMAND="$(
    cat << EOF
cd ${CURRENT_DIR_REMOTE}; ${@}
EOF
)"
# -i = interactive; -c = do this command
"${THIS_DIR}/ssh.sh" -t "bash --login -i -c '${REMOTE_COMMAND}'"
