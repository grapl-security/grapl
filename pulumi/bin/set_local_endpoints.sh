#!/usr/bin/env bash
# This grabs the endpoints from nomad and sets them in the current pulumi stack
set -euo pipefail

THIS_DIR=$(dirname "${BASH_SOURCE[0]}")
readonly THIS_DIR

ensureRightDir() {
    cd "$THIS_DIR/../grapl"
}
ensureRightDir

# Read from $1 or default to what `pulumi stack` says
CURRENT_STACK="${1:-$(pulumi stack --show-name)}"
GRAPL_ROOT="$(git rev-parse --show-toplevel)"
readonly CURRENT_STACK
readonly GRAPL_ROOT

confirmModify() {
    read -r -p "This will modify your ${CURRENT_STACK} config. Continue (y/n)?" choice
    case "${choice}" in
        y | Y) echo "Okay!" ;;
        *) exit 42 ;;
    esac
}

get_endpoints() {
  SQS_ENDPOINT=$(docker ps | grep sqs | awk '{ print $11 }' | awk -F"-" '{ print $1 }')
  DYNAMO_ENDPOINT=$(docker ps | grep dynamo | awk '{ print $12 }' | awk -F"-" '{ print $1 }')
  LOCALSTACK_ENDPOINT="http://localhost:4566"

  JSON_STRING=$( jq -n \
    --arg sqs "$SQS_ENDPOINT" \
    --arg dynamo "$DYNAMO_ENDPOINT" \
    --arg localstack "$LOCALSTACK_ENDPOINT" \
                  '{apigateway: $localstack,
                  cloudwatch: $localstack,
                  cloudwatchevents: $localstack,
                  cloudwatchlogs: $localstack,
                  dynamodb: $dynamo,
                  ec2: $localstack,
                  iam: $localstack,
                  lambda: $localstack,
                  s3: $localstack,
                  secretsmanager: $localstack,
                  sns: $localstack,
                  sqs: $sqs,
                  lambda: $localstack,
                  lambda: $localstack}' )

  echo "$JSON_STRING"
}

add_endpoints() {
    # Slightly tweaked version of what we have in
    # .buildkite/shared/lib/rc.sh"
    # removes the --cwd, --stack stuff
    local -r stack="${1}"
    local -r input_json="${2}"

    # shellcheck source=/dev/null
    source "${GRAPL_ROOT}/.buildkite/shared/lib/json_tools.sh"
    flattened_input_json=$(flatten_json "${input_json}")

    jq -r 'to_entries | .[] | [.key, .value] | @tsv' <<< "${flattened_input_json}" |
        while IFS=$'\t' read -r key value; do
            pulumi config set \
                --path "aws:endpoints.${key}" \
                "${value}" \
                --stack "${stack}" \
                --plaintext
        done
}

main() {
    confirmModify

    local -r endpoints=$(get_endpoints)
    echo ${endpoints}

    echo "--- Modify the current stack"
    add_endpoints "${CURRENT_STACK}" "${endpoints}"

    pulumi config --stack "${CURRENT_STACK}"
}

main
