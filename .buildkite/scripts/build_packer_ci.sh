#!/usr/bin/env bash

# Build our AMI in Buildkite, supplying the necessary
# information to create metadata tags.
# This is called from .buildkite/pipeline.merge.ami-build.yml

set -euo pipefail

source .buildkite/scripts/lib/packer.sh

# Also specified in the HCLs
readonly manifests=(
    "grapl-service.packer-manifest.json"
    "nomad-server.packer-manifest.json"
)

echo -e "--- :packer: Performing build of AMI"
export GIT_SHA="${BUILDKITE_COMMIT}"
export GIT_BRANCH="${BUILDKITE_BRANCH}"
: "${BUILDKITE_BUILD_NUMBER}"

# This is in the `packer.sh` sourced above
build_ami

for manifest in "${manifests[@]}"; do
    echo -e "--- :packer: Manifest ${manifest} Contents"
    cat "${manifest}"
    echo

    echo -e "--- :buildkite: Uploading ${manifest} file"
    buildkite-agent artifact upload "${manifest}"

    # Just to be safe, because subsequent runs can append to it
    rm "${manifest}"
done
