#!/usr/bin/env bash

set -euo pipefail

THIS_DIR=$(dirname "${BASH_SOURCE[0]}")
# shellcheck source-path=SCRIPTDIR
source "${THIS_DIR}/lib.sh"
# shellcheck disable=SC1090
source "${GRAPL_DEVBOX_CONFIG}"

# the --include stuff was inspired by https://stackoverflow.com/posts/63438492/revisions

rsync --archive --info=progress2 \
    --include='**.gitignore' --exclude='**/.git' --filter=':- .gitignore' --delete-after \
    --rsh "${THIS_DIR}/ssh.sh" \
    "${GRAPL_DEVBOX_LOCAL_GRAPL}/" \
    ":${GRAPL_DEVBOX_REMOTE_GRAPL}"
