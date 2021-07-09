#!/usr/bin/env bash

# TODO: Deduplicate - https://github.com/grapl-security/issue-tracker/issues/614

# Build our AMI in Buildkite, supplying the necessary
# information to create metadata tags.
# This is called from .buildkite/pipeline.merge.ami-build.yml

set -euo pipefail

source .buildkite/scripts/lib/packer.sh

export GIT_SHA="${BUILDKITE_COMMIT}"
export GIT_BRANCH="${BUILDKITE_BRANCH}"
# Marks these two as required
: "${BUILDKITE_BUILD_NUMBER}"
: "${PACKER_IMAGE_NAME}"

buildPackerCI() {
    echo -e "--- :packer: Performing build of AMI"

    # Both defined in packer.sh
    build_ami "${PACKER_IMAGE_NAME}"
    upload_manifest "${PACKER_IMAGE_NAME}"
}

buildPackerCI
