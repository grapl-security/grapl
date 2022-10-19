#!/bin/bash

##########
# Wrapper around `dump_artifacts` that binds it to
# the `grapl/nomad/testing` stack.
#
# Best used with Metahook post-command. Don't forget to set `artifact_paths:`!
##########
set -euo pipefail

NOMAD_ADDRESS=$(pulumi stack output address --stack="grapl/nomad/testing")
export NOMAD_ADDRESS

# dump-artifacts is a Grapl custom alias defined in pants.toml
./pants dump-artifacts --no-dump-agent-logs
