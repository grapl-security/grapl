#!/bin/bash
set -euo pipefail

THIS_DIR=$(dirname "${BASH_SOURCE[0]}")

# shellcheck source-path=SCRIPTDIR
source "${THIS_DIR}/nomad_cli_tools.sh"

# The Nomad integration test _definition_ is uploaded as part of
# __main__.py's `nomad_integration_tests`.

# Now we have to actually dispatch a job; Pulumi simply uploaded
# the jobspec, since it's a Parameterized Batch Job.

echo "--- Dispatching integration tests"

job_id=$(nomad_dispatch integration-tests)
echo "${job_id}"

dispatch_timed_out=0
await_nomad_dispatch_finish "${job_id}" $((5 * 60)) && dispatch_timed_out=1

# Show how each job did
# TODO: It'd be nice to show this *during* the await_nomad_dispatch_finish,
nomad_get_per_task_results "${job_id}"

# Exit if anything failed (thanks -euo pipefail!)
check_for_task_failures_in_job "${job_id}"

if [ "${dispatch_timed_out}" -ne "0" ]; then
    important_looking_banner "Integration tests timed out."
    nomad_stop_job "${job_id}"
    sleep 5
    exit 42
fi
