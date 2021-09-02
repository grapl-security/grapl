#!/bin/bash
set -euo pipefail

curl_quiet() {
    # shellcheck disable=SC2086
    curl --location --silent --show-error $@
}

nomad_node_id() {
    # Assume there's only 1 node
    nomad node status -self -json | jq --raw-output '.ID'
}

nomad_dispatch() {
    # Grab the new Job ID from the Dispatch command

    local -r parameterized_batch_job="${1}"
    # Output looks like
    # Dispatched Job ID = integration-tests/dispatch-1630610999-b227c2f8
    # Evaluation ID     = 9b11828e
    local -r dispatch_output=$(nomad job dispatch "${parameterized_batch_job}")
    local -r job_id=$(echo "${dispatch_output}" | head -n 1 | cut -d " " -f 5)
    echo "${job_id}"
}

nomad_get_job() {
    # Assumes there's a single job matching job_id
    local -r job_id="${1}"
    curl_quiet --request GET "http://localhost:4646/v1/jobs?prefix=${job_id}" | jq -r ".[0]"
}

nomad_dispatch_status() {
    # returns one of "pending", "running", "dead"
    local -r job_id="${1}"
    local -r curl_result=$(nomad_get_job "${job_id}")
    echo "${curl_result}" | jq -r ".Status"
}
