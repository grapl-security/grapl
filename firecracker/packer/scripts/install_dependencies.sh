#!/usr/bin/env bash

set -euo pipefail
set -o xtrace

########################################
# Install dependencies (currently, just debootstrap)
########################################
readonly APT_GET_WITH_LOCK=(apt-get -o dpkg::lock::timeout=10)
sudo "${APT_GET_WITH_LOCK[@]}" update

# Print available versions for debugging purposes.
# (You might say, "madison? what's madison?" Just a badly named apt command.)
sudo apt-cache madison debootstrap

sudo "${APT_GET_WITH_LOCK[@]}" install --yes --no-install-recommends \
    debootstrap=1.0.118ubuntu1.6
