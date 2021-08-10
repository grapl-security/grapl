#!/usr/bin/env bash
# This script is meant to be run outside of Docker in a real, honest-to-goodness shell.
set -euo pipefail

GRAPL_ROOT="$(git rev-parse --show-toplevel)"
cd "${GRAPL_ROOT}"

# Make sure our tree is clean
git diff --exit-code || (echo "Your git tree is dirty; bailing!" && exit 42)

make grapl-template-generator

./bin/grapl-template-generator cool-test-service

