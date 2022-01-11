#!/bin/bash

##########
# Wrapper around `nomad/bin/run_parameterized_job.sh` that binds it to
# the `grapl/nomad/testing` stack.
##########
set -euo pipefail

readonly STACK="grapl/nomad/testing"

REPOSITORY_ROOT=$(git rev-parse --show-toplevel)
readonly REPOSITORY_ROOT

_NOMAD_ADDRESS=$(pulumi stack output address --stack="${STACK}")
readonly _NOMAD_ADDRESS

NOMAD_ADDRESS="${_NOMAD_ADDRESS}" \
    "${REPOSITORY_ROOT}/nomad/bin/run_parameterized_job.sh" "${@}"
