#!/usr/bin/env bash

set -euo pipefail

# Localizing the core "logic" (such as it is) for building our AMI. We
# use this both to build locally and in CI.
build_ami() {
    echo "packer build ${PACKER_VARS} packer/nomad-server"
    packer build ${PACKER_VARS} packer/nomad-server
}
