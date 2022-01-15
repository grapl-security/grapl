#!/usr/bin/env bash
# This will grab a subset of keys from `origin/rc`'s Pulumi.testing.yaml.
set -euo pipefail
set -o xtrace

THIS_DIR=$(dirname "${BASH_SOURCE[0]}")
readonly THIS_DIR

ensureRightDir() {
    cd "$THIS_DIR/../grapl"
}
ensureRightDir

# Read from $1 or default to what `pulumi stack` says
CURRENT_STACK="${1:-grapl/$(pulumi stack --show-name)}"
RC_CONFIG_FILE="/tmp/rc_pulumi_testing.yaml"
readonly CURRENT_STACK
readonly RC_CONFIG_FILE

confirmModify() {
    # Skip this check if FORCE_MODIFY=1
    # (this could be a getopts, but, meh)
    if [[ -n "${FORCE_MODIFY:-}" ]]; then
        return 0
    fi

    read -r -p "This will modify your ${CURRENT_STACK} config. Continue (y/n)?" choice
    case "${choice}" in
        y | Y) echo "Okay!" ;;
        *) exit 42 ;;
    esac
}

add_artifacts() {
    local -r stack="${1}"
    local -r input_json="${2}"

    jq -r 'to_entries | .[] | [.key, .value] | @tsv' <<< "${input_json}" |
        while IFS=$'\t' read -r key value; do
            pulumi config set \
                --path "artifacts.${key}" \
                "${value}" \
                --stack "${stack}"
        done
}

main() {
    confirmModify

    echo "--- Grab artifacts from origin/rc config file"
    git fetch origin
    git show origin/rc:pulumi/grapl/Pulumi.testing.yaml > ${RC_CONFIG_FILE}
    local -r artifacts=$(pulumi config --config-file="${RC_CONFIG_FILE}" --stack="${CURRENT_STACK}" get artifacts)

    echo "--- Modify the current stack"
    add_artifacts "${CURRENT_STACK}" "${artifacts}"

    pulumi config --stack "${CURRENT_STACK}"

    echo "!!! VERY IMPORTANT !!!"
    echo "Any artifacts defined in here *WILL* override any local images."
    echo " so selectively remove whatever you happen to be working on at a given time."

}

main
