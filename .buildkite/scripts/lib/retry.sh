#!/usr/bin/env bash

# Usage:
#   _retry 3 3 false
#   would retry 3x with a backoff of 3, 9, exit
function _retry() {
    local -r retries="${1}"
    shift

    local -r exponential_backoff="${1}"
    shift

    count=0
    until "$@"; do
        exit=$?
        count=$((count + 1))
        if [ "${count}" -lt "${retries}" ]; then
            echo "Retry ${count}/${retries} exited ${exit}, retrying."
            if [[ ${exponential_backoff} -ne 0 ]]; then
                sleep_time=$(($exponential_backoff ** count))
                echo " >> Sleeping ${sleep_time}"
                sleep "${sleep_time}"
            fi
        else
            echo "Retry ${count}/${retries} exited ${exit}, no more retries left."
            exit "${exit}"
        fi
    done
}

# Usage:
#   retry_no_cooldown 3 echo "hello"
function retry_no_cooldown() {
    local -r retries="${1}"
    shift

    _retry "${retries}" 0 "${@}"
}

# Retries with an exponential-of-2 cooldown.
# Usage:
#   retry 3 echo "hello"
function retry() {
    local -r retries="${1}"
    shift

    _retry "${retries}" 2 "${@}"
}
