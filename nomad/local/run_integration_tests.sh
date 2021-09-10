#!/bin/bash
set -euo pipefail

THIS_DIR=$(dirname "${BASH_SOURCE[0]}")

# shellcheck source-path=SCRIPTDIR
source "${THIS_DIR}/nomad_cli_tools.sh"

# The Nomad integration test _definition_ is uploaded as part of
# __main__.py's `nomad_integration_tests`.

# Now we have to actually dispatch a job; Pulumi simply uploaded
# the jobspec, since it's a Parameterized Batch Job.

# There's a potential race condition where pulumi may not have finished uploading the jobspec
# shellcheck disable=SC2016
timeout 30 bash -c -- 'while [[ -z $(nomad job inspect integration-tests 2>&1 | grep running) ]]; do printf "Waiting for jobspec\n";sleep 1;done'

echo "--- Dispatching integration tests"

job_id=$(nomad_dispatch integration-tests)
echo "${job_id}"

await_nomad_dispatch_finish "${job_id}" $((5 * 60))

# Show how each job did
nomad_get_per_task_results "${job_id}"

# Exit if anything failed (thanks -euo pipefail!)
check_for_task_failures_in_job "${job_id}"
