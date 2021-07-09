#!/usr/bin/env bash
# This test runs in `pipeline.verify.ami-test.yml.`
# It is *NOT* shunit.

# TODO: Deduplicate - https://github.com/grapl-security/issue-tracker/issues/614

set -euo pipefail

source .buildkite/scripts/lib/packer.sh

echo -e "--- :packer: Performing test build of AMI"
PACKER_VARS="-var build_ami=false" build_ami "${PACKER_IMAGE_NAME}"