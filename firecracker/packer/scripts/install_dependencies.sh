#!/usr/bin/env bash

set -euo pipefail
set -o xtrace

########################################
# Install dependencies (currently, just debootstrap)
########################################
sudo apt-get update
sudo apt-cache madison debootstrap
sudo apt-get install debootstrap=1.0.118ubuntu1
