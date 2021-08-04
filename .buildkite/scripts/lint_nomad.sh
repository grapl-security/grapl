#!/usr/bin/env bash
set -euo pipefail

echo "--- Linting Nomad"
find nomad/ -type f -name "*.nomad" | xargs -n 1 packer fmt -check -diff