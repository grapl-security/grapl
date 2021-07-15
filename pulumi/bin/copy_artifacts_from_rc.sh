#!/usr/bin/env bash
# This will grab a subset of keys from `origin/rc`'s Pulumi.testing.yaml.
set -euo pipefail

readonly THIS_DIR=$(dirname "${BASH_SOURCE[0]}")

ensureRightDir() {
    cd $THIS_DIR/../grapl
}
ensureRightDir

readonly GRAPL_ROOT="$(git rev-parse --show-toplevel)"
readonly RC_CONFIG_FILE="/tmp/rc_pulumi_testing.yaml"
readonly CURRENT_STACK="$(pulumi stack --show-name)"

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
        done
}

main() {
    confirmModify

    echo "--- Grab artifacts from origin/rc config file"
    git fetch origin
    git show origin/rc:pulumi/grapl/Pulumi.testing.yaml > ${RC_CONFIG_FILE}
    local -r artifacts=$(pulumi config --config-file="${RC_CONFIG_FILE}" get artifacts)

    echo "--- Grab a subset of keys from artifacts"
    local -r artifacts_subset=$(jq --raw-output --from-file "${THIS_DIR}/subset_of_artifacts.jq" <<< "${artifacts}")
    echo "New artifacts subset: ${artifacts_subset}"

    echo "--- Modify the current stack"
    add_artifacts "${CURRENT_STACK}" "${artifacts_subset}"

    # ensure it worked
    local -r get_result=$(pulumi config get --path artifacts.grapl-nomad-consul-client.us-east-1)
    if [[ $get_result =~ ami-.* ]]; then
        exit 0
    else
        echo "Unexpected result for $(pulumi config get): ${get_result}"
        exit 42
    fi

}

main
