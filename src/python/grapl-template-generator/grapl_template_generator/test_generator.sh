#!/usr/bin/env bash

# This script is meant to be run in a real, honest-to-goodness shell.
# No Docker, no Pants, just a real Buildkite host or dev machine.

set -euo pipefail

GRAPL_ROOT="$(git rev-parse --show-toplevel)"
cd "${GRAPL_ROOT}"

# Make sure our tree is clean
git diff HEAD --exit-code > /dev/null 2>&1 || (echo "Your git tree is dirty; bailing!" && exit 42)

make grapl-template-generator

./bin/grapl-template-generator cool-test-service

