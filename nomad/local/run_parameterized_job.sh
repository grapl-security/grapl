#!/bin/bash
set -euo pipefail

THIS_DIR=$(dirname "${BASH_SOURCE[0]}")

# shellcheck source-path=SCRIPTDIR
source "${THIS_DIR}/nomad_cli_tools.sh"

# The Nomad test _definition_ is uploaded as part of
# Pulumi __main__.py's `NomadJob(...)` calls.

readonly job_to_dispatch="${1}"
readonly mins_to_wait="${2}"

# Now we have to actually dispatch a job; Pulumi simply uploaded
# the jobspec, since it's a Parameterized Batch Job.
echo "--- Dispatching ${job_to_dispatch}"

job_id=$(nomad_dispatch "${job_to_dispatch}")
echo "You can view job progress at $(url_to_nomad_job_in_ui "${job_id}")"

dispatch_timed_out=0
await_nomad_dispatch_finish \
    "${job_id}" \
    $(($mins_to_wait * 60)) \
    "${job_to_dispatch}" ||
    dispatch_timed_out=1

# Show how each job did
# TODO: It'd be nice to show this *during* the await_nomad_dispatch_finish,
nomad_get_per_task_results "${job_id}"

# Exit if anything failed (thanks -euo pipefail!)
check_for_task_failures_in_job "${job_id}"

if [ "${dispatch_timed_out}" -ne "0" ]; then
    important_looking_banner "${job_to_dispatch} timed out."
    nomad_stop_job "${job_id}"
    sleep 5
    exit 42
fi
