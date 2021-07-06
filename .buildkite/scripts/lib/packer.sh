#!/usr/bin/env bash

set -euo pipefail

# Localizing the core "logic" (such as it is) for building our AMI. We
# use this both to build locally and in CI.
build_ami() {
    packer build packer/buildkite-base-ami.pkr.hcl ${PACKER_VARS}
}