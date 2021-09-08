#!/bin/bash
set -euo pipefail

NOMAD_ENDPOINT="http://localhost:4646"
curl_quiet() {
    # shellcheck disable=SC2068
    curl --location --silent --show-error $@
}

nomad_node_id() {
    # Assume there's only 1 node
    nomad node status -self -json | jq --raw-output '.ID'
}

nomad_dispatch() {
    # Grab the new Job ID from the Dispatch command
    # TODO: Probably just use the Nomad HTTP api for this

    local -r parameterized_batch_job="${1}"
    # Output looks like
    #
    local -r dispatch_output=$(curl_quiet --request POST --data "{}" "${NOMAD_ENDPOINT}/v1/job/${parameterized_batch_job}/dispatch")
    local -r job_id=$(echo "${dispatch_output}" | jq -r ".DispatchedJobID")
    echo "${job_id}"
}

nomad_get_allocation() {
    local -r job_id=$1
    # Output looks like https://www.nomadproject.io/api-docs/jobs#sample-response-5
    local -r curl_result=$(curl_quiet "${NOMAD_ENDPOINT}/v1/job/${job_id}/allocations")
    echo "${curl_result}"
}

nomad_get_per_task_results() {
    local -r job_id=$1

    # This assumes there's only 1 Allocation per Job. That's probably right.
    # Throw away most of the Allocation info; just the name of the task and whether it failed or not
    JQ_COMMAND=$(
        cat << EOF
        .[0].TaskStates | to_entries | map(
            {
                key, 
                value: {
                    "Failed": .value.Failed
                }
            }
        ) | from_entries
EOF
    )
    nomad_get_allocation "${job_id}" | jq "${JQ_COMMAND}"
}

nomad_dispatch_status() {
    # returns one of "pending", "running", "dead".
    # useful for knowing if your Dispatch is still running.
    local -r job_id="${1}"
    local -r curl_result=$(nomad_get_job "${job_id}")
    echo "${curl_result}" | jq -r ".Status"
}

await_nomad_dispatch_finish() {
    # Just keep trying until the Dispatch has run to completion (or timeout)
    local -r job_id=$1
    local -r attempts=$2 # in sleep-seconds

    local status
    # The below could be replaced with blocking queries on Nomad.
    for i in $(seq 1 "${attempts}"); do
        status=$(nomad_dispatch_status "${job_id}")
        if [ "${status}" = "dead" ]; then
            echo >&2 -ne "\nIntegration tests complete\n"
            return 0
        else
            # the `\r` lets us rewrite the last line
            echo >&2 -ne "[${i}/${attempts}] Integration tests still running - status: ${status}"\\r
            sleep 1
        fi
    done
    echo >&2 -ne "\nIntegration tests timed out - perhaps add more attempts?"
    return 1
}

nomad_get_job() {
    # Assumes there's a single job matching job_id
    local -r job_id="${1}"
    curl_quiet --request GET "${NOMAD_ENDPOINT}/v1/jobs?prefix=${job_id}" | jq -r ".[0]"
}

check_for_task_failures_in_job() {
    local -r job_id="${1}"

    local -r job_summary=$(nomad_get_job "${job_id}" | jq ".JobSummary.Summary")
    # Let the users know the full summary
    echo >&2 "${job_summary}"

    # Sum/accum each 'failed'
    local -r num_failed=$(echo "${job_summary}" | jq -r "[.[].Failed] | add")

    if [ "${num_failed}" != "0" ]; then
        # the `-e` and the weird escape codes are for color
        important_looking_banner "${num_failed} jobs failed - exiting!"
        return 42
    fi
}

important_looking_banner() {
    local -r message="${1}"
    echo -e "\n\n--- \e[30;46m${message}\e[m ---"
}
