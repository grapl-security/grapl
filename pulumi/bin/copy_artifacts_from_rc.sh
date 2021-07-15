#!/usr/bin/env bash
# This will grab a subset of keys from `origin/rc`'s Pulumi.testing.yaml.
set -euo pipefail

readonly THIS_DIR=$(dirname "${BASH_SOURCE[0]}")

ensureRightDir() {
    cd $THIS_DIR/../grapl
}
ensureRightDir

# Read from $1 or default to what `pulumi stack` says
readonly CURRENT_STACK="${1:-$(pulumi stack --show-name)}"
readonly GRAPL_ROOT="$(git rev-parse --show-toplevel)"
readonly RC_CONFIG_FILE="/tmp/rc_pulumi_testing.yaml"

confirmModify() {
    read -r -p "This will modify your ${CURRENT_STACK} config. Continue (y/n)?" choice
    case "${choice}" in
        y | Y) echo "Okay!" ;;
        *) exit 42 ;;
    esac
}

add_artifacts() {
    # Slightly tweaked version of what we have in
    # .buildkite/shared/lib/rc.sh"
    # removes the --cwd, --stack stuff
    local -r stack="${1}"
    local -r input_json="${2}"

    source "${GRAPL_ROOT}/.buildkite/shared/lib/json_tools.sh"
    flattened_input_json=$(flatten_json "${input_json}")

    jq -r 'to_entries | .[] | [.key, .value] | @tsv' <<< "${flattened_input_json}" |
        while IFS=$'\t' read -r key value; do
            pulumi config set \
                --path "artifacts.${key}" \
                "${value}"
                --stack "${CURRENT_STACK}"
        done
}

main() {
    confirmModify

    echo "--- Grab artifacts from origin/rc config file"
    git fetch origin
    git show origin/rc:pulumi/grapl/Pulumi.testing.yaml > ${RC_CONFIG_FILE}
    local -r artifacts=$(pulumi config --config-file="${RC_CONFIG_FILE}" get artifacts)

    echo "--- Modify the current stack"
    add_artifacts "${CURRENT_STACK}" "${artifacts}"

    pulumi config --stack "${CURRENT_STACK}"

    echo "--- !!! VERY IMPORTANT !!!"
    echo "Any artifacts defined in here *WILL* override anything you built locally, "
    echo " so selectively remove whatever you happen to be working on at a given time."

}

main
