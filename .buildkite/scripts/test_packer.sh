#!/usr/bin/env bash

# TODO: Deduplicate - https://github.com/grapl-security/issue-tracker/issues/614

set -euo pipefail

echo -e "--- :packer: Performing test build of AMI"

source .buildkite/scripts/lib/packer.sh

# We don't actually use anything in `constants` here, but make sure we can
# source it without errors.
source .buildkite/scripts/lib/packer_constants.sh

PACKER_VARS="-var build_ami=false" build_ami
