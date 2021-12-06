#!/bin/bash

##########
# lib file to automate grabbing the address of `grapl/nomad/testing`
##########
set -euo pipefail

STACK="grapl/nomad/testing"
readonly STACK

REPOSITORY_ROOT=$(git rev-parse --show-toplevel)
readonly REPOSITORY_ROOT
export REPOSITORY_ROOT

NOMAD_ADDRESS=$(pulumi stack output address --stack="${STACK}")
readonly NOMAD_ADDRESS
export NOMAD_ADDRESS
