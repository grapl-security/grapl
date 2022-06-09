#!/usr/bin/env bash

########################################################################
# dependencies.sh: Functions for ensuring that expected binaries are
# present.
#
# Scripts that rely on binaries that are not typically available
# should call these functions early in their execution to perform
# pre-flight sanity checks and provide early feedback that a necessary
# binary cannot be found.
########################################################################

# shellcheck source-path=SCRIPTDIR
source "$(dirname "${BASH_SOURCE[0]}")"/log.sh

CHECK_MARK_EMOJI="✅"
CROSS_MARK_EMOJI="❌"
PARTY_POPPER_EMOJI="🎉"
MAGNIFYING_GLASS_EMOJI="🔎"

# Check to see if a given binary is present on $PATH.
#
# Consider using `expect_binaries` (below) instead of this in your
# scripts.
function expect_binary() {
    binary="${1}"
    if resolved=$(command -v "${binary}"); then
        log "${CHECK_MARK_EMOJI}" Using "$(bright_white "${binary}")" from "$(bright_white "${resolved}")"
    else
        log "${CROSS_MARK_EMOJI}" Cannot find "$(bright_white "${binary}")" binary in \$PATH
        false
    fi
}

# Checks to see if all given binaries are on $PATH. If any are
# missing, exit the program.
#
# Generally speaking, prefer to call this function over
# `expect_binary` in scripts, as it provides a better end-user
# experience.
function expect_binaries() {
    log "${MAGNIFYING_GLASS_EMOJI}" Checking for required binaries...

    success=1

    for binary in "$@"; do
        if ! expect_binary "${binary}"; then
            success=0
        fi
    done

    if [ "${success}" -eq 1 ]; then
        log "${PARTY_POPPER_EMOJI}" Success!
    else
        fatal "Missing one or more required binaries"
    fi
}
