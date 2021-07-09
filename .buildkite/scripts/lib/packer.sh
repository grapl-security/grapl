#!/usr/bin/env bash

# Localizing the core "logic" (such as it is) for building our AMI. We
# use this both to build locally and in CI.
build_ami() {
    # Example usage:
    # PACKER_VARS="-var build_ami=false" build_ami

    # TODO: parallelize these builds!

    # shellcheck disable=SC2086
    packer build ${PACKER_VARS:-} -var is_server=true packer/nomad-server-client
    # shellcheck disable=SC2086
    packer build ${PACKER_VARS:-} -var is_server=false packer/nomad-server-client
}
