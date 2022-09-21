#!/bin/bash

##########
# Wrapper around `dump_artifacts` that binds it to
# the `grapl/nomad/testing` stack.
#
# Best used with Metahook post-command. Don't forget to set `artifact_paths:`!
##########
set -euo pipefail

REPOSITORY_ROOT=$(git rev-parse --show-toplevel)
readonly REPOSITORY_ROOT

readonly STACK="grapl/nomad/testing"
_NOMAD_ADDRESS=$(pulumi stack output address --stack="${STACK}")
readonly _NOMAD_ADDRESS

cd "${REPOSITORY_ROOT}"
# dump-artifacts is a Grapl custom alias defined in pants.toml
NOMAD_ADDRESS="${_NOMAD_ADDRESS}" ./pants dump-artifacts --no-dump-agent-logs
