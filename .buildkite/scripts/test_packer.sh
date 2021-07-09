#!/usr/bin/env bash

# TODO: Deduplicate - https://github.com/grapl-security/issue-tracker/issues/614

set -euo pipefail

echo -e "--- :packer: Performing test build of AMI"

source .buildkite/scripts/lib/packer.sh

PACKER_VARS="-var build_ami=false" build_ami
