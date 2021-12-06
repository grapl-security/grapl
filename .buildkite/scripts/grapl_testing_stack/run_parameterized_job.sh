#!/bin/bash

##########
# Wrapper around `nomad/bin/run_parameterized_job.sh` that binds it to
# the `grapl/nomad/testing` stack.

# Automatically run `dumpArtifacts` afterwards.
##########
set -euo pipefail

# shellcheck source-path=SCRIPTDIR
source "$(dirname "${BASH_SOURCE[0]}")/lib.sh"

NOMAD_ADDRESS="$(get_nomad_address)"
readonly NOMAD_ADDRESS

# Ensure we call dumpArtifacts even after test failure, and return exit code from
# the test, not the stop command.
dump_artifacts() {
    (
        cd "${REPOSITORY_ROOT}" &&
            NOMAD_ADDRESS="${NOMAD_ADDRESS}" make dump-artifacts
    )
}
trap dump_artifacts EXIT

NOMAD_ADDRESS="${NOMAD_ADDRESS}" \
    "${REPOSITORY_ROOT}/nomad/bin/run_parameterized_job.sh" "${@}"
