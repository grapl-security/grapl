#!/usr/bin/env bash

set -euo pipefail

THIS_DIR=$(dirname "${BASH_SOURCE[0]}")
# shellcheck source-path=SCRIPTDIR
source "${THIS_DIR}/lib.sh"
# shellcheck disable=SC1090
source "${GRAPL_DEVBOX_CONFIG}"

readonly SSH_SCRIPT="${THIS_DIR}/ssh.sh"
readonly VERBOSE_DEVBOX_SYNC="${VERBOSE_DEVBOX_SYNC:-0}"

rsync_progress_args=()
# These seemingly similar args - --info=progress and --progress2 - 
# have very different outputs. Notably, --progress outputs each file.
if [[ ${VERBOSE_DEVBOX_SYNC} -ne 0 ]]; then
    rsync_progress_args+=(--progress --verbose)
else
    rsync_progress_args+=(--info=progress2)
fi

# the --include stuff was inspired by https://stackoverflow.com/posts/63438492/revisions
if ! rsync --archive "${rsync_progress_args[@]}" \
    --include='**.gitignore' --exclude='**/.git' --filter=':- .gitignore' --delete-after \
    --rsh "${SSH_SCRIPT}" \
    "${GRAPL_DEVBOX_LOCAL_GRAPL}/" \
    ":${GRAPL_DEVBOX_REMOTE_GRAPL}" \
    ; then
    # TODO in the future: maybe throw an `aws s3 ls` in or something to detect
    # that the cause is indeed AWS
    echo_h1 "$(bright_red "It looks like devbox-sync failed. Maybe you need to 'aws sso login'?")"
    exit 42
fi

readonly LOW_SPACE_WARNING_LIMIT_GB=5
warn_if_low_space() {
    local -r df_result="$(${SSH_SCRIPT} -- df /dev/root --block-size=G --output=avail | tail -n1)"
    # <any whitespaces> <capture the number> <the letter G>
    local -r regex='^\W*([0-9]+)G$'
    if ! [[ ${df_result} =~ ${regex} ]]; then
        echo >&2 "Couldn't detect remaining space"
        return
    fi
    local -r remaining_gigs="${BASH_REMATCH[1]}"
    if [[ ${remaining_gigs} -lt ${LOW_SPACE_WARNING_LIMIT_GB} ]]; then
        echo_h1 "$(bright_yellow "WARNING: devbox has ${remaining_gigs}GB space remaining")"
    fi
}

warn_if_low_space

echo_h1 "$(bright_green "Devbox-sync complete")"
