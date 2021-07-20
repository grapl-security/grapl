#!/bin/sh
# Adapted from:
# - https://vector.dev/docs/setup/quickstart/
# - https://vector.dev/docs/setup/installation/package-managers/yum/
set -euo pipefail

# Add repo to YUM
curl -1sLf \
  'https://repositories.timber.io/public/vector/cfg/setup/bash.rpm.sh' \
| sudo -E bash

sudo yum install --assumeyes "vector-${VECTOR_VERSION}"

# Ensure it's all working correctly
vector --version