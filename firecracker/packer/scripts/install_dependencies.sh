#!/usr/bin/env bash

set -euo pipefail
set -o xtrace

########################################
# Install dependencies (currently, just debootstrap)
########################################
sudo apt-get update
# Print available versions for debugging purposes.
# (You might say, "madison? what's madison?" Just a badly named apt command.)
sudo apt-cache madison debootstrap
sleep 5
sudo apt-get install --yes --no-install-recommends \
    debootstrap=1.0.118ubuntu1.6
