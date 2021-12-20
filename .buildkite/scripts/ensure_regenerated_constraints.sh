#!/usr/bin/env bash

set -euo pipefail
################################################################################
# Ensure that, if `requirements.txt` has changed, `constraints.txt` has
# changed as well.
################################################################################

file_same() {
    git diff --exit-code --quiet origin/main -- "${1}"
}

if ! file_same "3rdparty/python/requirements.txt"; then
    if file_same "3rdparty/python/constraints.txt"; then
        echo "Please run ./build-support/manage_virtualenv.sh regenerate-constraints after changing requirements.txt"
        exit 1
    fi
fi
