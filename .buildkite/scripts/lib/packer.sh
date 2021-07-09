#!/usr/bin/env bash
set -euo pipefail

source .buildkite/scripts/lib/packer_constants.sh

# Localizing the core "logic" (such as it is) for building our AMI. We
# use this both to build locally and in CI.
build_ami() {
    # Example usage:
    # PACKER_VARS="-var build_ami=false" build_ami grapl-nomad-consul-server
    # PACKER_VARS="-var build_ami=false" build_ami grapl-nomad-consul-client

    local -r packer_image_name="$1"

    if [ "${packer_image_name}" == "${PACKER_IMAGE_NAME_SERVER}" ]; then
        # shellcheck disable=SC2086
        packer build ${PACKER_VARS:-} -var is_server=true packer/nomad-server-client
    elif [ "${packer_image_name}" == "${PACKER_IMAGE_NAME_CLIENT}" ]; then
        # shellcheck disable=SC2086
        packer build ${PACKER_VARS:-} -var is_server=false packer/nomad-server-client
    else
        echo "Unknown PACKER_IMAGE_NAME ${packer_image_name}"
        exit 42
    fi
}

upload_manifest() {
    # Given a packer image name, upload its manifest to Buildkite
    local -r packer_image_name="$1"
    local -r manifest="${packer_image_name}${PACKER_MANIFEST_SUFFIX}"

    echo -e "--- :packer: Manifest ${manifest} Contents"
    cat "${manifest}"
    echo

    echo -e "--- :buildkite: Uploading ${manifest} file"
    buildkite-agent artifact upload "${manifest}"

    # Just to be safe, because subsequent runs can append to it
    rm "${manifest}"
}
