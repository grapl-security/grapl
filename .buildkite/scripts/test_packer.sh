#!/usr/bin/env bash

set -euo pipefail

echo -e "--- :packer: Performing test build of AMI"

source .buildkite/scripts/lib/packer.sh

PACKER_VARS="-var build_ami=false" build_ami
