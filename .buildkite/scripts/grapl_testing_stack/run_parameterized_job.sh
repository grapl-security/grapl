#!/bin/bash

##########
# Wrapper around `nomad/bin/run_parameterized_job.sh` that binds it to
# the `grapl/nomad/testing` stack.

# Automatically run `dumpArtifacts` afterwards.
##########
set -euo pipefail

STACK="grapl/nomad/testing"
readonly STACK

REPOSITORY_ROOT=$(git rev-parse --show-toplevel)
readonly REPOSITORY_ROOT

_NOMAD_ADDRESS=$(pulumi stack output address --stack="${STACK}")
readonly _NOMAD_ADDRESS

# Ensure we call dumpArtifacts even after test failure, and return exit code from
# the test, not the stop command.
dump_artifacts() {
    (
        cd "${REPOSITORY_ROOT}" &&
            NOMAD_ADDRESS="${_NOMAD_ADDRESS}" make dump-artifacts
    )
}
trap dump_artifacts EXIT

NOMAD_ADDRESS="${_NOMAD_ADDRESS}" \
    "${REPOSITORY_ROOT}/nomad/bin/run_parameterized_job.sh" "${@}"
