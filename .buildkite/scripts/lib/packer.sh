#!/usr/bin/env bash

# Localizing the core "logic" (such as it is) for building our AMI. We
# use this both to build locally and in CI.
build_ami() {
    # Example usage:
    # PACKER_VARS="-var build_ami=false" build_ami

    # shellcheck disable=SC2086
    packer build ${PACKER_VARS:-} packer/nomad-server
    # shellcheck disable=SC2086
    packer build ${PACKER_VARS:-} packer/grapl-service
}
