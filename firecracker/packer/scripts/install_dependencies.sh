#!/usr/bin/env bash

set -euo pipefail
set -o xtrace

########################################
# Install dependencies (currently, just debootstrap)
########################################
sudo amazon-linux-extras install --assumeyes epel
sudo yum-config-manager --enable epel
sudo yum install --assumeyes debootstrap
