#!/usr/bin/env bash
set -euo pipefail

echo "--- Linting Nomad"
find nomad/ -type f -name "*.nomad" -print0 | xargs -n 1 --null packer fmt -check -diff
