#!/bin/bash
set -euo pipefail

NOMAD_ADDRESS="${NOMAD_ADDRESS:-http://localhost:4646}"

curl_quiet() {
    if [[ ${NOMAD_ADDRESS} =~ ^https://.*$ ]]; then
        # shellcheck disable=SC2068
        curl --proto "=https" --tlsv1.2 --location --silent --show-error $@
    else
        # shellcheck disable=SC2068
        curl --location --silent --show-error $@
    fi
}

nomad_allocations_for_eval() {
    local -r eval_id="${1}"
    local -r output="$(curl_quiet --request GET "${NOMAD_ADDRESS}/v1/evaluation/${eval_id}/allocations")"
    local -r alloc_ids="$(echo "${output}" | jq -r '[ .[].ID ]')"
    echo "${alloc_ids}"
}

nomad_dispatch() {
    local -r parameterized_batch_job="${1}"
    local -r dispatch_output="$(curl_quiet --request POST --data "{}" "${NOMAD_ADDRESS}/v1/job/${parameterized_batch_job}/dispatch")"
    echo "${dispatch_output}"
}

url_to_nomad_job_in_ui() {
    local -r job_id="${1}"
    # urlencode
    local -r urlencode_job_id=$(jq -rn --arg input "${job_id}" '$input|@uri')
    echo "${NOMAD_ADDRESS}/ui/jobs/${urlencode_job_id}"
}

nomad_stop_job() {
    local -r job_id="${1}"
    local -r dispatch_output=$(curl_quiet --request DELETE --data "{}" "${NOMAD_ADDRESS}/v1/job/${job_id}")
}

nomad_get_per_task_results() {
    # Returns something like {
    #   "graph-merger-integration-tests": {
    #     "Complete": 1
    #   },
    #   "graphql-endpoint-tests": {
    #     "Failed": 1
    #   }
    # }
    local -r job_id="${1}"

    jq_filter_out_zero_fields=$(
        cat << EOF
        .JobSummary.Summary | to_entries | map({
            key,
            value: .value | to_entries | map(select(.value > 0)) | from_entries
        }) | from_entries
EOF
    )
    local -r job_summary=$(nomad_get_job "${job_id}" | jq "${jq_filter_out_zero_fields}")
    echo "${job_summary}"
}

nomad_job_status() {
    # returns one of "pending", "running", "dead".
    # useful for knowing if your Dispatch is still running.
    local -r job_id="${1}"
    local -r curl_result=$(nomad_get_job "${job_id}")
    echo "${curl_result}" | jq -r ".Status"
}

await_nomad_job_finish() {
    # Just keep trying until the Dispatch has run to completion (or timeout)
    local -r job_id=$1
    local -r attempts=$2 # in sleep-seconds
    local -r label=$3

    local status
    # The below could be replaced with blocking queries on Nomad.
    for i in $(seq 1 "${attempts}"); do
        status=$(nomad_job_status "${job_id}")
        if [ "${status}" = "dead" ]; then
            echo >&2 -ne "\n${label} complete\n"
            return 0
        else
            # the `\r` lets us rewrite the last line
            echo >&2 -ne "[${i}/${attempts}] ${label} still running - status: ${status}"\\r
            sleep 1
        fi
    done
    echo >&2 -ne "\n${label} timed out - perhaps add more attempts?"
    return 1
}

nomad_get_job() {
    # Assumes there's a single job matching job_id
    local -r job_id="${1}"
    curl_quiet --request GET "${NOMAD_ADDRESS}/v1/jobs?prefix=${job_id}" | jq -r ".[0]"
}

check_for_task_failures_in_job() {
    local -r job_id="${1}"

    local -r job_summary=$(nomad_get_job "${job_id}" | jq ".JobSummary.Summary")

    # Sum/accum each 'failed'
    local -r num_failed=$(echo "${job_summary}" | jq -r "[.[].Failed] | add")

    if [ "${num_failed}" != "0" ]; then
        # the `-e` and the weird escape codes are for color
        important_looking_banner "${num_failed} jobs failed - exiting!"
        nomad_stop_job "${job_id}"
        return 52
    fi
}

nomad_get_alloc_id_for_task() {
    # Inspired by
    # https://github.com/hashicorp/nomad/issues/698#issuecomment-1031683060
    local -r job_id="${1}"
    local -r task_id="${2}"
    nomad job status "${job_id}" | grep "${task_id}" | awk '/run(.*)running/{print $1}'
}

important_looking_banner() {
    local -r message="${1}"
    echo -e "\n\n--- \e[30;46m${message}\e[m ---\n"
}
