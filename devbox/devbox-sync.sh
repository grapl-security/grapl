#!/usr/bin/env bash

set -euo pipefail

THIS_DIR=$(dirname "${BASH_SOURCE[0]}")
# shellcheck source-path=SCRIPTDIR
source "${THIS_DIR}/lib.sh"
# shellcheck disable=SC1090
source "${GRAPL_DEVBOX_CONFIG}"

readonly VERBOSE_DEVBOX_SYNC="${VERBOSE_DEVBOX_SYNC:-0}"

rsync_progress_args=()
if [[ ${VERBOSE_DEVBOX_SYNC} -ne 0 ]]; then
    rsync_progress_args+=(--progress --verbose)
else
    rsync_progress_args+=(--info=progress2)
fi

# the --include stuff was inspired by https://stackoverflow.com/posts/63438492/revisions

if ! rsync --archive "${rsync_progress_args[@]}" \
    --include='**.gitignore' --exclude='**/.git' --filter=':- .gitignore' --delete-after \
    --rsh "${THIS_DIR}/ssh.sh" \
    "${GRAPL_DEVBOX_LOCAL_GRAPL}/" \
    ":${GRAPL_DEVBOX_REMOTE_GRAPL}" \
    ; then
    # TODO in the future: maybe throw an `aws s3 ls` in or something to detect
    # that the cause is indeed AWS
    echo_h1 "$(bright_red "It looks like devbox-sync failed. Maybe you need to 'aws sso login'?")"
    exit 42
fi

echo_h1 "$(bright_green "Devbox-sync complete")"
