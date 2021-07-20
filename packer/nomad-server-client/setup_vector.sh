#!/bin/sh
# Adapted from:
# - https://vector.dev/docs/setup/quickstart/
# - https://vector.dev/docs/setup/installation/package-managers/yum/
set -euo pipefail

echo "--- Add repo"
curl -1sLf \
  'https://repositories.timber.io/public/vector/cfg/setup/bash.rpm.sh' \
| sudo -E bash

echo "--- Install Vector"
sudo yum install --assumeyes "vector-${VECTOR_VERSION}"

echo "--- Ensure it's all working correctly"
vector --version