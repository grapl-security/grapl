#!/bin/sh
# Adapted from:
# - https://vector.dev/docs/setup/quickstart/
# - https://vector.dev/docs/setup/installation/package-managers/apt/
set -euo pipefail

curl -1sLf \
  'https://repositories.timber.io/public/vector/cfg/setup/bash.rpm.sh' \
| sudo -E bash


sudo yum --showduplicates list vector | expand
sudo yum install "vector-${VECTOR_VERSION}"

# Ensure it's all working correctly
vector --version