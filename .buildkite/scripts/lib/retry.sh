#!/usr/bin/env bash

# Usage:
#   retry 3 echo "hello"
function retry() {
    local -r retries="${1}"
    shift

    count=0
    until "$@"; do
        exit=$?
        count=$((count + 1))
        if [ "${count}" -lt "${retries}" ]; then
            echo "Retry ${count}/${retries} exited ${exit}, retrying."
        else
            echo "Retry ${count}/${retries} exited ${exit}, no more retries left."
            exit "${exit}"
        fi
    done
}
