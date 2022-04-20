#!/bin/bash
set -euo pipefail

THIS_DIR=$(dirname "${BASH_SOURCE[0]}")
GRAPL_ROOT="$(git rev-parse --show-toplevel)"
LOCAL_INFRA_NOMAD_FILE="${GRAPL_ROOT}/nomad/local/grapl-local-infra.nomad"
OBSERVABILITY_NOMAD_FILE="${GRAPL_ROOT}/nomad/local/observability.nomad"

declare -a NOMAD_VARS=(
    -var "image_tag=${IMAGE_TAG}"
)

# shellcheck source-path=SCRIPTDIR
source "${THIS_DIR}/../lib/nomad_cli_tools.sh"

echo "Deploying Nomad local infrastructure"

# Wait a short period of time before attempting to deploy infrastructure
(
    readonly wait_secs=45
    # shellcheck disable=SC2016
    timeout --foreground "${wait_secs}" bash -c -- "$(
        cat << EOF
            wait_attempt=1
            while [[ -z \$(nomad node status 2>&1 | grep ready) ]]; do
                echo "Waiting for nomad-agent [\${wait_attempt}/${wait_secs}]"
                sleep 1
                ((wait_attempt=wait_attempt+1))
            done
EOF
    )"
)

# Do a Validate before a Plan. Helps end-users catch errors.
nomad job validate "${NOMAD_VARS[@]}" "${LOCAL_INFRA_NOMAD_FILE}"

# Okay, now the Nomad agent is up, but it might not be ready to accept jobs.
# Just loop on `nomad job plan` until it can.
attemptPlan() {
    nomad job plan "${NOMAD_VARS[@]}" "${LOCAL_INFRA_NOMAD_FILE}" > /dev/null 2>&1
    echo "$?"
}

nomad_run() {
    # Get the last word in this output, which is the eval id
    nomad job run -detach "${@}" | tail -n1 | awk '{print $NF}'
}

# fyi: Exit code 1 means "Allocations created or destroyed" and is what we want
while [[ $(attemptPlan) -ne 1 ]]; do
    echo "Waiting to be able to do a nomad-job-run"
    sleep 1
done

# Kick off all your nomad jobs in parallel
nomad_evals=()
nomad_evals+=("$(nomad_run "${NOMAD_VARS[@]}" "${LOCAL_INFRA_NOMAD_FILE}")")
nomad_evals+=("$(nomad_run "${OBSERVABILITY_NOMAD_FILE}")")

# `-monitor` will wait for each one to complete
for eval_id in "${nomad_evals[@]}"; do
    echo "Monitoring eval ${eval_id}..."
    nomad eval status -monitor "${eval_id}"
done

check_for_task_failures_in_job "grapl-local-infra"
check_for_task_failures_in_job "observability"

echo "Nomad Local Infra deployed!"
